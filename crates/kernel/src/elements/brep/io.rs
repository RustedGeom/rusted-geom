use crate::elements::brep::ids::{EdgeId, FaceId, LoopId, ShellId, TrimId, VertexId};
use crate::elements::brep::types::{
    BrepData, BrepEdge, BrepFace, BrepLoop, BrepShell, BrepSolid, BrepTrim, BrepVertex,
};
use crate::{RgmStatus, RgmUv2};
use bincode::{Decode, Encode};
use smallvec::SmallVec;

const BREP_MAGIC: u32 = 0x5247_4D42;
// Version 2: BrepVertex stores uv:[f64;2] instead of point:[f64;3]; uv_curve is Vec<[f64;2]>;
// BrepEdge.curve_3d is Option<u64> instead of bool+u64.
const BREP_VERSION: u32 = 2;

#[derive(Encode, Decode)]
struct BrepFileEnvelope {
    magic: u32,
    version: u32,
    snapshot: BrepSnapshot,
}

#[derive(Encode, Decode)]
struct BrepSnapshot {
    vertices: Vec<BrepVertexSnapshot>,
    edges: Vec<BrepEdgeSnapshot>,
    trims: Vec<BrepTrimSnapshot>,
    loops: Vec<BrepLoopSnapshot>,
    faces: Vec<BrepFaceSnapshot>,
    shells: Vec<BrepShellSnapshot>,
    solids: Vec<BrepSolidSnapshot>,
}

// B1: Store uv:[f64;2] instead of point:[f64;3]. point_3d is not persisted (trim-only vertices).
#[derive(Encode, Decode)]
struct BrepVertexSnapshot {
    uv: [f64; 2],
    tol: f64,
    incident_edges: Vec<u32>,
}

// S5: Option<u64> instead of has_curve_3d:bool + curve_3d:u64. Eliminates corrupt state.
#[derive(Encode, Decode)]
struct BrepEdgeSnapshot {
    curve_3d: Option<u64>,
    v_start: u32,
    v_end: u32,
    trims: Vec<u32>,
}

// P2+S4: uv_curve is Vec<[f64; 2]>; empty = unspecified, len >= 2 = polyline.
// Replaces Trim2dRepSnapshot enum variant split.
#[derive(Encode, Decode)]
struct BrepTrimSnapshot {
    edge: u32,
    face: u32,
    loop_id: u32,
    uv_curve: Vec<[f64; 2]>,
    reversed: bool,
}

#[derive(Encode, Decode)]
struct BrepLoopSnapshot {
    trims: Vec<u32>,
    is_outer: bool,
}

#[derive(Encode, Decode)]
struct BrepFaceSnapshot {
    surface: u64,
    loops: Vec<u32>,
    orientation: i8,
}

#[derive(Encode, Decode)]
struct BrepShellSnapshot {
    faces: Vec<u32>,
    closed: bool,
}

#[derive(Encode, Decode)]
struct BrepSolidSnapshot {
    shells: Vec<u32>,
}

fn to_snapshot(brep: &BrepData) -> BrepSnapshot {
    BrepSnapshot {
        vertices: brep
            .vertices
            .iter()
            .map(|vertex| BrepVertexSnapshot {
                // B1: encode uv, not point
                uv: [vertex.uv.u, vertex.uv.v],
                tol: vertex.tol,
                incident_edges: vertex.incident_edges.iter().map(|id| id.raw()).collect(),
            })
            .collect(),
        edges: brep
            .edges
            .iter()
            .map(|edge| BrepEdgeSnapshot {
                // S5: Option<u64> directly
                curve_3d: edge.curve_3d.map(|v| v.0),
                v_start: edge.v_start.raw(),
                v_end: edge.v_end.raw(),
                trims: edge.trims.iter().map(|id| id.raw()).collect(),
            })
            .collect(),
        trims: brep
            .trims
            .iter()
            .map(|trim| BrepTrimSnapshot {
                edge: trim.edge.raw(),
                face: trim.face.raw(),
                loop_id: trim.loop_id.raw(),
                // P2+S4: encode SmallVec as Vec<[f64;2]>
                uv_curve: trim.uv_curve.iter().map(|uv| [uv.u, uv.v]).collect(),
                reversed: trim.reversed,
            })
            .collect(),
        loops: brep
            .loops
            .iter()
            .map(|loop_data| BrepLoopSnapshot {
                trims: loop_data.trims.iter().map(|id| id.raw()).collect(),
                is_outer: loop_data.is_outer,
            })
            .collect(),
        faces: brep
            .faces
            .iter()
            .map(|face| BrepFaceSnapshot {
                surface: face.surface.0,
                loops: face.loops.iter().map(|id| id.raw()).collect(),
                orientation: face.orientation,
            })
            .collect(),
        shells: brep
            .shells
            .iter()
            .map(|shell| BrepShellSnapshot {
                faces: shell.faces.iter().map(|id| id.raw()).collect(),
                closed: shell.closed,
            })
            .collect(),
        solids: brep
            .solids
            .iter()
            .map(|solid| BrepSolidSnapshot {
                shells: solid.shells.iter().map(|id| id.raw()).collect(),
            })
            .collect(),
    }
}

