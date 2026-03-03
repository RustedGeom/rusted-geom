use crate::RgmPoint3 as Vec3;

use super::error::LandXmlError;
use super::types::{
    AsymmetricParabolaVerticalCurve, CircularVerticalCurve, DesignedVerticalModel,
    LandXmlParseMode, LandXmlWarning, ParabolaVerticalCurve, SampledVerticalModel,
    TangentInterval, VerticalControlCurve, VerticalControlNode, VerticalCurveInterval,
    VerticalModel, VerticalNode,
};

fn add_warning(
    warnings: &mut Vec<LandXmlWarning>,
    code: &str,
    message: String,
    path: Option<String>,
) {
    warnings.push(LandXmlWarning {
        code: code.to_string(),
        message,
        path,
    });
}

fn grade_between(a: &VerticalNode, b: &VerticalNode) -> Result<f64, LandXmlError> {
    let ds = b.station_m - a.station_m;
    if ds.abs() < 1e-12 {
        return Err(LandXmlError::parse(
            "duplicate vertical nodes with equal station cannot define grade",
        ));
    }
    Ok((b.elevation_m - a.elevation_m) / ds)
}

fn curve_interval_s0(curve: &VerticalCurveInterval) -> f64 {
    match curve {
        VerticalCurveInterval::SymmetricParabola(v) => v.s0,
        VerticalCurveInterval::Circular(v) => v.s0,
        VerticalCurveInterval::AsymmetricParabola(v) => v.s_bvc,
    }
}

fn curve_interval_s1(curve: &VerticalCurveInterval) -> f64 {
    match curve {
        VerticalCurveInterval::SymmetricParabola(v) => v.s1,
        VerticalCurveInterval::Circular(v) => v.s1,
        VerticalCurveInterval::AsymmetricParabola(v) => v.s_evc,
    }
}

const STATION_DEDUP_TOL: f64 = 1e-6;

fn dedup_controls(
    controls: &mut Vec<VerticalControlNode>,
    mode: LandXmlParseMode,
    warnings: &mut Vec<LandXmlWarning>,
) -> Result<(), LandXmlError> {
    let mut i = 0;
    while i + 1 < controls.len() {
        let ds = (controls[i + 1].station_m - controls[i].station_m).abs();
        if ds < STATION_DEDUP_TOL {
            match mode {
                LandXmlParseMode::Strict => {
                    return Err(LandXmlError::parse(format!(
                        "duplicate vertical nodes at station ~{:.6} ({})",
                        controls[i].station_m, controls[i].source_path,
                    )));
                }
                LandXmlParseMode::Lenient => {
                    let keep = if matches!(controls[i].curve, VerticalControlCurve::None) {
                        i + 1
                    } else {
                        i
                    };
                    let drop = if keep == i { i + 1 } else { i };
                    add_warning(
                        warnings,
                        "profile_dedup_station",
                        format!(
                            "Dropping duplicate vertical node at station ~{:.6} ({})",
                            controls[drop].station_m, controls[drop].source_path,
                        ),
                        Some(controls[drop].source_path.clone()),
                    );
                    controls.remove(drop);
                    continue;
                }
            }
        }
        i += 1;
    }
    Ok(())
}

