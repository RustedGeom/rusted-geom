use crate::elements::brep::types::BrepData;
use crate::{RgmBrepEntityKind, RgmBrepValidationReport, RgmUv2, RgmValidationIssue, RgmValidationSeverity};
use smallvec::SmallVec;

fn uv_distance(a: RgmUv2, b: RgmUv2) -> f64 {
    let du = a.u - b.u;
    let dv = a.v - b.v;
    (du * du + dv * dv).sqrt()
}

// S6: Use RgmBrepEntityKind enum instead of raw integer literals.
fn push_issue(
    report: &mut RgmBrepValidationReport,
    severity: RgmValidationSeverity,
    code: u32,
    entity_kind: RgmBrepEntityKind,
    entity_id: u32,
    uv: Option<RgmUv2>,
) {
    let slot = report.issue_count as usize;
    report.issue_count = report.issue_count.saturating_add(1);
    if (severity as i32) > (report.max_severity as i32) {
        report.max_severity = severity;
    }
    if slot < report.issues.len() {
        report.issues[slot] = RgmValidationIssue {
            severity,
            code,
            entity_kind: entity_kind as u32,
            entity_id,
            param_u: uv.map(|v| v.u).unwrap_or(f64::NAN),
            param_v: uv.map(|v| v.v).unwrap_or(f64::NAN),
        };
    } else {
        report.overflow = true;
    }
}

fn trim_endpoints(brep: &BrepData, trim_index: usize) -> Option<(RgmUv2, RgmUv2)> {
    let trim = brep
        .trims
        .get(crate::elements::brep::ids::TrimId::from_usize(trim_index))?;
    let edge = brep.edges.get(trim.edge)?;
    let v_start = brep.vertices.get(edge.v_start)?;
    let v_end = brep.vertices.get(edge.v_end)?;
    // B1: UV accessed via .uv field instead of .point.x/.point.y
    let fallback_start = v_start.uv;
    let fallback_end = v_end.uv;
    // P2+S4: uv_curve is SmallVec; len >= 2 means polyline
    if trim.uv_curve.len() >= 2 {
        Some((trim.uv_curve[0], trim.uv_curve[trim.uv_curve.len() - 1]))
    } else {
        Some((fallback_start, fallback_end))
    }
}

pub(crate) fn validate_brep_data(brep: &BrepData) -> RgmBrepValidationReport {
    let mut report = RgmBrepValidationReport::default();
    let tol = 1e-7;

    if brep.faces.is_empty() {
        push_issue(&mut report, RgmValidationSeverity::Error, 1001, RgmBrepEntityKind::Face, 0, None);
    }

    for (face_id, face) in brep.faces.iter_enumerated() {
        if face.loops.is_empty() {
            push_issue(
                &mut report,
                RgmValidationSeverity::Error,
                1101,
                RgmBrepEntityKind::Face,
                face_id.raw(),
                None,
            );
        }

        for &loop_id in &face.loops {
            if loop_id.index() >= brep.loops.len() {
                push_issue(
                    &mut report,
                    RgmValidationSeverity::Error,
                    1102,
                    RgmBrepEntityKind::Loop,
                    loop_id.raw(),
                    None,
                );
                continue;
            }
            let loop_data = &brep.loops[loop_id];
            if loop_data.trims.is_empty() {
                push_issue(
                    &mut report,
                    RgmValidationSeverity::Error,
                    1103,
                    RgmBrepEntityKind::Loop,
                    loop_id.raw(),
                    None,
                );
                continue;
            }

            for &trim_id in &loop_data.trims {
                if trim_id.index() >= brep.trims.len() {
                    push_issue(
                        &mut report,
                        RgmValidationSeverity::Error,
                        1201,
                        RgmBrepEntityKind::Trim,
                        trim_id.raw(),
                        None,
                    );
                    continue;
                }
                let trim = &brep.trims[trim_id];
                if trim.face != face_id {
                    push_issue(
                        &mut report,
                        RgmValidationSeverity::Error,
                        1202,
                        RgmBrepEntityKind::Trim,
                        trim_id.raw(),
                        None,
                    );
                }
                if trim.loop_id != loop_id {
                    push_issue(
                        &mut report,
                        RgmValidationSeverity::Error,
                        1203,
                        RgmBrepEntityKind::Trim,
                        trim_id.raw(),
                        None,
                    );
                }
                if trim.edge.index() >= brep.edges.len() {
                    push_issue(
                        &mut report,
                        RgmValidationSeverity::Error,
                        1204,
                        RgmBrepEntityKind::Trim,
                        trim_id.raw(),
                        None,
                    );
                }
            }

            // L2: Pre-collect (end_uv, next_start_uv) pairs before the connectivity check loop,
            // avoiding 2N calls to trim_endpoints for N trims.
            let n = loop_data.trims.len();
            let endpoints: SmallVec<[(RgmUv2, RgmUv2); 8]> = loop_data
                .trims
                .iter()
                .filter_map(|&tid| trim_endpoints(brep, tid.index()))
                .collect();

            if endpoints.len() == n {
                for i in 0..n {
                    let end_uv = endpoints[i].1;
                    let next_start_uv = endpoints[(i + 1) % n].0;
                    if uv_distance(end_uv, next_start_uv) > tol {
                        push_issue(
                            &mut report,
                            RgmValidationSeverity::Warning,
                            1301,
                            RgmBrepEntityKind::Loop,
                            loop_id.raw(),
                            Some(end_uv),
                        );
                    }
                }
            }
        }
    }

    for (edge_id, edge) in brep.edges.iter_enumerated() {
        if edge.v_start.index() >= brep.vertices.len() || edge.v_end.index() >= brep.vertices.len()
        {
            push_issue(
                &mut report,
                RgmValidationSeverity::Error,
                1401,
                RgmBrepEntityKind::Edge,
                edge_id.raw(),
                None,
            );
        }
        if edge.trims.is_empty() {
            push_issue(
                &mut report,
                RgmValidationSeverity::Warning,
                1402,
                RgmBrepEntityKind::Edge,
                edge_id.raw(),
                None,
            );
        }
    }

    for (solid_id, solid) in brep.solids.iter_enumerated() {
        if solid.shells.is_empty() {
            push_issue(
                &mut report,
                RgmValidationSeverity::Error,
                1501,
                RgmBrepEntityKind::Solid,
                solid_id.raw(),
                None,
            );
            continue;
        }
        for &shell_id in &solid.shells {
            if shell_id.index() >= brep.shells.len() {
                push_issue(
                    &mut report,
                    RgmValidationSeverity::Error,
                    1502,
                    RgmBrepEntityKind::Solid,
                    solid_id.raw(),
                    None,
                );
                continue;
            }
            let shell = &brep.shells[shell_id];
            if !shell.closed {
                push_issue(
                    &mut report,
                    RgmValidationSeverity::Warning,
                    1503,
                    RgmBrepEntityKind::Shell,
                    shell_id.raw(),
                    None,
                );
            }
        }
    }

    report
}

