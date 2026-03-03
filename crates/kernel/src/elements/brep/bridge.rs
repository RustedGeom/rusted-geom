use crate::elements::brep::ids::{FaceId, LoopId};
use crate::elements::brep::types::{BrepData, BrepEdge, BrepFace, BrepLoop, BrepTrim, BrepVertex};
use crate::session::objects::{FaceData, TrimEdgeData, TrimLoopData};
use crate::{RgmObjectHandle, RgmStatus, RgmUv2};
use smallvec::SmallVec;

fn edge_samples(edge: &TrimEdgeData) -> Vec<RgmUv2> {
    if edge.uv_samples.len() >= 2 {
        edge.uv_samples.clone()
    } else {
        vec![edge.start_uv, edge.end_uv]
    }
}

fn polyline_endpoints(
    samples: &[RgmUv2],
    fallback_start: RgmUv2,
    fallback_end: RgmUv2,
) -> (RgmUv2, RgmUv2) {
    if samples.len() >= 2 {
        (
            samples.first().copied().unwrap_or(fallback_start),
            samples.last().copied().unwrap_or(fallback_end),
        )
    } else {
        (fallback_start, fallback_end)
    }
}

// C2: Store the primary shell ID from push() result and reference it directly.
fn push_face_to_primary_shell(brep: &mut BrepData, face_id: FaceId) {
    if brep.shells.is_empty() {
        brep.shells.push(crate::elements::brep::types::BrepShell {
            faces: vec![face_id],
            closed: false,
        });
    } else {
        // Use first_mut() rather than from_raw(0) to avoid index assumptions.
        if let Some(shell) = brep.shells.as_raw_slice_mut().first_mut() {
            shell.faces.push(face_id);
        }
    }
}

pub(crate) fn add_face_to_brep(brep: &mut BrepData, face: &FaceData) -> Result<FaceId, RgmStatus> {
    let face_id = brep.faces.push(BrepFace {
        surface: face.surface,
        loops: SmallVec::new(),
        orientation: 1,
        bbox: None,
    });

    for loop_data in &face.loops {
        add_loop_to_face(brep, face_id, loop_data)?;
    }

    push_face_to_primary_shell(brep, face_id);
    brep.invalidate_topology();
    Ok(face_id)
}

pub(crate) fn add_surface_face_to_brep(brep: &mut BrepData, surface: RgmObjectHandle) -> FaceId {
    let face_id = brep.faces.push(BrepFace {
        surface,
        loops: SmallVec::new(),
        orientation: 1,
        bbox: None,
    });
    push_face_to_primary_shell(brep, face_id);
    brep.invalidate_topology();
    face_id
}

pub(crate) fn add_uv_loop_to_face(
    brep: &mut BrepData,
    face_id: FaceId,
    points: &[RgmUv2],
    is_outer: bool,
) -> Result<LoopId, RgmStatus> {
    if points.len() < 3 {
        return Err(RgmStatus::InvalidInput);
    }
    if face_id.index() >= brep.faces.len() {
        return Err(RgmStatus::OutOfRange);
    }

    let mut loop_points = points.to_vec();
    if loop_points.first() == loop_points.last() {
        loop_points.pop();
    }
    if loop_points.len() < 3 {
        return Err(RgmStatus::InvalidInput);
    }

    let loop_id = brep.loops.push(BrepLoop {
        trims: SmallVec::new(),
        is_outer,
    });
    brep.faces[face_id].loops.push(loop_id);

    let n = loop_points.len();

    // P3: Create N shared vertices first (one per loop point), then N edges referencing them.
    // Previously each edge pushed 2 independent vertices, yielding 2N vertices for an N-edge loop.
    let vert_ids: Vec<_> = loop_points
        .iter()
        .map(|&uv| {
            brep.vertices.push(BrepVertex {
                point_3d: None,
                uv,
                tol: 1e-9,
                incident_edges: SmallVec::new(),
            })
        })
        .collect();

    for i in 0..n {
        let v_start = vert_ids[i];
        let v_end = vert_ids[(i + 1) % n];
        let start = loop_points[i];
        let end = loop_points[(i + 1) % n];

        let edge_id = brep.edges.push(BrepEdge {
            curve_3d: None,
            v_start,
            v_end,
            trims: SmallVec::new(),
        });
        brep.vertices[v_start].incident_edges.push(edge_id);
        brep.vertices[v_end].incident_edges.push(edge_id);

        // P2+S4: SmallVec<[RgmUv2; 2]> instead of Trim2dRep::Polyline(Vec<..>)
        let mut uv_curve: SmallVec<[RgmUv2; 2]> = SmallVec::new();
        uv_curve.push(start);
        uv_curve.push(end);

        let trim_id = brep.trims.push(BrepTrim {
            edge: edge_id,
            face: face_id,
            loop_id,
            uv_curve,
            reversed: false,
        });
        brep.edges[edge_id].trims.push(trim_id);
        brep.loops[loop_id].trims.push(trim_id);
    }

    brep.invalidate_topology();
    Ok(loop_id)
}

