use super::error::LandXmlError;
use super::types::{StationIncrementDirection, StationMap};

#[derive(Clone, Copy, Debug)]
struct SegmentMap {
    int0: f64,
    int1: f64,
    sign: f64,
    offset: f64,
}

fn segment_maps(map: &StationMap) -> Vec<SegmentMap> {
    let mut equations = map.equations.clone();
    equations.sort_by(|a, b| {
        a.sta_internal_m
            .partial_cmp(&b.sta_internal_m)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut segs = Vec::new();
    let mut prev_int = map.sta_start_m;
    let mut sign = 1.0;
    let mut offset = map.sta_start_m;

    for eq in equations {
        if eq.sta_internal_m > prev_int {
            segs.push(SegmentMap {
                int0: prev_int,
                int1: eq.sta_internal_m,
                sign,
                offset,
            });
        }
        sign = if eq.increment == StationIncrementDirection::Decreasing {
            -1.0
        } else {
            1.0
        };
        offset = eq.sta_ahead_m - sign * eq.sta_internal_m;
        prev_int = eq.sta_internal_m;
    }

    let end_int = map.sta_start_m + map.length_m;
    if end_int >= prev_int {
        segs.push(SegmentMap {
            int0: prev_int,
            int1: end_int,
            sign,
            offset,
        });
    }
    segs
}

pub fn display_to_internal_station(
    map: &StationMap,
    display_station: f64,
) -> Result<f64, LandXmlError> {
    if !display_station.is_finite() {
        return Err(LandXmlError::invalid_input(
            "display station must be a finite number",
        ));
    }

    let segs = segment_maps(map);
    for seg in segs {
        let s0 = seg.sign * seg.int0 + seg.offset;
        let s1 = seg.sign * seg.int1 + seg.offset;
        let lo = s0.min(s1) - 1e-9;
        let hi = s0.max(s1) + 1e-9;
        if (lo..=hi).contains(&display_station) {
            return Ok((display_station - seg.offset) / seg.sign);
        }
    }

    Err(LandXmlError::out_of_range(format!(
        "display station {display_station} is outside mapped station range"
    )))
}

pub fn internal_to_display_station(
    map: &StationMap,
    internal_station: f64,
) -> Result<f64, LandXmlError> {
    if !internal_station.is_finite() {
        return Err(LandXmlError::invalid_input(
            "internal station must be a finite number",
        ));
    }

    let segs = segment_maps(map);
    for seg in segs {
        let lo = seg.int0.min(seg.int1) - 1e-9;
        let hi = seg.int0.max(seg.int1) + 1e-9;
        if (lo..=hi).contains(&internal_station) {
            return Ok(seg.sign * internal_station + seg.offset);
        }
    }

    Err(LandXmlError::out_of_range(format!(
        "internal station {internal_station} is outside mapped station range"
    )))
}