fn from_snapshot(snapshot: BrepSnapshot) -> BrepData {
    let mut brep = BrepData::new();

    for vertex in snapshot.vertices {
        brep.vertices.push(BrepVertex {
            // B1: restore uv from snapshot; point_3d is None for serialized trim vertices
            point_3d: None,
            uv: RgmUv2 {
                u: vertex.uv[0],
                v: vertex.uv[1],
            },
            tol: vertex.tol,
            // L1: collect into SmallVec<[EdgeId; 2]>
            incident_edges: vertex
                .incident_edges
                .into_iter()
                .map(EdgeId::from_raw)
                .collect::<SmallVec<[EdgeId; 2]>>(),
        });
    }

    for edge in snapshot.edges {
        brep.edges.push(BrepEdge {
            // S5: Option<u64> decoded directly
            curve_3d: edge.curve_3d.map(crate::RgmObjectHandle),
            v_start: VertexId::from_raw(edge.v_start),
            v_end: VertexId::from_raw(edge.v_end),
            trims: edge
                .trims
                .into_iter()
                .map(TrimId::from_raw)
                .collect::<SmallVec<[TrimId; 2]>>(),
        });
    }

    for trim in snapshot.trims {
        brep.trims.push(BrepTrim {
            edge: EdgeId::from_raw(trim.edge),
            face: FaceId::from_raw(trim.face),
            loop_id: LoopId::from_raw(trim.loop_id),
            // P2+S4: decode Vec<[f64;2]> into SmallVec<[RgmUv2; 2]>
            uv_curve: trim
                .uv_curve
                .into_iter()
                .map(|uv| RgmUv2 { u: uv[0], v: uv[1] })
                .collect::<SmallVec<[RgmUv2; 2]>>(),
            reversed: trim.reversed,
        });
    }

    for loop_data in snapshot.loops {
        brep.loops.push(BrepLoop {
            trims: loop_data
                .trims
                .into_iter()
                .map(TrimId::from_raw)
                .collect::<SmallVec<[TrimId; 8]>>(),
            is_outer: loop_data.is_outer,
        });
    }

    for face in snapshot.faces {
        brep.faces.push(BrepFace {
            surface: crate::RgmObjectHandle(face.surface),
            loops: face
                .loops
                .into_iter()
                .map(LoopId::from_raw)
                .collect::<SmallVec<[LoopId; 4]>>(),
            orientation: face.orientation,
            bbox: None,
        });
    }

    for shell in snapshot.shells {
        brep.shells.push(BrepShell {
            faces: shell.faces.into_iter().map(FaceId::from_raw).collect(),
            closed: shell.closed,
        });
    }

    for solid in snapshot.solids {
        brep.solids.push(BrepSolid {
            shells: solid.shells.into_iter().map(ShellId::from_raw).collect(),
        });
    }

    // S1: loaded BREPs are always finalized
    brep.finalized = true;
    brep.invalidate_topology();
    brep
}

pub(crate) fn encode_native_brep(brep: &BrepData) -> Result<Vec<u8>, RgmStatus> {
    let envelope = BrepFileEnvelope {
        magic: BREP_MAGIC,
        version: BREP_VERSION,
        snapshot: to_snapshot(brep),
    };
    bincode::encode_to_vec(envelope, bincode::config::standard())
        .map_err(|_| RgmStatus::InternalError)
}

pub(crate) fn decode_native_brep(bytes: &[u8]) -> Result<BrepData, RgmStatus> {
    let (envelope, _) =
        bincode::decode_from_slice::<BrepFileEnvelope, _>(bytes, bincode::config::standard())
            .map_err(|_| RgmStatus::InvalidInput)?;
    if envelope.magic != BREP_MAGIC || envelope.version != BREP_VERSION {
        return Err(RgmStatus::InvalidInput);
    }
    Ok(from_snapshot(envelope.snapshot))
}
