use std::collections::{BTreeMap, HashMap};

use roxmltree::{Document, Node};

use crate::RgmPoint3 as Vec3;

use super::error::LandXmlError;
use super::horizontal::{arc_sweep_radians, heading_from_chord, parse_rotation_clockwise};
use super::spiral::evaluate_spiral_local;
use super::types::{
    AlignmentRecord, CircularArcSegment, HorizontalAlignment, HorizontalSegment, LandXmlDocument,
    LandXmlParseMode, LandXmlParseOptions, LandXmlPointOrder, LandXmlUnits, LandXmlUnitsPolicy,
    LandXmlWarning, LineSegment, PlanLinear, PlanLinearKind, ProfileKind, ProfileSeries,
    SpiralSegment, SpiralType, StationEquation, StationIncrementDirection, StationMap, TerrainTin,
    Vec3Ops, VerticalControlCurve, VerticalControlNode, VerticalModel,
};
use super::vertical::{build_designed_model, sampled_model_from_pairs};

fn tag_name<'a, 'input>(node: Node<'a, 'input>) -> &'input str {
    node.tag_name().name()
}

fn parse_f64(raw: &str) -> Result<f64, LandXmlError> {
    let s = raw.trim();
    if s.eq_ignore_ascii_case("inf") {
        return Ok(f64::INFINITY);
    }
    if s.eq_ignore_ascii_case("-inf") {
        return Ok(f64::NEG_INFINITY);
    }
    s.parse::<f64>()
        .map_err(|_| LandXmlError::parse(format!("invalid number '{s}'")))
}

fn parse_f64_opt(raw: Option<&str>) -> Result<Option<f64>, LandXmlError> {
    match raw {
        Some(v) => Ok(Some(parse_f64(v)?)),
        None => Ok(None),
    }
}

fn parse_numbers(text: &str) -> Result<Vec<f64>, LandXmlError> {
    text.split_whitespace()
        .map(parse_f64)
        .collect::<Result<Vec<_>, _>>()
}

fn parse_ints(text: &str) -> Result<Vec<u32>, LandXmlError> {
    let mut out = Vec::new();
    for token in text.split_whitespace() {
        let v = token
            .trim()
            .parse::<u32>()
            .map_err(|_| LandXmlError::parse(format!("invalid integer '{token}'")))?;
        out.push(v);
    }
    Ok(out)
}

fn parse_point_xyz(text: &str, order: LandXmlPointOrder) -> Result<Vec3, LandXmlError> {
    let nums = parse_numbers(text)?;
    if nums.len() < 2 {
        return Err(LandXmlError::parse(
            "point text must contain at least two values",
        ));
    }

    let (x, y, z) = match order {
        LandXmlPointOrder::Nez => {
            let n = nums[0];
            let e = nums[1];
            let z = nums.get(2).copied().unwrap_or(0.0);
            (e, n, z)
        }
        LandXmlPointOrder::Enz => {
            let e = nums[0];
            let n = nums[1];
            let z = nums.get(2).copied().unwrap_or(0.0);
            (e, n, z)
        }
        LandXmlPointOrder::Ezn => {
            let e = nums[0];
            let z = nums.get(1).copied().unwrap_or(0.0);
            let n = nums.get(2).copied().unwrap_or(0.0);
            (e, n, z)
        }
    };

    Ok(Vec3 { x, y, z })
}

fn parse_station_elevation_pair(text: &str) -> Result<(f64, f64), LandXmlError> {
    let nums = parse_numbers(text)?;
    if nums.len() < 2 {
        return Err(LandXmlError::parse(
            "station/elevation text requires two values",
        ));
    }
    Ok((nums[0], nums[1]))
}

