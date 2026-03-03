use std::collections::BTreeMap;

use crate::RgmPoint3 as Vec3;

use super::error::LandXmlError;
use super::horizontal::evaluate_alignment_2d;
use super::station::display_to_internal_station;
use super::types::{Alignment3DSample, AlignmentRecord, ProfileSeries, Vec3Ops};
use super::vertical::evaluate_vertical_model;

pub fn evaluate_alignment_3d_internal(
    alignment: &AlignmentRecord,
    profile: &ProfileSeries,
    station_internal_m: f64,
) -> Result<Alignment3DSample, LandXmlError> {
    let planar = evaluate_alignment_2d(&alignment.horizontal, station_internal_m)?;
    let vertical_station = station_internal_m - alignment.sta_start_m;
    let (z, grade, vcurv) = evaluate_vertical_model(&profile.vertical_model, vertical_station)?;

    let dxy = Vec3 { x: planar.heading_rad.cos(), y: planar.heading_rad.sin(), z: 0.0 };
    let d3 = Vec3 { x: dxy.x, y: dxy.y, z: grade }.normalize();

    Ok(Alignment3DSample {
        point: Vec3 { x: planar.point.x, y: planar.point.y, z },
        tangent: d3,
        grade,
        horizontal_curvature_per_m: planar.curvature_per_m,
        vertical_curvature_per_m: vcurv,
    })
}

pub fn evaluate_alignment_3d(
    alignment: &AlignmentRecord,
    profile: &ProfileSeries,
    station_display_m: f64,
) -> Result<Alignment3DSample, LandXmlError> {
    let internal = display_to_internal_station(&alignment.station_map, station_display_m)?;
    evaluate_alignment_3d_internal(alignment, profile, internal)
}

pub fn sample_alignment_3d(
    alignment: &AlignmentRecord,
    profile: &ProfileSeries,
    start_station_display_m: f64,
    end_station_display_m: f64,
    step_m: f64,
) -> Result<Vec<Vec3>, LandXmlError> {
    if !step_m.is_finite() || step_m <= 0.0 {
        return Err(LandXmlError::invalid_input("step must be > 0"));
    }

    let mut out = Vec::new();
    let mut s = start_station_display_m;
    while s <= end_station_display_m + 1e-9 {
        out.push(evaluate_alignment_3d(alignment, profile, s)?.point);
        s += step_m;
    }
    if out.is_empty() {
        out.push(evaluate_alignment_3d(alignment, profile, start_station_display_m)?.point);
    }
    Ok(out)
}

pub fn sample_alignment_3d_all_profiles(
    alignment: &AlignmentRecord,
    start_station_display_m: f64,
    end_station_display_m: f64,
    step_m: f64,
) -> Result<BTreeMap<String, Vec<Vec3>>, LandXmlError> {
    let mut out = BTreeMap::new();
    for profile in &alignment.profiles {
        out.insert(
            profile.id.clone(),
            sample_alignment_3d(
                alignment,
                profile,
                start_station_display_m,
                end_station_display_m,
                step_m,
            )?,
        );
    }
    Ok(out)
}

