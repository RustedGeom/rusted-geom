//! Session object model: geometry data types, mesh BVH acceleration cache,
//! session state, and object-lookup helpers.
//!
//! All items are `pub(crate)` — they cross the boundary between the flat
//! `kernel_impl` `include!` scope and the real module graph.  The companion
//! module [`super::store`] holds the global session registry and insert helpers
//! that depend on types defined here.

use crate::math::arc_length::ArcLengthCache;
use crate::math::nurbs_curve_eval::NurbsCurveCore;
use crate::math::nurbs_surface_eval::NurbsSurfaceCore;
use crate::{RgmObjectHandle, RgmPoint3, RgmStatus, RgmUv2};
use std::collections::HashMap;

// ─── Geometry Data Types ──────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub(crate) struct NurbsCurveData {
    pub(crate) core: NurbsCurveCore,
    pub(crate) closed: bool,
    pub(crate) fit_points: Vec<RgmPoint3>,
    pub(crate) arc_length: ArcLengthCache,
}

#[derive(Clone, Debug)]
pub(crate) struct PolycurveSegmentData {
    pub(crate) curve: RgmObjectHandle,
    pub(crate) reversed: bool,
    pub(crate) length: f64,
}

#[derive(Clone, Debug)]
pub(crate) struct PolycurveData {
    pub(crate) segments: Vec<PolycurveSegmentData>,
    pub(crate) cumulative_lengths: Vec<f64>,
    pub(crate) total_length: f64,
}

#[derive(Clone, Debug)]
pub(crate) struct MeshData {
    pub(crate) vertices: Vec<RgmPoint3>,
    pub(crate) triangles: Vec<[u32; 3]>,
    pub(crate) transform: [[f64; 4]; 4],
}

#[derive(Clone, Debug)]
pub(crate) struct SurfaceData {
    pub(crate) core: NurbsSurfaceCore,
    pub(crate) transform: [[f64; 4]; 4],
}

#[derive(Clone, Debug)]
pub(crate) struct TrimEdgeData {
    pub(crate) start_uv: RgmUv2,
    pub(crate) end_uv: RgmUv2,
    pub(crate) curve_3d: Option<RgmObjectHandle>,
    pub(crate) uv_samples: Vec<RgmUv2>,
}