fn parse_station_elevation_pairs(text: &str) -> Result<Vec<(f64, f64)>, LandXmlError> {
    let nums = parse_numbers(text)?;
    if nums.len() < 2 {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 1 < nums.len() {
        out.push((nums[i], nums[i + 1]));
        i += 2;
    }
    Ok(out)
}

fn parse_clockwise_increment(raw: Option<&str>) -> StationIncrementDirection {
    let Some(v) = raw else {
        return StationIncrementDirection::Increasing;
    };
    if v.trim().eq_ignore_ascii_case("decreasing") {
        StationIncrementDirection::Decreasing
    } else {
        StationIncrementDirection::Increasing
    }
}

fn parse_angle_to_rad(
    raw: Option<&str>,
    options: &LandXmlParseOptions,
    unit_hint: Option<&str>,
) -> Result<Option<f64>, LandXmlError> {
    let Some(v) = raw else {
        return Ok(None);
    };
    let mut x = parse_f64(v)?;

    let explicit_unit = options
        .angular_unit_override
        .as_deref()
        .map(|s| s.to_ascii_lowercase());

    let hinted_unit = unit_hint.map(|s| s.to_ascii_lowercase());
    let use_unit = explicit_unit.as_deref().or(hinted_unit.as_deref());

    match use_unit {
        Some("deg") | Some("degree") | Some("degrees") => {
            x = x.to_radians();
        }
        Some(u) if u.contains("deg") || u.contains("degree") => {
            x = x.to_radians();
        }
        Some("rad") | Some("radian") | Some("radians") => {}
        Some(u) if u.contains("rad") => {}
        _ => {
            if x.abs() > std::f64::consts::TAU * 1.5 {
                x = x.to_radians();
            }
        }
    }

    Ok(Some(x))
}

fn linear_unit_to_meters(unit: &str) -> Option<f64> {
    match unit {
        "millimeter" => Some(1e-3),
        "centimeter" => Some(1e-2),
        "meter" => Some(1.0),
        "kilometer" => Some(1_000.0),
        "inch" => Some(0.0254),
        "foot" => Some(0.3048),
        "USSurveyFoot" => Some(1200.0 / 3937.0),
        "mile" => Some(1609.344),
        _ => None,
    }
}

fn read_units(root: Node<'_, '_>, options: &LandXmlParseOptions) -> LandXmlUnits {
    let mut source_linear = "meter".to_string();
    let mut source_angular: Option<String> = None;
    let mut factor = 1.0;

    for node in root.descendants().filter(|n| n.is_element()) {
        let name = tag_name(node);
        if name == "Metric" || name == "Imperial" {
            if let Some(linear) = node.attribute("linearUnit") {
                source_linear = linear.to_string();
                factor = linear_unit_to_meters(linear).unwrap_or(1.0);
            }
            if let Some(angular) = node.attribute("angularUnit") {
                source_angular = Some(angular.to_string());
            }
            if let Some(direction) = node.attribute("directionUnit") {
                source_angular = Some(direction.to_string());
            }
            break;
        }
    }

    LandXmlUnits {
        source_linear_unit: source_linear,
        source_angular_unit: source_angular,
        linear_to_meters: factor,
        normalized_to_meters: options.units_policy == LandXmlUnitsPolicy::NormalizeToMeters,
    }
}

fn text_of_child(node: Node<'_, '_>, child_name: &str) -> Option<String> {
    node.children()
        .find(|n| n.is_element() && tag_name(*n) == child_name)
        .and_then(|n| n.text())
        .map(|s| s.trim().to_string())
}

fn point_of_child(
    node: Node<'_, '_>,
    child_name: &str,
    point_order: LandXmlPointOrder,
    scale: f64,
) -> Result<Vec3, LandXmlError> {
    let text = text_of_child(node, child_name)
        .ok_or_else(|| LandXmlError::parse(format!("missing <{child_name}> point")))?;
    let p = parse_point_xyz(&text, point_order)?;
    Ok(Vec3 { x: p.x * scale, y: p.y * scale, z: p.z * scale })
}

fn point_of_child_opt(
    node: Node<'_, '_>,
    child_name: &str,
    point_order: LandXmlPointOrder,
    scale: f64,
) -> Result<Option<Vec3>, LandXmlError> {
    let Some(text) = text_of_child(node, child_name) else {
        return Ok(None);
    };
    let p = parse_point_xyz(&text, point_order)?;
    Ok(Some(Vec3 { x: p.x * scale, y: p.y * scale, z: p.z * scale }))
}

fn rotate_2d(x: f64, y: f64, angle: f64) -> (f64, f64) {
    let c = angle.cos();
    let s = angle.sin();
    (x * c - y * s, x * s + y * c)
}

fn segment_end_state(segment: &HorizontalSegment) -> (Vec3, f64) {
    match segment {
        HorizontalSegment::Line(s) => (s.end, s.start_heading_rad),
        HorizontalSegment::CircularArc(s) => {
            let heading = if s.clockwise {
                s.start_heading_rad - s.sweep_rad
            } else {
                s.start_heading_rad + s.sweep_rad
            };
            (s.end, heading)
        }
        HorizontalSegment::Spiral(s) => {
            let local = evaluate_spiral_local(s, s.length_m);
            let (dx, dy) = rotate_2d(local.x_local, local.y_local, s.start_heading_rad);
            let p = Vec3 { x: s.start.x + dx, y: s.start.y + dy, z: s.start.z };
            (p, local.heading_rad)
        }
    }
}

fn inv_radius(radius_opt: Option<f64>, clockwise: bool) -> f64 {
    let Some(radius) = radius_opt else {
        return 0.0;
    };
    if !radius.is_finite() {
        return 0.0;
    }
    if radius.abs() < 1e-12 {
        return 0.0;
    }
    let mut k = 1.0 / radius;
    if clockwise {
        k = -k.abs();
    } else {
        k = k.abs();
    }
    k
}

fn solve_spiral_start_heading(
    spi_type: SpiralType,
    k0_per_m: f64,
    k1_per_m: f64,
    length_m: f64,
    start: Vec3,
    end: Vec3,
    clockwise: bool,
) -> Option<f64> {
    if !length_m.is_finite() || length_m <= 1e-9 {
        return None;
    }
    let probe = SpiralSegment {
        spi_type,
        start: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        end: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        start_station_m: 0.0,
        length_m,
        radius_start_m: None,
        radius_end_m: None,
        k0_per_m,
        k1_per_m,
        start_heading_rad: 0.0,
        clockwise,
    };
    let local = evaluate_spiral_local(&probe, length_m);
    let local_norm = (local.x_local * local.x_local + local.y_local * local.y_local).sqrt();
    if local_norm <= 1e-9 || !local_norm.is_finite() {
        return None;
    }

    let chord = end.sub(start);
    let chord_norm = (chord.x * chord.x + chord.y * chord.y).sqrt();
    if chord_norm <= 1e-9 || !chord_norm.is_finite() {
        return None;
    }

    let local_angle = local.y_local.atan2(local.x_local);
    let chord_angle = chord.y.atan2(chord.x);
    Some(chord_angle - local_angle)
}

fn parse_horizontal_alignment(
    alignment_node: Node<'_, '_>,
    options: &LandXmlParseOptions,
    angular_unit_hint: Option<&str>,
    scale: f64,
    sta_start_m: f64,
) -> Result<HorizontalAlignment, LandXmlError> {
    let coord_geom = alignment_node
        .children()
        .find(|n| n.is_element() && tag_name(*n) == "CoordGeom");
    let Some(coord_geom) = coord_geom else {
        return Ok(HorizontalAlignment {
            segments: vec![],
            total_length_m: 0.0,
        });
    };

    let mut segments = Vec::new();
    let mut running_station = sta_start_m;
    let mut previous_end_state: Option<(Vec3, f64)> = None;

    for seg in coord_geom.children().filter(|n| n.is_element()) {
        match tag_name(seg) {
            "Line" => {
                let raw_start = point_of_child(seg, "Start", options.point_order, scale)?;
                let mut start = raw_start;
                if let Some((prev_end, _)) = previous_end_state {
                    start = prev_end;
                }

                let mut end = if let Some(raw_end) =
                    point_of_child_opt(seg, "End", options.point_order, scale)?
                {
                    let delta = start.sub(raw_start);
                    raw_end.add(delta)
                } else {
                    start
                };

                let len = parse_f64_opt(seg.attribute("length"))?
                    .map(|v| v * scale)
                    .unwrap_or_else(|| start.distance(end));
                let s0 = parse_f64_opt(seg.attribute("staStart"))?
                    .map(|v| v * scale)
                    .unwrap_or(running_station);
                let heading = if let Some(dir) =
                    parse_angle_to_rad(seg.attribute("dir"), options, angular_unit_hint)?
                {
                    dir
                } else if start.distance(end) > 1e-9 {
                    heading_from_chord(start, end)
                } else if let Some((_, prev_heading)) = previous_end_state {
                    prev_heading
                } else {
                    0.0
                };
                if start.distance(end) <= 1e-9 && len > 1e-9 {
                    end = Vec3 {
                        x: start.x + len * heading.cos(),
                        y: start.y + len * heading.sin(),
                        z: start.z,
                    };
                }

                segments.push(HorizontalSegment::Line(LineSegment {
                    start,
                    end,
                    start_station_m: s0,
                    length_m: len,
                    start_heading_rad: heading,
                }));
                running_station = s0 + len;
                previous_end_state = segments.last().map(segment_end_state);
            }
            "Curve" => {
                let raw_start = point_of_child(seg, "Start", options.point_order, scale)?;
                let mut start = raw_start;
                if let Some((prev_end, _)) = previous_end_state {
                    start = prev_end;
                }
                let delta = start.sub(raw_start);
                let end = point_of_child(seg, "End", options.point_order, scale)?.add(delta);
                let center = point_of_child(seg, "Center", options.point_order, scale)?.add(delta);

                let radius = start.distance(center);
                let start_angle = (start.y - center.y).atan2(start.x - center.x);
                let end_angle = (end.y - center.y).atan2(end.x - center.x);
                let clockwise = parse_rotation_clockwise(seg.attribute("rot"));
                let sweep = arc_sweep_radians(start_angle, end_angle, clockwise);
                let len = parse_f64_opt(seg.attribute("length"))?
                    .map(|v| v * scale)
                    .unwrap_or(radius * sweep);
                let s0 = parse_f64_opt(seg.attribute("staStart"))?
                    .map(|v| v * scale)
                    .unwrap_or(running_station);
                let heading = if clockwise {
                    start_angle - std::f64::consts::FRAC_PI_2
                } else {
                    start_angle + std::f64::consts::FRAC_PI_2
                };

                segments.push(HorizontalSegment::CircularArc(CircularArcSegment {
                    start,
                    end,
                    center,
                    start_station_m: s0,
                    length_m: len,
                    radius_m: radius,
                    start_angle_rad: start_angle,
                    sweep_rad: sweep,
                    clockwise,
                    start_heading_rad: heading,
                }));
                running_station = s0 + len;
                previous_end_state = segments.last().map(segment_end_state);
            }
            "Spiral" => {
                let raw_start = point_of_child(seg, "Start", options.point_order, scale)?;
                let mut start = raw_start;
                if let Some((prev_end, _)) = previous_end_state {
                    start = prev_end;
                }
                let delta = start.sub(raw_start);
                let end = point_of_child(seg, "End", options.point_order, scale)?.add(delta);
                let spi_type_raw = seg
                    .attribute("spiType")
                    .ok_or_else(|| LandXmlError::parse("Spiral missing spiType"))?;
                let spi_type = SpiralType::parse(spi_type_raw).ok_or_else(|| {
                    LandXmlError::parse(format!("unsupported spiral spiType '{spi_type_raw}'"))
                })?;

                let length_m = parse_f64_opt(seg.attribute("length"))?
                    .ok_or_else(|| LandXmlError::parse("Spiral missing length"))?
                    * scale;
                let radius_start = parse_f64_opt(seg.attribute("radiusStart"))?;
                let radius_end = parse_f64_opt(seg.attribute("radiusEnd"))?;
                let clockwise = parse_rotation_clockwise(seg.attribute("rot"));
                let s0 = parse_f64_opt(seg.attribute("staStart"))?
                    .map(|v| v * scale)
                    .unwrap_or(running_station);
                let k0 = inv_radius(radius_start.map(|v| v * scale), clockwise);
                let k1 = inv_radius(radius_end.map(|v| v * scale), clockwise);
                let explicit_heading =
                    parse_angle_to_rad(seg.attribute("dirStart"), options, angular_unit_hint)?.or(
                        parse_angle_to_rad(seg.attribute("dir"), options, angular_unit_hint)?,
                    );
                let solved_heading =
                    solve_spiral_start_heading(spi_type, k0, k1, length_m, start, end, clockwise);
                let start_heading = explicit_heading
                    .or(solved_heading)
                    .or(previous_end_state.map(|(_, heading)| heading))
                    .unwrap_or_else(|| heading_from_chord(start, end));

                segments.push(HorizontalSegment::Spiral(SpiralSegment {
                    spi_type,
                    start,
                    end,
                    start_station_m: s0,
                    length_m,
                    radius_start_m: radius_start.filter(|v| v.is_finite()).map(|v| v * scale),
                    radius_end_m: radius_end.filter(|v| v.is_finite()).map(|v| v * scale),
                    k0_per_m: k0,
                    k1_per_m: k1,
                    start_heading_rad: start_heading,
                    clockwise,
                }));
                running_station = s0 + length_m;
                previous_end_state = segments.last().map(segment_end_state);
            }
            _ => {}
        }
    }

    let total = if running_station > sta_start_m {
        running_station - sta_start_m
    } else {
        0.0
    };

    Ok(HorizontalAlignment {
        segments,
        total_length_m: total,
    })
}

fn parse_station_map(
    alignment_node: Node<'_, '_>,
    sta_start_m: f64,
    length_m: f64,
    scale: f64,
) -> Result<StationMap, LandXmlError> {
    let mut equations = Vec::new();
    for node in alignment_node
        .children()
        .filter(|n| n.is_element() && tag_name(*n) == "StaEquation")
    {
        let internal = parse_f64_opt(node.attribute("staInternal"))?
            .ok_or_else(|| LandXmlError::parse("StaEquation missing staInternal"))?
            * scale;
        let ahead = parse_f64_opt(node.attribute("staAhead"))?
            .ok_or_else(|| LandXmlError::parse("StaEquation missing staAhead"))?
            * scale;
        let back = parse_f64_opt(node.attribute("staBack"))?.map(|v| v * scale);
        equations.push(StationEquation {
            sta_internal_m: internal,
            sta_ahead_m: ahead,
            sta_back_m: back,
            increment: parse_clockwise_increment(node.attribute("staIncrement")),
        });
    }

    equations.sort_by(|a, b| {
        a.sta_internal_m
            .partial_cmp(&b.sta_internal_m)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(StationMap {
        sta_start_m,
        length_m,
        equations,
    })
}

fn push_warning(
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

const STATION_EPS_M: f64 = 1e-4;

fn profile_bounds_from_controls(controls: &[VerticalControlNode]) -> Option<(f64, f64)> {
    let first = controls.first()?;
    let mut min_station = first.station_m;
    let mut max_station = first.station_m;
    for control in controls.iter().skip(1) {
        min_station = min_station.min(control.station_m);
        max_station = max_station.max(control.station_m);
    }
    Some((min_station, max_station))
}

fn profile_bounds_from_pairs(pairs: &[(f64, f64)]) -> Option<(f64, f64)> {
    let first = pairs.first()?;
    let mut min_station = first.0;
    let mut max_station = first.0;
    for (station, _) in pairs.iter().skip(1) {
        min_station = min_station.min(*station);
        max_station = max_station.max(*station);
    }
    Some((min_station, max_station))
}

fn validate_profile_station_bounds(
    alignment_name: &str,
    profile_name: &str,
    source_name: &str,
    profile_start_m: f64,
    profile_end_m: f64,
    alignment_start_m: f64,
    alignment_end_m: f64,
) -> Result<(), LandXmlError> {
    if profile_start_m + STATION_EPS_M < alignment_start_m {
        return Err(LandXmlError::parse(format!(
            "Profile '{profile_name}' ({source_name}) for alignment '{alignment_name}' starts at station {profile_start_m:.6}, before alignment start {alignment_start_m:.6}"
        )));
    }
    if profile_end_m - STATION_EPS_M > alignment_end_m {
        return Err(LandXmlError::parse(format!(
            "Profile '{profile_name}' ({source_name}) for alignment '{alignment_name}' ends at station {profile_end_m:.6}, after alignment end {alignment_end_m:.6}"
        )));
    }
    Ok(())
}

fn parse_profile_series(
    alignment_name: &str,
    alignment_node: Node<'_, '_>,
    alignment_start_m: f64,
    alignment_end_m: f64,
    options: &LandXmlParseOptions,
    scale: f64,
    warnings: &mut Vec<LandXmlWarning>,
) -> Result<Vec<ProfileSeries>, LandXmlError> {
    let mut out = Vec::new();

    for (profile_idx, profile_node) in alignment_node
        .children()
        .filter(|n| n.is_element() && tag_name(*n) == "Profile")
        .enumerate()
    {
        let profile_name = profile_node
            .attribute("name")
            .map(str::to_string)
            .unwrap_or_else(|| format!("Profile{}", profile_idx + 1));

        let mut align_count = 0usize;
        let mut surf_count = 0usize;

        for child in profile_node.children().filter(|n| n.is_element()) {
            match tag_name(child) {
                "ProfAlign" => {
                    align_count += 1;
                    let profalign_name = child
                        .attribute("name")
                        .map(str::to_string)
                        .unwrap_or_else(|| format!("ProfAlign{align_count}"));

                    let mut controls = Vec::new();
                    for node in child.children().filter(|n| n.is_element()) {
                        let nname = tag_name(node);
                        match nname {
                            "PVI" | "ParaCurve" | "CircCurve" | "UnsymParaCurve" => {
                                let text = node.text().unwrap_or("").trim();
                                if text.is_empty() {
                                    continue;
                                }
                                let (station_raw, elev_raw) = parse_station_elevation_pair(text)?;
                                let station_m = station_raw * scale;
                                let elevation_m = elev_raw * scale;

                                let curve = match nname {
                                    "PVI" => VerticalControlCurve::None,
                                    "ParaCurve" => {
                                        let length =
                                            parse_f64_opt(node.attribute("length"))?.ok_or_else(
                                                || LandXmlError::parse("ParaCurve missing length"),
                                            )? * scale;
                                        VerticalControlCurve::SymmetricParabola { length_m: length }
                                    }
                                    "CircCurve" => {
                                        let length =
                                            parse_f64_opt(node.attribute("length"))?.ok_or_else(
                                                || LandXmlError::parse("CircCurve missing length"),
                                            )? * scale;
                                        let radius =
                                            parse_f64_opt(node.attribute("radius"))?.ok_or_else(
                                                || LandXmlError::parse("CircCurve missing radius"),
                                            )? * scale;
                                        VerticalControlCurve::Circular {
                                            length_m: length,
                                            radius_m: radius,
                                        }
                                    }
                                    "UnsymParaCurve" => {
                                        let length_in = parse_f64_opt(node.attribute("lengthIn"))?
                                            .ok_or_else(|| {
                                                LandXmlError::parse(
                                                    "UnsymParaCurve missing lengthIn",
                                                )
                                            })?
                                            * scale;
                                        let length_out =
                                            parse_f64_opt(node.attribute("lengthOut"))?
                                                .ok_or_else(|| {
                                                    LandXmlError::parse(
                                                        "UnsymParaCurve missing lengthOut",
                                                    )
                                                })?
                                                * scale;
                                        if options.mode == LandXmlParseMode::Lenient {
                                            push_warning(
                                                warnings,
                                                "unsym_paracurve",
                                                format!(
                                                    "Parsed UnsymParaCurve (lengthIn={length_in}, lengthOut={length_out})"
                                                ),
                                                None,
                                            );
                                        }
                                        VerticalControlCurve::AsymmetricParabola {
                                            length_in_m: length_in,
                                            length_out_m: length_out,
                                        }
                                    }
                                    _ => VerticalControlCurve::None,
                                };

                                controls.push(VerticalControlNode {
                                    station_m,
                                    elevation_m,
                                    curve,
                                    source_path: format!(
                                        "Alignment[{alignment_name}]/Profile[{profile_name}]/ProfAlign[{profalign_name}]/{nname}"
                                    ),
                                });
                            }
                            _ => {}
                        }
                    }

                    if controls.len() >= 2 {
                        let (profile_start_m, profile_end_m) =
                            profile_bounds_from_controls(&controls).ok_or_else(|| {
                                LandXmlError::parse("profile alignment has no station samples")
                            })?;
                        if let Err(err) = validate_profile_station_bounds(
                            alignment_name,
                            &profile_name,
                            &profalign_name,
                            profile_start_m,
                            profile_end_m,
                            alignment_start_m,
                            alignment_end_m,
                        ) {
                            if options.mode == LandXmlParseMode::Lenient {
                                push_warning(
                                    warnings,
                                    "profile_out_of_alignment_range",
                                    err.to_string(),
                                    Some(format!(
                                        "Alignment[{alignment_name}]/Profile[{profile_name}]/ProfAlign[{profalign_name}]"
                                    )),
                                );
                                continue;
                            }
                            return Err(err);
                        }

                        let normalized_controls = controls
                            .into_iter()
                            .map(|mut c| {
                                c.station_m -= alignment_start_m;
                                c
                            })
                            .collect::<Vec<_>>();

                        let designed =
                            build_designed_model(normalized_controls, options.mode, warnings)?;
                        let sampled_profile = designed
                            .nodes
                            .iter()
                            .map(|n| Vec3 {
                                x: n.station_m + alignment_start_m,
                                y: 0.0,
                                z: n.elevation_m,
                            })
                            .collect::<Vec<_>>();
                        out.push(ProfileSeries {
                            id: format!(
                                "{alignment_name}::{profile_name}::{profalign_name}::ProfAlign{align_count}"
                            ),
                            kind: ProfileKind::ProfAlign,
                            station_start_m: profile_start_m,
                            station_end_m: profile_end_m,
                            vertical_model: VerticalModel::Designed(designed),
                            sampled_profile,
                        });
                    } else if options.mode == LandXmlParseMode::Lenient {
                        push_warning(
                            warnings,
                            "empty_profalign",
                            format!(
                                "Skipped ProfAlign '{profalign_name}' with insufficient vertical nodes"
                            ),
                            None,
                        );
                    }
                }
                "ProfSurf" => {
                    surf_count += 1;
                    let profsurf_name = child
                        .attribute("name")
                        .map(str::to_string)
                        .unwrap_or_else(|| format!("ProfSurf{surf_count}"));

                    let pntlist = child
                        .descendants()
                        .find(|n| n.is_element() && tag_name(*n) == "PntList2D")
                        .and_then(|n| n.text())
                        .unwrap_or("");

                    let mut pairs = parse_station_elevation_pairs(pntlist)?;
                    for p in &mut pairs {
                        p.0 *= scale;
                        p.1 *= scale;
                    }
                    if pairs.len() >= 2 {
                        let (profile_start_m, profile_end_m) = profile_bounds_from_pairs(&pairs)
                            .ok_or_else(|| {
                                LandXmlError::parse("surface profile has no station samples")
                            })?;
                        if let Err(err) = validate_profile_station_bounds(
                            alignment_name,
                            &profile_name,
                            &profsurf_name,
                            profile_start_m,
                            profile_end_m,
                            alignment_start_m,
                            alignment_end_m,
                        ) {
                            if options.mode == LandXmlParseMode::Lenient {
                                push_warning(
                                    warnings,
                                    "profile_out_of_alignment_range",
                                    err.to_string(),
                                    Some(format!(
                                        "Alignment[{alignment_name}]/Profile[{profile_name}]/ProfSurf[{profsurf_name}]"
                                    )),
                                );
                                continue;
                            }
                            return Err(err);
                        }

                        let normalized_pairs = pairs
                            .iter()
                            .map(|(station, elevation)| (station - alignment_start_m, *elevation))
                            .collect::<Vec<_>>();
                        let sampled_profile = pairs
                            .iter()
                            .map(|(s, z)| Vec3 { x: *s, y: 0.0, z: *z })
                            .collect::<Vec<_>>();
                        out.push(ProfileSeries {
                            id: format!(
                                "{alignment_name}::{profile_name}::{profsurf_name}::ProfSurf{surf_count}"
                            ),
                            kind: ProfileKind::ProfSurf,
                            station_start_m: profile_start_m,
                            station_end_m: profile_end_m,
                            vertical_model: VerticalModel::Sampled(sampled_model_from_pairs(
                                normalized_pairs,
                            )),
                            sampled_profile,
                        });
                    } else if options.mode == LandXmlParseMode::Lenient {
                        push_warning(
                            warnings,
                            "empty_profsurf",
                            format!(
                                "Skipped ProfSurf '{profsurf_name}' with insufficient PntList2D pairs"
                            ),
                            None,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    Ok(out)
}

fn parse_alignments(
    root: Node<'_, '_>,
    options: &LandXmlParseOptions,
    angular_unit_hint: Option<&str>,
    scale: f64,
    warnings: &mut Vec<LandXmlWarning>,
) -> Result<Vec<AlignmentRecord>, LandXmlError> {
    let mut alignments = Vec::new();

    for alignment_node in root
        .descendants()
        .filter(|n| n.is_element() && tag_name(*n) == "Alignment")
    {
        let name = alignment_node
            .attribute("name")
            .map(str::to_string)
            .unwrap_or_else(|| format!("Alignment{}", alignments.len() + 1));
        let sta_start_m =
            parse_f64_opt(alignment_node.attribute("staStart"))?.unwrap_or(0.0) * scale;
        let length_m = parse_f64_opt(alignment_node.attribute("length"))?.unwrap_or(0.0) * scale;

        let horizontal = parse_horizontal_alignment(
            alignment_node,
            options,
            angular_unit_hint,
            scale,
            sta_start_m,
        )?;
        let station_map = parse_station_map(
            alignment_node,
            sta_start_m,
            length_m.max(horizontal.total_length_m),
            scale,
        )?;
        let alignment_end_m = sta_start_m + length_m.max(horizontal.total_length_m);
        let profiles = parse_profile_series(
            &name,
            alignment_node,
            sta_start_m,
            alignment_end_m,
            options,
            scale,
            warnings,
        )?;

        alignments.push(AlignmentRecord {
            name,
            sta_start_m,
            length_m: length_m.max(horizontal.total_length_m),
            station_map,
            horizontal,
            profiles,
        });
    }

    Ok(alignments)
}

fn parse_surfaces(
    root: Node<'_, '_>,
    options: &LandXmlParseOptions,
    scale: f64,
    warnings: &mut Vec<LandXmlWarning>,
) -> Result<Vec<TerrainTin>, LandXmlError> {
    let mut terrains = Vec::new();

    for surface_node in root
        .descendants()
        .filter(|n| n.is_element() && tag_name(*n) == "Surface")
    {
        let surface_name = surface_node
            .attribute("name")
            .map(str::to_string)
            .unwrap_or_else(|| format!("Surface{}", terrains.len() + 1));

        for (def_idx, def) in surface_node
            .children()
            .filter(|n| n.is_element() && tag_name(*n) == "Definition")
            .enumerate()
        {
            let surf_type = def.attribute("surfType").unwrap_or("");
            if !surf_type.eq_ignore_ascii_case("TIN") {
                if options.mode == LandXmlParseMode::Lenient {
                    push_warning(
                        warnings,
                        "surface_non_tin",
                        format!(
                            "Skipping non-TIN surface definition '{surf_type}' in surface '{surface_name}'"
                        ),
                        None,
                    );
                    continue;
                }
                return Err(LandXmlError::parse(format!(
                    "Unsupported surface Definition surfType='{surf_type}', only TIN is supported"
                )));
            }

            let pnts = def
                .children()
                .find(|n| n.is_element() && tag_name(*n) == "Pnts")
                .ok_or_else(|| LandXmlError::parse("TIN Definition missing Pnts"))?;

            let mut points = BTreeMap::<u32, Vec3>::new();
            for p in pnts
                .children()
                .filter(|n| n.is_element() && tag_name(*n) == "P")
            {
                let id = p
                    .attribute("id")
                    .ok_or_else(|| LandXmlError::parse("surface point missing id attribute"))?
                    .parse::<u32>()
                    .map_err(|_| LandXmlError::parse("invalid surface point id"))?;
                let text = p
                    .text()
                    .ok_or_else(|| LandXmlError::parse("surface point missing coordinate text"))?;
                let point = parse_point_xyz(text, options.point_order)?;
                points.insert(
                    id,
                    Vec3 { x: point.x * scale, y: point.y * scale, z: point.z * scale },
                );
            }

            if points.len() < 3 {
                if options.mode == LandXmlParseMode::Lenient {
                    push_warning(
                        warnings,
                        "tin_few_points",
                        format!("Skipping surface '{surface_name}' TIN with fewer than 3 points"),
                        None,
                    );
                    continue;
                }
                return Err(LandXmlError::parse(
                    "TIN Definition requires at least three points",
                ));
            }

            let mut vertices = Vec::with_capacity(points.len());
            let mut index_map = HashMap::<u32, u32>::new();
            for (i, (id, point)) in points.iter().enumerate() {
                vertices.push(*point);
                index_map.insert(*id, i as u32);
            }

            let mut triangles = Vec::new();
            let faces_nodes = def
                .children()
                .filter(|n| n.is_element() && tag_name(*n) == "Faces");

            for faces in faces_nodes {
                for f in faces
                    .children()
                    .filter(|n| n.is_element() && tag_name(*n) == "F")
                {
                    let ids = parse_ints(f.text().unwrap_or(""))?;
                    match ids.len() {
                        3 => {
                            for id in ids {
                                let idx = *index_map.get(&id).ok_or_else(|| {
                                    LandXmlError::parse(format!(
                                        "TIN face references unknown point id {id}"
                                    ))
                                })?;
                                triangles.push(idx);
                            }
                        }
                        4 => {
                            let i0 = *index_map.get(&ids[0]).ok_or_else(|| {
                                LandXmlError::parse("TIN quad references unknown point id")
                            })?;
                            let i1 = *index_map.get(&ids[1]).ok_or_else(|| {
                                LandXmlError::parse("TIN quad references unknown point id")
                            })?;
                            let i2 = *index_map.get(&ids[2]).ok_or_else(|| {
                                LandXmlError::parse("TIN quad references unknown point id")
                            })?;
                            let i3 = *index_map.get(&ids[3]).ok_or_else(|| {
                                LandXmlError::parse("TIN quad references unknown point id")
                            })?;
                            triangles.extend_from_slice(&[i0, i1, i2, i0, i2, i3]);
                        }
                        _ => {
                            if options.mode == LandXmlParseMode::Strict {
                                return Err(LandXmlError::parse(
                                    "TIN face must contain 3 or 4 point references",
                                ));
                            }
                            push_warning(
                                warnings,
                                "tin_face_arity",
                                format!(
                                    "Skipping TIN face with {} vertices in surface '{}'",
                                    ids.len(),
                                    surface_name
                                ),
                                None,
                            );
                        }
                    }
                }
            }

            terrains.push(TerrainTin {
                name: format!("{}::Definition{}", surface_name, def_idx + 1),
                vertices_m: vertices,
                triangles,
            });
        }
    }

    Ok(terrains)
}

fn parse_nez_point_list(
    text: &str,
    stride: usize,
    order: LandXmlPointOrder,
    scale: f64,
) -> Vec<Vec3> {
    let nums: Vec<f64> = text
        .split_whitespace()
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i + stride.max(2) <= nums.len() {
        let (x, y, z) = match order {
            LandXmlPointOrder::Nez => {
                let n = nums[i];
                let e = nums[i + 1];
                let z = if stride >= 3 { nums[i + 2] } else { 0.0 };
                (e, n, z)
            }
            LandXmlPointOrder::Enz => {
                let e = nums[i];
                let n = nums[i + 1];
                let z = if stride >= 3 { nums[i + 2] } else { 0.0 };
                (e, n, z)
            }
            LandXmlPointOrder::Ezn => {
                let e = nums[i];
                let z = if stride >= 3 { nums[i + 1] } else { 0.0 };
                let n = if stride >= 3 { nums[i + 2] } else { nums[i + 1] };
                (e, n, z)
            }
        };
        out.push(Vec3 {
            x: x * scale,
            y: y * scale,
            z: z * scale,
        });
        i += stride;
    }
    out
}

fn sample_arc_points(start: Vec3, center: Vec3, end: Vec3, n: usize) -> Vec<Vec3> {
    let r = ((start.x - center.x).powi(2) + (start.y - center.y).powi(2)).sqrt();
    let a0 = (start.y - center.y).atan2(start.x - center.x);
    let a1 = (end.y - center.y).atan2(end.x - center.x);
    let mut sweep = a1 - a0;
    if sweep.abs() < 1e-9 {
        sweep = std::f64::consts::TAU;
    } else if sweep > std::f64::consts::PI {
        sweep -= std::f64::consts::TAU;
    } else if sweep < -std::f64::consts::PI {
        sweep += std::f64::consts::TAU;
    }
    let steps = n.max(2);
    let mut pts = Vec::with_capacity(steps + 1);
    let z_span = end.z - start.z;
    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let angle = a0 + sweep * t;
        pts.push(Vec3 {
            x: center.x + r * angle.cos(),
            y: center.y + r * angle.sin(),
            z: start.z + z_span * t,
        });
    }
    pts
}

fn parse_coord_geom_to_points(
    coord_geom: Node<'_, '_>,
    order: LandXmlPointOrder,
    scale: f64,
) -> Vec<Vec3> {
    let mut pts = Vec::new();
    for child in coord_geom.children().filter(|n| n.is_element()) {
        match tag_name(child) {
            "Line" => {
                let start_text = text_of_child(child, "Start");
                let end_text = text_of_child(child, "End");
                if let (Some(s), Some(e)) = (start_text, end_text) {
                    if let (Ok(sp), Ok(ep)) = (
                        parse_point_xyz(&s, order),
                        parse_point_xyz(&e, order),
                    ) {
                        let sp = Vec3 { x: sp.x * scale, y: sp.y * scale, z: sp.z * scale };
                        let ep = Vec3 { x: ep.x * scale, y: ep.y * scale, z: ep.z * scale };
                        if pts.is_empty() || pts.last().map_or(true, |l: &Vec3| l.distance(sp) > 1e-9) {
                            pts.push(sp);
                        }
                        pts.push(ep);
                    }
                }
            }
            "Curve" => {
                let start_text = text_of_child(child, "Start");
                let center_text = text_of_child(child, "Center");
                let end_text = text_of_child(child, "End");
                if let (Some(s), Some(c), Some(e)) = (start_text, center_text, end_text) {
                    if let (Ok(sp), Ok(cp), Ok(ep)) = (
                        parse_point_xyz(&s, order),
                        parse_point_xyz(&c, order),
                        parse_point_xyz(&e, order),
                    ) {
                        let sp = Vec3 { x: sp.x * scale, y: sp.y * scale, z: sp.z * scale };
                        let cp = Vec3 { x: cp.x * scale, y: cp.y * scale, z: cp.z * scale };
                        let ep = Vec3 { x: ep.x * scale, y: ep.y * scale, z: ep.z * scale };
                        let arc = sample_arc_points(sp, cp, ep, 32);
                        for (i, p) in arc.into_iter().enumerate() {
                            if i == 0 && !pts.is_empty() && pts.last().map_or(false, |l: &Vec3| l.distance(p) < 1e-9) {
                                continue;
                            }
                            pts.push(p);
                        }
                    }
                }
            }
            "Spiral" => {
                let start_text = text_of_child(child, "Start");
                let end_text = text_of_child(child, "End");
                if let (Some(s), Some(e)) = (start_text, end_text) {
                    if let (Ok(sp), Ok(ep)) = (
                        parse_point_xyz(&s, order),
                        parse_point_xyz(&e, order),
                    ) {
                        let sp = Vec3 { x: sp.x * scale, y: sp.y * scale, z: sp.z * scale };
                        let ep = Vec3 { x: ep.x * scale, y: ep.y * scale, z: ep.z * scale };
                        let n = 16;
                        for i in 0..=n {
                            let t = i as f64 / n as f64;
                            let p = Vec3 {
                                x: sp.x + (ep.x - sp.x) * t,
                                y: sp.y + (ep.y - sp.y) * t,
                                z: sp.z + (ep.z - sp.z) * t,
                            };
                            if i == 0 && !pts.is_empty() && pts.last().map_or(false, |l: &Vec3| l.distance(p) < 1e-9) {
                                continue;
                            }
                            pts.push(p);
                        }
                    }
                }
            }
            "IrregularLine" | "Chain" => {
                for pnt_list in child.children().filter(|n| n.is_element()) {
                    let tag = tag_name(pnt_list);
                    if tag == "PntList3D" {
                        if let Some(text) = pnt_list.text() {
                            pts.extend(parse_nez_point_list(text, 3, order, scale));
                        }
                    } else if tag == "PntList2D" {
                        if let Some(text) = pnt_list.text() {
                            pts.extend(parse_nez_point_list(text, 2, order, scale));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    pts
}

fn classify_plan_linear_kind(
    application_name: &str,
    plan_feature: Node<'_, '_>,
) -> PlanLinearKind {
    let is_openroads = application_name
        .to_ascii_lowercase()
        .contains("openroads");
    if is_openroads {
        return PlanLinearKind::Breakline;
    }
    let name_lower = plan_feature
        .attribute("name")
        .unwrap_or("")
        .to_ascii_lowercase();
    if name_lower.contains("breakline") {
        return PlanLinearKind::Breakline;
    }
    for child in plan_feature.children().filter(|n| n.is_element() && tag_name(*n) == "Feature") {
        let code = child.attribute("code").unwrap_or("").to_ascii_lowercase();
        if code.contains("breakline") {
            return PlanLinearKind::Breakline;
        }
    }
    PlanLinearKind::FeatureLine
}

fn parse_plan_linears(
    root: Node<'_, '_>,
    options: &LandXmlParseOptions,
    scale: f64,
    warnings: &mut Vec<LandXmlWarning>,
) -> Vec<PlanLinear> {
    let application_name = root
        .descendants()
        .find(|n| n.is_element() && tag_name(*n) == "Application")
        .and_then(|n| n.attribute("name"))
        .unwrap_or("");

    let mut out = Vec::new();

    for pf in root
        .descendants()
        .filter(|n| n.is_element() && tag_name(*n) == "PlanFeature")
    {
        let coord_geom = pf
            .children()
            .find(|n| n.is_element() && tag_name(*n) == "CoordGeom");
        let Some(cg) = coord_geom else {
            continue;
        };
        let points = parse_coord_geom_to_points(cg, options.point_order, scale);
        if points.len() < 2 {
            if options.mode == LandXmlParseMode::Lenient {
                push_warning(
                    warnings,
                    "plan_feature_few_points",
                    format!(
                        "Skipping PlanFeature '{}' with fewer than 2 points",
                        pf.attribute("name").unwrap_or("?")
                    ),
                    None,
                );
            }
            continue;
        }
        let name = pf
            .attribute("name")
            .map(str::to_string)
            .unwrap_or_else(|| format!("PlanFeature {}", out.len() + 1));
        let kind = classify_plan_linear_kind(application_name, pf);
        out.push(PlanLinear { name, kind, points });
    }

    for bl in root
        .descendants()
        .filter(|n| n.is_element() && tag_name(*n) == "Breakline")
    {
        let p3d = bl
            .children()
            .find(|n| n.is_element() && tag_name(*n) == "PntList3D");
        let p2d = bl
            .children()
            .find(|n| n.is_element() && tag_name(*n) == "PntList2D");
        let points = if let Some(node) = p3d {
            parse_nez_point_list(node.text().unwrap_or(""), 3, options.point_order, scale)
        } else if let Some(node) = p2d {
            parse_nez_point_list(node.text().unwrap_or(""), 2, options.point_order, scale)
        } else {
            continue;
        };
        if points.len() < 2 {
            continue;
        }
        let name = bl
            .attribute("name")
            .map(str::to_string)
            .unwrap_or_else(|| format!("Breakline {}", out.len() + 1));
        out.push(PlanLinear {
            name,
            kind: PlanLinearKind::Breakline,
            points,
        });
    }

    out
}

pub fn parse_landxml(
    xml: &str,
    options: LandXmlParseOptions,
) -> Result<LandXmlDocument, LandXmlError> {
    let xml = xml.strip_prefix('\u{FEFF}').unwrap_or(xml);
    let doc =
        Document::parse(xml).map_err(|e| LandXmlError::parse(format!("xml parse error: {e}")))?;
    let root = doc.root_element();

    let units = read_units(root, &options);
    let scale = if options.units_policy == LandXmlUnitsPolicy::NormalizeToMeters {
        units.linear_to_meters
    } else {
        1.0
    };

    let mut warnings = Vec::new();

    let alignments = parse_alignments(
        root,
        &options,
        units.source_angular_unit.as_deref(),
        scale,
        &mut warnings,
    )?;
    let surfaces = parse_surfaces(root, &options, scale, &mut warnings)?;
    let plan_linears = parse_plan_linears(root, &options, scale, &mut warnings);

    Ok(LandXmlDocument {
        units,
        alignments,
        surfaces,
        plan_linears,
        warnings,
    })
}
