use super::{KernelSession, LandXmlDocHandle, MeshHandle};
use crate::landxml::{
    self, evaluate_alignment_3d, LandXmlParseMode, LandXmlParseOptions, LandXmlPointOrder,
    LandXmlUnitsPolicy,
};
use crate::session::objects::LandXmlDocData;
use crate::session::store::{insert_landxml_doc, insert_mesh, with_session_mut, SESSIONS};
use crate::RgmObjectHandle;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl KernelSession {
    /// Parse a LandXML 1.2 string.
    ///
    /// `mode`: 0 = Strict, 1 = Lenient.
    /// `point_order`: 0 = NEZ, 1 = ENZ, 2 = EZN.
    /// `units_policy`: 0 = NormalizeToMeters, 1 = PreserveSource.
    pub fn landxml_parse(
        &self,
        xml: &str,
        mode: u32,
        point_order: u32,
        units_policy: u32,
    ) -> Result<LandXmlDocHandle, JsValue> {
        let options = LandXmlParseOptions {
            mode: match mode {
                1 => LandXmlParseMode::Lenient,
                _ => LandXmlParseMode::Strict,
            },
            point_order: match point_order {
                1 => LandXmlPointOrder::Enz,
                2 => LandXmlPointOrder::Ezn,
                _ => LandXmlPointOrder::Nez,
            },
            units_policy: match units_policy {
                1 => LandXmlUnitsPolicy::PreserveSource,
                _ => LandXmlUnitsPolicy::NormalizeToMeters,
            },
            angular_unit_override: None,
        };

        let doc = landxml::parse_landxml(xml, options)
            .map_err(|e| JsValue::from_str(&e.message))?;

        let data = LandXmlDocData { doc };
        let handle = with_session_mut(self.handle(), |state| {
            Ok(insert_landxml_doc(state, data))
        })
        .map_err(super::error::js_err)?;

        Ok(LandXmlDocHandle::new(self.session_id, handle.0))
    }

    /// Number of TIN surfaces in the document.
    pub fn landxml_surface_count(&self, doc: &LandXmlDocHandle) -> Result<u32, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let data = crate::session::objects::find_landxml_doc(
            &state,
            RgmObjectHandle(doc.object_id),
        )
        .map_err(super::error::js_err)?;
        Ok(data.doc.surfaces.len() as u32)
    }

    /// Name of surface at `index`.
    pub fn landxml_surface_name(
        &self,
        doc: &LandXmlDocHandle,
        index: u32,
    ) -> Result<String, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let data = crate::session::objects::find_landxml_doc(
            &state,
            RgmObjectHandle(doc.object_id),
        )
        .map_err(super::error::js_err)?;
        let surface = data
            .doc
            .surfaces
            .get(index as usize)
            .ok_or_else(|| JsValue::from_str("surface index out of range"))?;
        Ok(surface.name.clone())
    }

    /// Copy raw vertices of surface `index` as flat `[x,y,z, ...]` f64 (UTM coordinates).
    pub fn landxml_surface_copy_vertices(
        &self,
        doc: &LandXmlDocHandle,
        index: u32,
    ) -> Result<Vec<f64>, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let data = crate::session::objects::find_landxml_doc(
            &state,
            RgmObjectHandle(doc.object_id),
        )
        .map_err(super::error::js_err)?;
        let surface = data
            .doc
            .surfaces
            .get(index as usize)
            .ok_or_else(|| JsValue::from_str("surface index out of range"))?;
        let flat: Vec<f64> = surface
            .vertices_m
            .iter()
            .flat_map(|p| [p.x, p.y, p.z])
            .collect();
        Ok(flat)
    }

    /// Copy triangle indices of surface `index` as flat `[i0,i1,i2, ...]`.
    pub fn landxml_surface_copy_indices(
        &self,
        doc: &LandXmlDocHandle,
        index: u32,
    ) -> Result<Vec<u32>, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let data = crate::session::objects::find_landxml_doc(
            &state,
            RgmObjectHandle(doc.object_id),
        )
        .map_err(super::error::js_err)?;
        let surface = data
            .doc
            .surfaces
            .get(index as usize)
            .ok_or_else(|| JsValue::from_str("surface index out of range"))?;
        Ok(surface.triangles.clone())
    }

    /// Number of alignments in the document.
    pub fn landxml_alignment_count(&self, doc: &LandXmlDocHandle) -> Result<u32, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let data = crate::session::objects::find_landxml_doc(
            &state,
            RgmObjectHandle(doc.object_id),
        )
        .map_err(super::error::js_err)?;
        Ok(data.doc.alignments.len() as u32)
    }

    /// Name of alignment at `index`.
    pub fn landxml_alignment_name(
        &self,
        doc: &LandXmlDocHandle,
        index: u32,
    ) -> Result<String, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let data = crate::session::objects::find_landxml_doc(
            &state,
            RgmObjectHandle(doc.object_id),
        )
        .map_err(super::error::js_err)?;
        let alignment = data
            .doc
            .alignments
            .get(index as usize)
            .ok_or_else(|| JsValue::from_str("alignment index out of range"))?;
        Ok(alignment.name.clone())
    }

    /// Sample 2D horizontal segments with z=0, color-coded by type.
    ///
    /// Format: `[seg_count, type0, npts0, x,y,0,..., type1, npts1, x,y,0,..., ...]`
    /// type: 0 = Line, 1 = Arc, 2 = Spiral.
    /// Adaptive step: spirals use `span / 240`, lines/arcs use `span / 120`.
    pub fn landxml_sample_horiz_2d_segments(
        &self,
        doc: &LandXmlDocHandle,
        alignment_index: u32,
    ) -> Result<Vec<f64>, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let data = crate::session::objects::find_landxml_doc(
            &state,
            RgmObjectHandle(doc.object_id),
        )
        .map_err(super::error::js_err)?;
        let alignment = data
            .doc
            .alignments
            .get(alignment_index as usize)
            .ok_or_else(|| JsValue::from_str("alignment index out of range"))?;

        let segments = &alignment.horizontal.segments;
        let mut packed = Vec::new();
        packed.push(segments.len() as f64);

        for seg in segments {
            let (seg_type, s0, len) = match seg {
                landxml::HorizontalSegment::Line(s) => (0.0_f64, s.start_station_m, s.length_m),
                landxml::HorizontalSegment::CircularArc(s) => {
                    (1.0_f64, s.start_station_m, s.length_m)
                }
                landxml::HorizontalSegment::Spiral(s) => {
                    (2.0_f64, s.start_station_m, s.length_m)
                }
            };

            let divisor = if seg_type == 2.0 { 240.0 } else { 120.0 };
            let step = (len / divisor).clamp(0.1, 2.0).max(len / 1200.0);
            let mut station = s0;
            let s_end = s0 + len;
            let mut pts: Vec<f64> = Vec::new();
            while station <= s_end + 1e-9 {
                let s_clamped = station.min(s_end);
                if let Ok(sample) =
                    landxml::evaluate_alignment_2d(&alignment.horizontal, s_clamped)
                {
                    pts.extend_from_slice(&[sample.point.x, sample.point.y, 0.0]);
                }
                station += step;
            }
            packed.push(seg_type);
            packed.push((pts.len() / 3) as f64);
            packed.extend_from_slice(&pts);
        }

        Ok(packed)
    }

    /// Sample a 3D resultant track for one alignment + one profile.
    ///
    /// Format: `[npts, x,y,z, x,y,z, ...]`.
    /// Evaluates per-station and skips individual failures (matching the
    /// reference viewer's try-catch-per-station pattern).
    pub fn landxml_sample_alignment_3d(
        &self,
        doc: &LandXmlDocHandle,
        alignment_index: u32,
        profile_index: u32,
        n_steps: u32,
    ) -> Result<Vec<f64>, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let data = crate::session::objects::find_landxml_doc(
            &state,
            RgmObjectHandle(doc.object_id),
        )
        .map_err(super::error::js_err)?;
        let alignment = data
            .doc
            .alignments
            .get(alignment_index as usize)
            .ok_or_else(|| JsValue::from_str("alignment index out of range"))?;
        let profile = alignment
            .profiles
            .get(profile_index as usize)
            .ok_or_else(|| JsValue::from_str("profile index out of range"))?;

        let sta_start = alignment.sta_start_m;
        let sta_end = sta_start + alignment.length_m;
        let n = n_steps.max(2) as usize;
        let step = (sta_end - sta_start) / (n - 1).max(1) as f64;

        let mut packed = Vec::with_capacity(1 + n * 3);
        packed.push(0.0); // placeholder for point count
        let mut count = 0usize;
        for i in 0..n {
            let s = (sta_start + step * i as f64).min(sta_end);
            if let Ok(sample) = evaluate_alignment_3d(alignment, profile, s) {
                packed.extend_from_slice(&[sample.point.x, sample.point.y, sample.point.z]);
                count += 1;
            }
        }
        packed[0] = count as f64;
        Ok(packed)
    }

    /// Number of profiles for alignment at `alignment_index`.
    pub fn landxml_alignment_profile_count(
        &self,
        doc: &LandXmlDocHandle,
        alignment_index: u32,
    ) -> Result<u32, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let data = crate::session::objects::find_landxml_doc(
            &state,
            RgmObjectHandle(doc.object_id),
        )
        .map_err(super::error::js_err)?;
        let alignment = data
            .doc
            .alignments
            .get(alignment_index as usize)
            .ok_or_else(|| JsValue::from_str("alignment index out of range"))?;
        Ok(alignment.profiles.len() as u32)
    }

    /// Name/ID of profile at `profile_index` for alignment at `alignment_index`.
    pub fn landxml_alignment_profile_name(
        &self,
        doc: &LandXmlDocHandle,
        alignment_index: u32,
        profile_index: u32,
    ) -> Result<String, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let data = crate::session::objects::find_landxml_doc(
            &state,
            RgmObjectHandle(doc.object_id),
        )
        .map_err(super::error::js_err)?;
        let alignment = data
            .doc
            .alignments
            .get(alignment_index as usize)
            .ok_or_else(|| JsValue::from_str("alignment index out of range"))?;
        let profile = alignment
            .profiles
            .get(profile_index as usize)
            .ok_or_else(|| JsValue::from_str("profile index out of range"))?;
        Ok(profile.id.clone())
    }

    /// Number of parse warnings.
    pub fn landxml_warning_count(&self, doc: &LandXmlDocHandle) -> Result<u32, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let data = crate::session::objects::find_landxml_doc(
            &state,
            RgmObjectHandle(doc.object_id),
        )
        .map_err(super::error::js_err)?;
        Ok(data.doc.warnings.len() as u32)
    }

    /// Source linear unit string (e.g. "meter", "USSurveyFoot").
    pub fn landxml_linear_unit(&self, doc: &LandXmlDocHandle) -> Result<String, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let data = crate::session::objects::find_landxml_doc(
            &state,
            RgmObjectHandle(doc.object_id),
        )
        .map_err(super::error::js_err)?;
        Ok(data.doc.units.source_linear_unit.clone())
    }

    /// Extract a TIN surface as a session mesh, enabling kernel-side operations.
    pub fn landxml_extract_surface_mesh(
        &self,
        doc: &LandXmlDocHandle,
        index: u32,
    ) -> Result<MeshHandle, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;

        let (vertices, triangles) = {
            let state = entry.value().read();
            let data = crate::session::objects::find_landxml_doc(
                &state,
                RgmObjectHandle(doc.object_id),
            )
            .map_err(super::error::js_err)?;
            let surface = data
                .doc
                .surfaces
                .get(index as usize)
                .ok_or_else(|| JsValue::from_str("surface index out of range"))?;

            let verts: Vec<crate::RgmPoint3> = surface.vertices_m.clone();
            let tris: Vec<[u32; 3]> = surface
                .triangles
                .chunks_exact(3)
                .map(|c| [c[0], c[1], c[2]])
                .collect();
            (verts, tris)
        };

        let mesh_data = crate::session::objects::MeshData {
            vertices,
            triangles,
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };

        let handle = with_session_mut(self.handle(), |state| Ok(insert_mesh(state, mesh_data)))
            .map_err(super::error::js_err)?;

        Ok(MeshHandle::new(self.session_id, handle.0))
    }

    /// Station range `[sta_start, sta_end]` for alignment at `alignment_index`.
    pub fn landxml_alignment_station_range(
        &self,
        doc: &LandXmlDocHandle,
        alignment_index: u32,
    ) -> Result<Vec<f64>, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let data = crate::session::objects::find_landxml_doc(
            &state,
            RgmObjectHandle(doc.object_id),
        )
        .map_err(super::error::js_err)?;
        let alignment = data
            .doc
            .alignments
            .get(alignment_index as usize)
            .ok_or_else(|| JsValue::from_str("alignment index out of range"))?;
        Ok(vec![alignment.sta_start_m, alignment.sta_start_m + alignment.length_m])
    }

    /// Probe an alignment+profile at a display station.
    ///
    /// Returns packed `[px, py, pz, tx, ty, tz, grade]` where:
    ///   - `(px,py,pz)` = 3D point on the alignment+profile at the probe station,
    ///   - `(tx,ty,tz)` = 3D tangent vector,
    ///   - `grade`      = vertical grade (rise/run).
    ///
    /// The caller constructs the perpendicular plane from the tangent.
    pub fn landxml_probe_alignment(
        &self,
        doc: &LandXmlDocHandle,
        alignment_index: u32,
        profile_index: u32,
        display_station: f64,
    ) -> Result<Vec<f64>, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let data = crate::session::objects::find_landxml_doc(
            &state,
            RgmObjectHandle(doc.object_id),
        )
        .map_err(super::error::js_err)?;
        let alignment = data
            .doc
            .alignments
            .get(alignment_index as usize)
            .ok_or_else(|| JsValue::from_str("alignment index out of range"))?;
        let profile = alignment
            .profiles
            .get(profile_index as usize)
            .ok_or_else(|| JsValue::from_str("profile index out of range"))?;

        let sample = evaluate_alignment_3d(alignment, profile, display_station)
            .map_err(|e| JsValue::from_str(&e.message))?;

        Ok(vec![
            sample.point.x, sample.point.y, sample.point.z,
            sample.tangent.x, sample.tangent.y, sample.tangent.z,
            sample.grade,
        ])
    }

    /// Number of plan linears (FeatureLines + Breaklines) in the document.
    pub fn landxml_plan_linear_count(&self, doc: &LandXmlDocHandle) -> Result<u32, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let data = crate::session::objects::find_landxml_doc(
            &state,
            RgmObjectHandle(doc.object_id),
        )
        .map_err(super::error::js_err)?;
        Ok(data.doc.plan_linears.len() as u32)
    }

    /// Name of plan linear at `index`.
    pub fn landxml_plan_linear_name(
        &self,
        doc: &LandXmlDocHandle,
        index: u32,
    ) -> Result<String, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let data = crate::session::objects::find_landxml_doc(
            &state,
            RgmObjectHandle(doc.object_id),
        )
        .map_err(super::error::js_err)?;
        let linear = data
            .doc
            .plan_linears
            .get(index as usize)
            .ok_or_else(|| JsValue::from_str("plan linear index out of range"))?;
        Ok(linear.name.clone())
    }

    /// Kind of plan linear at `index`: 0 = FeatureLine, 1 = Breakline.
    pub fn landxml_plan_linear_kind(
        &self,
        doc: &LandXmlDocHandle,
        index: u32,
    ) -> Result<u32, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let data = crate::session::objects::find_landxml_doc(
            &state,
            RgmObjectHandle(doc.object_id),
        )
        .map_err(super::error::js_err)?;
        let linear = data
            .doc
            .plan_linears
            .get(index as usize)
            .ok_or_else(|| JsValue::from_str("plan linear index out of range"))?;
        Ok(match linear.kind {
            landxml::PlanLinearKind::FeatureLine => 0,
            landxml::PlanLinearKind::Breakline => 1,
        })
    }

    /// Copy points of plan linear at `index` as flat `[x,y,z, ...]`.
    pub fn landxml_plan_linear_copy_points(
        &self,
        doc: &LandXmlDocHandle,
        index: u32,
    ) -> Result<Vec<f64>, JsValue> {
        let entry = SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let data = crate::session::objects::find_landxml_doc(
            &state,
            RgmObjectHandle(doc.object_id),
        )
        .map_err(super::error::js_err)?;
        let linear = data
            .doc
            .plan_linears
            .get(index as usize)
            .ok_or_else(|| JsValue::from_str("plan linear index out of range"))?;
        let flat: Vec<f64> = linear
            .points
            .iter()
            .flat_map(|p| [p.x, p.y, p.z])
            .collect();
        Ok(flat)
    }
}