pub(crate) fn heal_brep_data(brep: &mut BrepData) -> u32 {
    let mut fixes = 0_u32;

    for edge in brep.edges.iter_mut() {
        if !edge.trims.is_empty() {
            fixes = fixes.saturating_add(1);
        }
        edge.trims.clear();
    }
    for (trim_id, trim) in brep.trims.iter_enumerated() {
        if let Some(edge) = brep.edges.get_mut(trim.edge) {
            edge.trims.push(trim_id);
        }
    }

    for trim in brep.trims.iter_mut() {
        let edge = match brep.edges.get(trim.edge) {
            Some(value) => value,
            None => continue,
        };
        // B1: UV accessed via .uv field
        let start = match brep.vertices.get(edge.v_start) {
            Some(value) => value.uv,
            None => continue,
        };
        let end = match brep.vertices.get(edge.v_end) {
            Some(value) => value.uv,
            None => continue,
        };

        // P2+S4: uv_curve is SmallVec; empty = unspecified
        if trim.uv_curve.is_empty() {
            trim.uv_curve.push(start);
            trim.uv_curve.push(end);
            fixes = fixes.saturating_add(1);
        } else if trim.uv_curve.len() == 1 {
            trim.uv_curve.push(end);
            fixes = fixes.saturating_add(1);
        }
    }

    let tol = 1e-7;
    for loop_data in brep.loops.iter() {
        if loop_data.trims.len() < 2 {
            continue;
        }
        let first_trim_id = loop_data.trims[0];
        let last_trim_id = loop_data.trims[loop_data.trims.len() - 1];
        let Some((first_start, _)) = trim_endpoints(brep, first_trim_id.index()) else {
            continue;
        };
        let Some((_, last_end)) = trim_endpoints(brep, last_trim_id.index()) else {
            continue;
        };

        if uv_distance(last_end, first_start) > tol {
            if let Some(last_trim) = brep.trims.get_mut(last_trim_id) {
                // P2+S4: uv_curve is SmallVec
                if !last_trim.uv_curve.is_empty() {
                    if let Some(last) = last_trim.uv_curve.last_mut() {
                        *last = first_start;
                        fixes = fixes.saturating_add(1);
                    }
                } else {
                    last_trim.uv_curve.push(last_end);
                    last_trim.uv_curve.push(first_start);
                    fixes = fixes.saturating_add(1);
                }
            }
        }
    }

    if fixes > 0 {
        brep.invalidate_topology();
    }
    fixes
}

// L4: Check max_severity directly instead of linear scan.
pub(crate) fn report_has_errors(report: &RgmBrepValidationReport) -> bool {
    report.max_severity == RgmValidationSeverity::Error
}