#[derive(Clone, Debug)]
pub(crate) struct TrimLoopData {
    pub(crate) edges: Vec<TrimEdgeData>,
    pub(crate) is_outer: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct FaceData {
    pub(crate) surface: RgmObjectHandle,
    pub(crate) loops: Vec<TrimLoopData>,
}

#[derive(Clone, Debug)]
pub(crate) struct IntersectionBranchData {
    pub(crate) points: Vec<RgmPoint3>,
    pub(crate) uv_a: Vec<RgmUv2>,
    pub(crate) uv_b: Vec<RgmUv2>,
    pub(crate) curve_t: Vec<f64>,
    pub(crate) closed: bool,
    pub(crate) flags: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct IntersectionData {
    pub(crate) branches: Vec<IntersectionBranchData>,
}

#[derive(Clone, Debug)]
pub(crate) enum CurveData {
    NurbsCurve(NurbsCurveData),
    /// A line segment stored as its canonical NURBS representation.
    Line(NurbsCurveData),
    /// An arc stored as its canonical NURBS representation.
    Arc(NurbsCurveData),
    /// A full circle stored as its canonical NURBS representation.
    Circle(NurbsCurveData),
    /// A polyline stored as its canonical NURBS representation.
    Polyline(NurbsCurveData),
    Polycurve(PolycurveData),
}

#[derive(Clone, Debug)]
pub(crate) enum GeometryObject {
    Curve(CurveData),
    Mesh(MeshData),
    Surface(SurfaceData),
    Face(FaceData),
    Intersection(IntersectionData),
}

// ─── Mesh BVH Acceleration Cache ─────────────────────────────────────────────
//
// BVH (bounding-volume hierarchy) structures used to accelerate ray–mesh and
// plane–mesh intersection queries.  One `MeshAccelCache` is stored per mesh in
// `SessionState.mesh_accels`.  The build algorithm and traversal code live in
// `kernel_impl/intersection_mesh_ops_b.rs`.

fn triangle_aabb(points: [RgmPoint3; 3]) -> (RgmPoint3, RgmPoint3) {
    let mut min = points[0];
    let mut max = points[0];
    for point in points.iter().skip(1) {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        min.z = min.z.min(point.z);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
        max.z = max.z.max(point.z);
    }
    (min, max)
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TriangleRecord {
    pub(crate) points: [RgmPoint3; 3],
    pub(crate) min: RgmPoint3,
    pub(crate) max: RgmPoint3,
}

impl TriangleRecord {
    pub(crate) fn from_mesh(vertices: &[RgmPoint3], tri: [u32; 3]) -> Self {
        let points = [
            vertices[tri[0] as usize],
            vertices[tri[1] as usize],
            vertices[tri[2] as usize],
        ];
        let (min, max) = triangle_aabb(points);
        Self { points, min, max }
    }
}

pub(crate) struct MeshAccelCache {
    pub(crate) triangles: Vec<TriangleRecord>,
    pub(crate) bvh: Option<MeshBvh>,
}

#[derive(Clone, Copy)]
pub(crate) struct BvhNode {
    pub(crate) min: RgmPoint3,
    pub(crate) max: RgmPoint3,
    pub(crate) left: Option<usize>,
    pub(crate) right: Option<usize>,
    pub(crate) start: usize,
    pub(crate) count: usize,
}

impl BvhNode {
    pub(crate) fn leaf(min: RgmPoint3, max: RgmPoint3, start: usize, count: usize) -> Self {
        Self {
            min,
            max,
            left: None,
            right: None,
            start,
            count,
        }
    }

    pub(crate) fn branch(min: RgmPoint3, max: RgmPoint3, left: usize, right: usize) -> Self {
        Self {
            min,
            max,
            left: Some(left),
            right: Some(right),
            start: 0,
            count: 0,
        }
    }

    pub(crate) fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

pub(crate) struct MeshBvh {
    pub(crate) root: usize,
    pub(crate) nodes: Vec<BvhNode>,
    pub(crate) tri_indices: Vec<usize>,
}

impl MeshBvh {
    pub(crate) fn build(records: &[TriangleRecord]) -> Option<Self> {
        if records.is_empty() {
            return None;
        }
        let mut tri_indices = (0..records.len()).collect::<Vec<_>>();
        let tri_count = tri_indices.len();
        let mut nodes = Vec::new();
        let root = Self::build_node(records, &mut tri_indices, &mut nodes, 0, tri_count);
        Some(Self {
            root,
            nodes,
            tri_indices,
        })
    }

    fn build_node(
        records: &[TriangleRecord],
        tri_indices: &mut Vec<usize>,
        nodes: &mut Vec<BvhNode>,
        start: usize,
        end: usize,
    ) -> usize {
        let node_idx = nodes.len();
        nodes.push(BvhNode::leaf(
            RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 },
            RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 },
            start,
            end.saturating_sub(start),
        ));

        let (min, max) = Self::range_bounds(records, &tri_indices[start..end]);
        let count = end.saturating_sub(start);
        const LEAF_TRIANGLES: usize = 8;
        if count <= LEAF_TRIANGLES {
            nodes[node_idx] = BvhNode::leaf(min, max, start, count);
            return node_idx;
        }

        let axis = Self::split_axis(records, &tri_indices[start..end]);
        tri_indices[start..end].sort_by(|a, b| {
            let ca = Self::triangle_centroid_axis(records[*a], axis);
            let cb = Self::triangle_centroid_axis(records[*b], axis);
            ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mid = start + count / 2;
        if mid == start || mid == end {
            nodes[node_idx] = BvhNode::leaf(min, max, start, count);
            return node_idx;
        }

        let left = Self::build_node(records, tri_indices, nodes, start, mid);
        let right = Self::build_node(records, tri_indices, nodes, mid, end);
        nodes[node_idx] = BvhNode::branch(min, max, left, right);
        node_idx
    }

    fn range_bounds(records: &[TriangleRecord], indices: &[usize]) -> (RgmPoint3, RgmPoint3) {
        let mut min = records[indices[0]].min;
        let mut max = records[indices[0]].max;
        for &idx in indices.iter().skip(1) {
            let tri = records[idx];
            min.x = min.x.min(tri.min.x);
            min.y = min.y.min(tri.min.y);
            min.z = min.z.min(tri.min.z);
            max.x = max.x.max(tri.max.x);
            max.y = max.y.max(tri.max.y);
            max.z = max.z.max(tri.max.z);
        }
        (min, max)
    }

    fn split_axis(records: &[TriangleRecord], indices: &[usize]) -> usize {
        let mut cmin = Self::triangle_centroid(records[indices[0]]);
        let mut cmax = cmin;
        for &idx in indices.iter().skip(1) {
            let c = Self::triangle_centroid(records[idx]);
            cmin.x = cmin.x.min(c.x);
            cmin.y = cmin.y.min(c.y);
            cmin.z = cmin.z.min(c.z);
            cmax.x = cmax.x.max(c.x);
            cmax.y = cmax.y.max(c.y);
            cmax.z = cmax.z.max(c.z);
        }
        let ex = cmax.x - cmin.x;
        let ey = cmax.y - cmin.y;
        let ez = cmax.z - cmin.z;
        if ex >= ey && ex >= ez {
            0
        } else if ey >= ez {
            1
        } else {
            2
        }
    }

    fn triangle_centroid(record: TriangleRecord) -> RgmPoint3 {
        RgmPoint3 {
            x: (record.min.x + record.max.x) * 0.5,
            y: (record.min.y + record.max.y) * 0.5,
            z: (record.min.z + record.max.z) * 0.5,
        }
    }

    fn triangle_centroid_axis(record: TriangleRecord, axis: usize) -> f64 {
        match axis {
            0 => (record.min.x + record.max.x) * 0.5,
            1 => (record.min.y + record.max.y) * 0.5,
            _ => (record.min.z + record.max.z) * 0.5,
        }
    }
}

// ─── Session State ────────────────────────────────────────────────────────────

#[derive(Default)]
pub(crate) struct SessionState {
    pub(crate) objects: HashMap<u64, GeometryObject>,
    pub(crate) mesh_accels: HashMap<u64, MeshAccelCache>,
    pub(crate) last_error_code: RgmStatus,
    pub(crate) last_error_message: String,
}

// ─── Object Lookup Helpers ────────────────────────────────────────────────────

pub(crate) fn find_curve<'a>(
    state: &'a SessionState,
    object: RgmObjectHandle,
) -> Result<&'a CurveData, RgmStatus> {
    match state.objects.get(&object.0) {
        Some(GeometryObject::Curve(curve)) => Ok(curve),
        Some(_) => Err(RgmStatus::InvalidInput),
        None => Err(RgmStatus::NotFound),
    }
}

pub(crate) fn find_mesh<'a>(
    state: &'a SessionState,
    object: RgmObjectHandle,
) -> Result<&'a MeshData, RgmStatus> {
    match state.objects.get(&object.0) {
        Some(GeometryObject::Mesh(mesh)) => Ok(mesh),
        Some(_) => Err(RgmStatus::InvalidInput),
        None => Err(RgmStatus::NotFound),
    }
}

