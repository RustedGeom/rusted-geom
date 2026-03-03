use std::f64::consts::TAU;

use crate::RgmPoint3 as Vec3;

use super::error::LandXmlError;
use super::spiral::evaluate_spiral_local;
use super::types::{
    Alignment2DSample, CircularArcSegment, HorizontalAlignment, HorizontalSegment, LineSegment,
    SpiralSegment, Vec3Ops,
};

fn rotate_2d(x: f64, y: f64, angle: f64) -> (f64, f64) {
    let c = angle.cos();
    let s = angle.sin();
    (x * c - y * s, x * s + y * c)
}

fn eval_line(line: &LineSegment, s: f64) -> Alignment2DSample {
    let t = if line.length_m <= 1e-12 {
        0.0
    } else {
        (s / line.length_m).clamp(0.0, 1.0)
    };
    let p = line.start.add(line.end.sub(line.start).scale(t));
    Alignment2DSample {
        point: p,
        heading_rad: line.start_heading_rad,
        curvature_per_m: 0.0,
    }
}

fn eval_arc(arc: &CircularArcSegment, s: f64) -> Alignment2DSample {
    let t = if arc.length_m <= 1e-12 {
        0.0
    } else {
        (s / arc.length_m).clamp(0.0, 1.0)
    };
    let theta = if arc.clockwise {
        arc.start_angle_rad - arc.sweep_rad * t
    } else {
        arc.start_angle_rad + arc.sweep_rad * t
    };
    let p = Vec3 {
        x: arc.center.x + arc.radius_m * theta.cos(),
        y: arc.center.y + arc.radius_m * theta.sin(),
        z: arc.start.z,
    };

    let heading = if arc.clockwise {
        theta - std::f64::consts::FRAC_PI_2
    } else {
        theta + std::f64::consts::FRAC_PI_2
    };
    let k = if arc.radius_m.abs() < 1e-12 {
        0.0
    } else if arc.clockwise {
        -1.0 / arc.radius_m
    } else {
        1.0 / arc.radius_m
    };

    Alignment2DSample {
        point: p,
        heading_rad: heading,
        curvature_per_m: k,
    }
}

fn eval_spiral(seg: &SpiralSegment, s: f64) -> Alignment2DSample {
    let local = evaluate_spiral_local(seg, s);
    let (dx, dy) = rotate_2d(local.x_local, local.y_local, seg.start_heading_rad);

    Alignment2DSample {
        point: Vec3 { x: seg.start.x + dx, y: seg.start.y + dy, z: seg.start.z },
        heading_rad: local.heading_rad,
        curvature_per_m: local.curvature_per_m,
    }
}

pub fn evaluate_alignment_2d(
    alignment: &HorizontalAlignment,
    station_internal_m: f64,
) -> Result<Alignment2DSample, LandXmlError> {
    if alignment.segments.is_empty() {
        return Err(LandXmlError::not_found(
            "alignment has no horizontal segments",
        ));
    }
    if !station_internal_m.is_finite() {
        return Err(LandXmlError::invalid_input(
            "station must be a finite number",
        ));
    }

    for segment in &alignment.segments {
        let (s0, len) = match segment {
            HorizontalSegment::Line(s) => (s.start_station_m, s.length_m),
            HorizontalSegment::CircularArc(s) => (s.start_station_m, s.length_m),
            HorizontalSegment::Spiral(s) => (s.start_station_m, s.length_m),
        };
        if station_internal_m + 1e-9 < s0 || station_internal_m - 1e-9 > s0 + len {
            continue;
        }
        let local = (station_internal_m - s0).clamp(0.0, len.max(0.0));
        return Ok(match segment {
            HorizontalSegment::Line(s) => eval_line(s, local),
            HorizontalSegment::CircularArc(s) => eval_arc(s, local),
            HorizontalSegment::Spiral(s) => eval_spiral(s, local),
        });
    }

    if let Some(last) = alignment.segments.last() {
        let out = match last {
            HorizontalSegment::Line(s) => eval_line(s, s.length_m),
            HorizontalSegment::CircularArc(s) => eval_arc(s, s.length_m),
            HorizontalSegment::Spiral(s) => eval_spiral(s, s.length_m),
        };
        return Ok(out);
    }

    Err(LandXmlError::out_of_range(format!(
        "station {station_internal_m} is outside horizontal alignment range"
    )))
}

pub fn heading_from_chord(start: Vec3, end: Vec3) -> f64 {
    (end.y - start.y).atan2(end.x - start.x)
}

pub fn parse_rotation_clockwise(raw: Option<&str>) -> bool {
    let Some(v) = raw else {
        return false;
    };
    let low = v.trim().to_ascii_lowercase();
    matches!(low.as_str(), "cw" | "clockwise" | "true" | "1")
}

pub fn arc_sweep_radians(start_angle: f64, end_angle: f64, clockwise: bool) -> f64 {
    if clockwise {
        let mut sweep = start_angle - end_angle;
        while sweep < 0.0 {
            sweep += TAU;
        }
        sweep
    } else {
        let mut sweep = end_angle - start_angle;
        while sweep < 0.0 {
            sweep += TAU;
        }
        sweep
    }
}