pub fn build_designed_model(
    mut controls: Vec<VerticalControlNode>,
    mode: LandXmlParseMode,
    warnings: &mut Vec<LandXmlWarning>,
) -> Result<DesignedVerticalModel, LandXmlError> {
    if controls.len() < 2 {
        return Err(LandXmlError::parse(
            "profile alignment requires at least two PVI-compatible points",
        ));
    }

    controls.sort_by(|a, b| {
        a.station_m
            .partial_cmp(&b.station_m)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    dedup_controls(&mut controls, mode, warnings)?;

    if controls.len() < 2 {
        return Err(LandXmlError::parse(
            "profile alignment requires at least two PVI-compatible points after dedup",
        ));
    }

    let nodes: Vec<VerticalNode> = controls
        .iter()
        .map(|n| VerticalNode {
            station_m: n.station_m,
            elevation_m: n.elevation_m,
        })
        .collect();

    let mut tangents = Vec::new();
    for i in 0..nodes.len() - 1 {
        let a = &nodes[i];
        let b = &nodes[i + 1];
        let g = grade_between(a, b)?;
        tangents.push(TangentInterval {
            s0: a.station_m,
            s1: b.station_m,
            z0: a.elevation_m,
            grade: g,
        });
    }

    let mut curves = Vec::new();
    for i in 0..controls.len() {
        let control = &controls[i];
        let (g0, g1) = if i == 0 || i + 1 >= controls.len() {
            match mode {
                LandXmlParseMode::Strict => {
                    if !matches!(control.curve, VerticalControlCurve::None) {
                        return Err(LandXmlError::parse(format!(
                            "curve at profile endpoint is not supported ({})",
                            control.source_path
                        )));
                    }
                    continue;
                }
                LandXmlParseMode::Lenient => {
                    if !matches!(control.curve, VerticalControlCurve::None) {
                        add_warning(
                            warnings,
                            "profile_curve_endpoint",
                            format!(
                                "Skipping curve at profile endpoint in {}",
                                control.source_path
                            ),
                            Some(control.source_path.clone()),
                        );
                    }
                    continue;
                }
            }
        } else {
            (
                grade_between(&nodes[i - 1], &nodes[i])?,
                grade_between(&nodes[i], &nodes[i + 1])?,
            )
        };

        match control.curve.clone() {
            VerticalControlCurve::None => {}
            VerticalControlCurve::SymmetricParabola { length_m } => {
                if length_m <= 0.0 {
                    match mode {
                        LandXmlParseMode::Strict => {
                            return Err(LandXmlError::parse(format!(
                                "ParaCurve length must be > 0 ({})",
                                control.source_path
                            )));
                        }
                        LandXmlParseMode::Lenient => {
                            add_warning(
                                warnings,
                                "profile_paracurve_length",
                                format!(
                                    "Skipping ParaCurve with non-positive length in {}",
                                    control.source_path
                                ),
                                Some(control.source_path.clone()),
                            );
                            continue;
                        }
                    }
                }
                let s0 = control.station_m - 0.5 * length_m;
                let s1 = control.station_m + 0.5 * length_m;
                let z0 = control.elevation_m - g0 * (control.station_m - s0);
                let a = (g1 - g0) / length_m;

                curves.push(VerticalCurveInterval::SymmetricParabola(
                    ParabolaVerticalCurve { s0, s1, z0, g0, a },
                ));
            }
            VerticalControlCurve::Circular { length_m, radius_m } => {
                if length_m <= 0.0 || !radius_m.is_finite() || radius_m.abs() < 1e-12 {
                    match mode {
                        LandXmlParseMode::Strict => {
                            return Err(LandXmlError::parse(format!(
                                "CircCurve requires positive length and non-zero finite radius ({})",
                                control.source_path
                            )));
                        }
                        LandXmlParseMode::Lenient => {
                            add_warning(
                                warnings,
                                "profile_circcurve_params",
                                format!(
                                    "Skipping CircCurve with invalid parameters in {}",
                                    control.source_path
                                ),
                                Some(control.source_path.clone()),
                            );
                            continue;
                        }
                    }
                }

                let s0 = control.station_m - 0.5 * length_m;
                let s1 = control.station_m + 0.5 * length_m;
                let z0 = control.elevation_m - g0 * (control.station_m - s0);
                let theta0 = g0.atan();
                let mut b = if radius_m > 0.0 {
                    1.0 / radius_m
                } else {
                    -1.0 / radius_m.abs()
                };

                if (g1 - g0).signum() < 0.0 {
                    b = -b.abs();
                } else {
                    b = b.abs();
                }

                let theta_end = theta0 + b * length_m;
                let g1_est = theta_end.tan();
                if (g1_est - g1).abs() > 5e-2 {
                    add_warning(
                        warnings,
                        "profile_circcurve_grade_mismatch",
                        format!(
                            "CircCurve in {} has radius/length not matching adjacent tangents (expected grade {:.6}, got {:.6})",
                            control.source_path, g1, g1_est
                        ),
                        Some(control.source_path.clone()),
                    );
                }

                curves.push(VerticalCurveInterval::Circular(CircularVerticalCurve {
                    s0,
                    s1,
                    z0,
                    theta0_rad: theta0,
                    b_per_m: b,
                    radius_m: Some(radius_m),
                }));
            }
            VerticalControlCurve::AsymmetricParabola { length_in_m, length_out_m } => {
                if length_in_m <= 0.0 || length_out_m <= 0.0 {
                    match mode {
                        LandXmlParseMode::Strict => {
                            return Err(LandXmlError::parse(format!(
                                "UnsymParaCurve lengths must be > 0 ({})",
                                control.source_path
                            )));
                        }
                        LandXmlParseMode::Lenient => {
                            add_warning(
                                warnings,
                                "profile_unsymparacurve_length",
                                format!(
                                    "Skipping UnsymParaCurve with non-positive length in {}",
                                    control.source_path
                                ),
                                Some(control.source_path.clone()),
                            );
                            continue;
                        }
                    }
                }
                let s_pvi = control.station_m;
                let z_pvi = control.elevation_m;
                let s_bvc = s_pvi - length_in_m;
                let s_evc = s_pvi + length_out_m;
                let z_bvc = z_pvi - g0 * length_in_m;
                let g_mid = 2.0 * (z_pvi - z_bvc) / length_in_m - g0;

                curves.push(VerticalCurveInterval::AsymmetricParabola(
                    AsymmetricParabolaVerticalCurve {
                        s_bvc,
                        s_pvi,
                        s_evc,
                        z_bvc,
                        z_pvi,
                        g0,
                        g_mid,
                        g1,
                    },
                ));
            }
        }
    }

    curves.sort_by(|a, b| {
        let a0 = curve_interval_s0(a);
        let b0 = curve_interval_s0(b);
        a0.partial_cmp(&b0).unwrap_or(std::cmp::Ordering::Equal)
    });

    for i in 1..curves.len() {
        let prev_end = curve_interval_s1(&curves[i - 1]);
        let this_start = curve_interval_s0(&curves[i]);
        if this_start < prev_end - 1e-9 {
            let msg = "vertical curves overlap; this profile geometry is invalid".to_string();
            match mode {
                LandXmlParseMode::Strict => return Err(LandXmlError::parse(msg)),
                LandXmlParseMode::Lenient => {
                    add_warning(warnings, "profile_curve_overlap", msg, None);
                }
            }
        }
    }

    Ok(DesignedVerticalModel {
        nodes,
        tangents,
        curves,
    })
}

fn eval_parabola(v: &ParabolaVerticalCurve, s: f64) -> (f64, f64, f64) {
    let d = (s - v.s0).clamp(0.0, v.s1 - v.s0);
    let z = v.z0 + v.g0 * d + 0.5 * v.a * d * d;
    let grade = v.g0 + v.a * d;
    (z, grade, v.a)
}

fn eval_circular(v: &CircularVerticalCurve, s: f64) -> (f64, f64, f64) {
    let d = (s - v.s0).clamp(0.0, v.s1 - v.s0);
    let b = v.b_per_m;
    let theta = v.theta0_rad + b * d;
    let grade = theta.tan();

    let z = if b.abs() < 1e-12 {
        v.z0 + v.theta0_rad.tan() * d
    } else {
        let c0 = v.theta0_rad.cos().abs().max(1e-12);
        let c1 = theta.cos().abs().max(1e-12);
        v.z0 + (c0.ln() - c1.ln()) / b
    };

    let vertical_curvature = b / theta.cos().powi(2).max(1e-12);
    (z, grade, vertical_curvature)
}

fn eval_asymmetric_parabola(v: &AsymmetricParabolaVerticalCurve, s: f64) -> (f64, f64, f64) {
    if s <= v.s_pvi {
        let l_in = v.s_pvi - v.s_bvc;
        let r1 = if l_in.abs() < 1e-12 { 0.0 } else { (v.g_mid - v.g0) / l_in };
        let d = (s - v.s_bvc).clamp(0.0, l_in);
        let z = v.z_bvc + v.g0 * d + 0.5 * r1 * d * d;
        let grade = v.g0 + r1 * d;
        (z, grade, r1)
    } else {
        let l_out = v.s_evc - v.s_pvi;
        let r2 = if l_out.abs() < 1e-12 { 0.0 } else { (v.g1 - v.g_mid) / l_out };
        let d = (s - v.s_pvi).clamp(0.0, l_out);
        let z = v.z_pvi + v.g_mid * d + 0.5 * r2 * d * d;
        let grade = v.g_mid + r2 * d;
        (z, grade, r2)
    }
}

pub fn evaluate_vertical_model(
    model: &VerticalModel,
    station_m: f64,
) -> Result<(f64, f64, f64), LandXmlError> {
    if !station_m.is_finite() {
        return Err(LandXmlError::invalid_input(
            "vertical station must be finite",
        ));
    }

    match model {
        VerticalModel::Designed(designed) => {
            for curve in &designed.curves {
                match curve {
                    VerticalCurveInterval::SymmetricParabola(v)
                        if station_m >= v.s0 - 1e-9 && station_m <= v.s1 + 1e-9 =>
                    {
                        return Ok(eval_parabola(v, station_m));
                    }
                    VerticalCurveInterval::Circular(v)
                        if station_m >= v.s0 - 1e-9 && station_m <= v.s1 + 1e-9 =>
                    {
                        return Ok(eval_circular(v, station_m));
                    }
                    VerticalCurveInterval::AsymmetricParabola(v)
                        if station_m >= v.s_bvc - 1e-9 && station_m <= v.s_evc + 1e-9 =>
                    {
                        return Ok(eval_asymmetric_parabola(v, station_m));
                    }
                    _ => {}
                }
            }

            for tan in &designed.tangents {
                if station_m >= tan.s0 - 1e-9 && station_m <= tan.s1 + 1e-9 {
                    let d = station_m - tan.s0;
                    return Ok((tan.z0 + tan.grade * d, tan.grade, 0.0));
                }
            }

            Err(LandXmlError::out_of_range(format!(
                "vertical station {station_m} is outside profile domain"
            )))
        }
        VerticalModel::Sampled(sampled) => {
            if sampled.samples.is_empty() {
                return Err(LandXmlError::not_found(
                    "sampled profile has no station/elevation points",
                ));
            }

            if sampled.samples.len() == 1 {
                let p = &sampled.samples[0];
                return Ok((p.elevation_m, 0.0, 0.0));
            }

            for i in 0..sampled.samples.len() - 1 {
                let a = &sampled.samples[i];
                let b = &sampled.samples[i + 1];
                if station_m >= a.station_m - 1e-9 && station_m <= b.station_m + 1e-9 {
                    let ds = (b.station_m - a.station_m).max(1e-12);
                    let t = ((station_m - a.station_m) / ds).clamp(0.0, 1.0);
                    let z = a.elevation_m + (b.elevation_m - a.elevation_m) * t;
                    let g = (b.elevation_m - a.elevation_m) / ds;
                    return Ok((z, g, 0.0));
                }
            }

            Err(LandXmlError::out_of_range(format!(
                "vertical station {station_m} is outside sampled profile domain"
            )))
        }
    }
}

pub fn sample_vertical_model(
    model: &VerticalModel,
    start_station_m: f64,
    end_station_m: f64,
    step_m: f64,
) -> Result<Vec<Vec3>, LandXmlError> {
    if !start_station_m.is_finite() || !end_station_m.is_finite() {
        return Err(LandXmlError::invalid_input(
            "sample range bounds must be finite",
        ));
    }
    if !step_m.is_finite() || step_m <= 0.0 {
        return Err(LandXmlError::invalid_input("sample step must be > 0"));
    }

    let mut out = Vec::new();
    let mut s = start_station_m;
    while s <= end_station_m + 1e-9 {
        let (z, _, _) = evaluate_vertical_model(model, s)?;
        out.push(Vec3 { x: s, y: 0.0, z });
        s += step_m;
    }
    if out.is_empty() {
        let (z, _, _) = evaluate_vertical_model(model, start_station_m)?;
        out.push(Vec3 { x: start_station_m, y: 0.0, z });
    }
    Ok(out)
}

pub fn sampled_model_from_pairs(pairs: Vec<(f64, f64)>) -> SampledVerticalModel {
    let mut samples: Vec<VerticalNode> = pairs
        .into_iter()
        .map(|(station_m, elevation_m)| VerticalNode {
            station_m,
            elevation_m,
        })
        .collect();
    samples.sort_by(|a, b| {
        a.station_m
            .partial_cmp(&b.station_m)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    SampledVerticalModel { samples }
}