fn add_loop_to_face(
    brep: &mut BrepData,
    face_id: FaceId,
    loop_data: &TrimLoopData,
) -> Result<LoopId, RgmStatus> {
    if loop_data.edges.len() < 2 {
        return Err(RgmStatus::InvalidInput);
    }

    let loop_id = brep.loops.push(BrepLoop {
        trims: SmallVec::new(),
        is_outer: loop_data.is_outer,
    });
    brep.faces[face_id].loops.push(loop_id);

    let n = loop_data.edges.len();

    // P3: Create N shared vertices first, then N edges.
    // Collect (samples, start_uv, end_uv) for each edge first.
    let edge_info: Vec<(Vec<RgmUv2>, RgmUv2, RgmUv2)> = loop_data
        .edges
        .iter()
        .map(|edge_data| {
            let samples = edge_samples(edge_data);
            let (start_uv, end_uv) =
                polyline_endpoints(&samples, edge_data.start_uv, edge_data.end_uv);
            (samples, start_uv, end_uv)
        })
        .collect();

    let vert_ids: Vec<_> = edge_info
        .iter()
        .map(|(_, start_uv, _)| {
            brep.vertices.push(BrepVertex {
                point_3d: None,
                uv: *start_uv,
                tol: 1e-9,
                incident_edges: SmallVec::new(),
            })
        })
        .collect();

    for i in 0..n {
        let (ref samples, _, _) = edge_info[i];
        let v_start = vert_ids[i];
        let v_end = vert_ids[(i + 1) % n];

        let edge_id = brep.edges.push(BrepEdge {
            curve_3d: loop_data.edges[i].curve_3d,
            v_start,
            v_end,
            trims: SmallVec::new(),
        });
        brep.vertices[v_start].incident_edges.push(edge_id);
        brep.vertices[v_end].incident_edges.push(edge_id);

        let uv_curve: SmallVec<[RgmUv2; 2]> = samples.iter().copied().collect();

        let trim_id = brep.trims.push(BrepTrim {
            edge: edge_id,
            face: face_id,
            loop_id,
            uv_curve,
            reversed: false,
        });
        brep.edges[edge_id].trims.push(trim_id);
        brep.loops[loop_id].trims.push(trim_id);
    }

    Ok(loop_id)
}

pub(crate) fn face_from_brep(brep: &BrepData, face_id_raw: u32) -> Result<FaceData, RgmStatus> {
    let face_id = FaceId::from_raw(face_id_raw);
    if face_id.index() >= brep.faces.len() {
        return Err(RgmStatus::OutOfRange);
    }
    let face = &brep.faces[face_id];
    let mut loops = Vec::with_capacity(face.loops.len());

    for &loop_id in &face.loops {
        if loop_id.index() >= brep.loops.len() {
            return Err(RgmStatus::InvalidInput);
        }
        let loop_data = &brep.loops[loop_id];
        let mut edges = Vec::with_capacity(loop_data.trims.len());

        for &trim_id in &loop_data.trims {
            if trim_id.index() >= brep.trims.len() {
                return Err(RgmStatus::InvalidInput);
            }
            let trim = &brep.trims[trim_id];
            if trim.edge.index() >= brep.edges.len() {
                return Err(RgmStatus::InvalidInput);
            }
            let edge = &brep.edges[trim.edge];
            if edge.v_start.index() >= brep.vertices.len()
                || edge.v_end.index() >= brep.vertices.len()
            {
                return Err(RgmStatus::InvalidInput);
            }

            // P2+S4: uv_curve is now a plain SmallVec; len >= 2 means polyline.
            // B1: vertex UV accessed via .uv field instead of .point.x/.point.y
            let (mut start_uv, mut end_uv, mut samples) = if trim.uv_curve.len() >= 2 {
                (
                    trim.uv_curve[0],
                    trim.uv_curve[trim.uv_curve.len() - 1],
                    trim.uv_curve.to_vec(),
                )
            } else {
                let start = brep.vertices[edge.v_start].uv;
                let end = brep.vertices[edge.v_end].uv;
                (start, end, vec![start, end])
            };

            if trim.reversed {
                std::mem::swap(&mut start_uv, &mut end_uv);
                samples.reverse();
            }

            edges.push(TrimEdgeData {
                start_uv,
                end_uv,
                curve_3d: edge.curve_3d,
                uv_samples: samples,
            });
        }

        loops.push(TrimLoopData {
            edges,
            is_outer: loop_data.is_outer,
        });
    }

    Ok(FaceData {
        surface: face.surface,
        loops,
    })
}