pub(crate) fn find_surface<'a>(
    state: &'a SessionState,
    object: RgmObjectHandle,
) -> Result<&'a SurfaceData, RgmStatus> {
    match state.objects.get(&object.0) {
        Some(GeometryObject::Surface(surface)) => Ok(surface),
        Some(_) => Err(RgmStatus::InvalidInput),
        None => Err(RgmStatus::NotFound),
    }
}

pub(crate) fn find_face<'a>(
    state: &'a SessionState,
    object: RgmObjectHandle,
) -> Result<&'a FaceData, RgmStatus> {
    match state.objects.get(&object.0) {
        Some(GeometryObject::Face(face)) => Ok(face),
        Some(_) => Err(RgmStatus::InvalidInput),
        None => Err(RgmStatus::NotFound),
    }
}

pub(crate) fn find_intersection<'a>(
    state: &'a SessionState,
    object: RgmObjectHandle,
) -> Result<&'a IntersectionData, RgmStatus> {
    match state.objects.get(&object.0) {
        Some(GeometryObject::Intersection(intersection)) => Ok(intersection),
        Some(_) => Err(RgmStatus::InvalidInput),
        None => Err(RgmStatus::NotFound),
    }
}

pub(crate) fn find_face_mut<'a>(
    state: &'a mut SessionState,
    object: RgmObjectHandle,
) -> Result<&'a mut FaceData, RgmStatus> {
    match state.objects.get_mut(&object.0) {
        Some(GeometryObject::Face(face)) => Ok(face),
        Some(_) => Err(RgmStatus::InvalidInput),
        None => Err(RgmStatus::NotFound),
    }
}

/// Returns a reference to the canonical `NurbsCurveData` for any curve variant
/// that has one, or `None` for `Polycurve`.
pub(crate) fn curve_canonical_nurbs(curve: &CurveData) -> Option<&NurbsCurveData> {
    match curve {
        CurveData::NurbsCurve(data)
        | CurveData::Line(data)
        | CurveData::Arc(data)
        | CurveData::Circle(data)
        | CurveData::Polyline(data) => Some(data),
        CurveData::Polycurve(_) => None,
    }
}
