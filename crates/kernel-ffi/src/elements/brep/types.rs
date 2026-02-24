use crate::elements::brep::ids::{EdgeId, FaceId, LoopId, ShellId, SolidId, TrimId, VertexId};
use crate::{RgmObjectHandle, RgmPoint3, RgmUv2};
use index_vec::IndexVec;
use smallvec::SmallVec;

#[derive(Clone, Copy, Debug)]
pub(crate) struct BrepAabb3 {
    pub(crate) min: RgmPoint3,
    pub(crate) max: RgmPoint3,
}

// B1: Explicit UV field; point_3d is world-space (from 3D edge curve), uv is parameter-space.
// L1: manifold vertex has exactly 2 incident edges — SmallVec<[EdgeId; 2]>.
#[derive(Clone, Debug)]
pub(crate) struct BrepVertex {
    pub(crate) point_3d: Option<RgmPoint3>,
    pub(crate) uv: RgmUv2,
    pub(crate) tol: f64,
    pub(crate) incident_edges: SmallVec<[EdgeId; 2]>,
}

#[derive(Clone, Debug)]
pub(crate) struct BrepEdge {
    pub(crate) curve_3d: Option<RgmObjectHandle>,
    pub(crate) v_start: VertexId,
    pub(crate) v_end: VertexId,
    pub(crate) trims: SmallVec<[TrimId; 2]>,
}

// P2+S4: Plain SmallVec replaces Trim2dRep enum. Empty = unspecified; len >= 2 = polyline.
// The 2-point inline case (straight-line trim) requires no heap allocation.
#[derive(Clone, Debug)]
pub(crate) struct BrepTrim {
    pub(crate) edge: EdgeId,
    pub(crate) face: FaceId,
    pub(crate) loop_id: LoopId,
    pub(crate) uv_curve: SmallVec<[RgmUv2; 2]>,
    pub(crate) reversed: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct BrepLoop {
    pub(crate) trims: SmallVec<[TrimId; 8]>,
    pub(crate) is_outer: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct BrepFace {
    pub(crate) surface: RgmObjectHandle,
    pub(crate) loops: SmallVec<[LoopId; 4]>,
    // L3: TODO(v2): apply orientation in tessellation (flip normal when orientation < 0)
    pub(crate) orientation: i8,
    pub(crate) bbox: Option<BrepAabb3>,
}

#[derive(Clone, Debug)]
pub(crate) struct BrepShell {
    pub(crate) faces: Vec<FaceId>,
    pub(crate) closed: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct BrepSolid {
    pub(crate) shells: Vec<ShellId>,
}

// S2+S3: Removed dead fields (edge_to_faces, vertex_to_edges, shell_bbox, bbox_dirty).
// Removed separate dirty flags (adjacency_dirty, area_dirty); is_none() is the single indicator.
// TODO(v2): add edge_to_faces, vertex_to_edges caches when traversal is needed.
#[derive(Clone, Debug, Default)]
pub(crate) struct BrepCache {
    pub(crate) area_estimate: Option<f64>,
    pub(crate) face_neighbors: Option<Vec<SmallVec<[FaceId; 6]>>>,
}

impl BrepCache {
    pub(crate) fn invalidate(&mut self) {
        self.area_estimate = None;
        self.face_neighbors = None;
    }
}

// S1: finalized: bool replaces the BrepInProgress/Brep enum split.
#[derive(Clone, Debug, Default)]
pub(crate) struct BrepData {
    pub(crate) vertices: IndexVec<VertexId, BrepVertex>,
    pub(crate) edges: IndexVec<EdgeId, BrepEdge>,
    pub(crate) trims: IndexVec<TrimId, BrepTrim>,
    pub(crate) loops: IndexVec<LoopId, BrepLoop>,
    pub(crate) faces: IndexVec<FaceId, BrepFace>,
    pub(crate) shells: IndexVec<ShellId, BrepShell>,
    pub(crate) solids: IndexVec<SolidId, BrepSolid>,
    pub(crate) cache: BrepCache,
    pub(crate) finalized: bool,
}

impl BrepData {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn invalidate_topology(&mut self) {
        self.cache.invalidate();
    }

    pub(crate) fn invalidate_geometry(&mut self) {
        self.cache.area_estimate = None;
    }
}
