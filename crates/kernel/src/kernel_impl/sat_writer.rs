// ─── ACIS SAT 7.0 Writer ─────────────────────────────────────────────────────
//
// Produces a simplified ACIS SAT (version 7.0) text file containing B-spline
// curve and surface definitions.  Standalone curves and surfaces are wrapped
// in a minimal body → lump → shell → face topology chain so they form valid
// SAT entities.  Full B-rep objects emit the complete topology graph.
//
// No external crates — output is built entirely with `std::fmt`.
//
// This file is `include!`-ed from ffi_impl.rs; all imports from foundation.rs
// and the session modules are already in scope.

// ─── Entity record builder ───────────────────────────────────────────────────

struct SatEntity {
    record: String,
}

struct SatWriter {
    entities: Vec<SatEntity>,
}

impl SatWriter {
    fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    fn push(&mut self, record: String) -> usize {
        let idx = self.entities.len();
        self.entities.push(SatEntity { record });
        idx
    }

    fn sat_ref(idx: usize) -> String {
        format!("${idx}")
    }

    // ── Curve helpers ────────────────────────────────────────────────────

    fn write_spline_curve(&mut self, core: &NurbsCurveCore) -> usize {
        let ncp = core.control_points.len();
        let nk = core.knots.len();
        let rational = if core.weights.iter().all(|&w| (w - 1.0).abs() < 1e-12) {
            "nonrational"
        } else {
            "rational"
        };
        let periodic = if core.periodic {
            "periodic"
        } else {
            "nonperiodic"
        };

        let mut s = String::with_capacity(512);
        s.push_str(&format!(
            "exactcur-curve $-1 nurbs {} {} {} {} {} ",
            core.degree, ncp, rational, periodic, nk
        ));

        for knot in &core.knots {
            sat_push_float(&mut s, *knot);
            s.push(' ');
        }

        if rational == "rational" {
            for w in &core.weights {
                sat_push_float(&mut s, *w);
                s.push(' ');
            }
        }

        for pt in &core.control_points {
            sat_push_float(&mut s, pt.x);
            s.push(' ');
            sat_push_float(&mut s, pt.y);
            s.push(' ');
            sat_push_float(&mut s, pt.z);
            s.push(' ');
        }

        s.push('#');
        self.push(s)
    }

    fn write_curve_entity(&mut self, core: &NurbsCurveCore) -> usize {
        let spline_idx = self.write_spline_curve(core);
        let rec = format!(
            "curve $-1 {} #",
            Self::sat_ref(spline_idx)
        );
        self.push(rec)
    }

    // ── Surface helpers ──────────────────────────────────────────────────

    fn write_spline_surface(
        &mut self,
        core: &NurbsSurfaceCore,
        transform: &[[f64; 4]; 4],
    ) -> usize {
        let ncp_u = core.control_u_count;
        let ncp_v = core.control_v_count;
        let nk_u = core.knots_u.len();
        let nk_v = core.knots_v.len();
        let rational = if core.weights.iter().all(|&w| (w - 1.0).abs() < 1e-12) {
            "nonrational"
        } else {
            "rational"
        };
        let periodic_u = if core.periodic_u {
            "periodic"
        } else {
            "nonperiodic"
        };
        let periodic_v = if core.periodic_v {
            "periodic"
        } else {
            "nonperiodic"
        };

        let mut s = String::with_capacity(2048);
        s.push_str(&format!(
            "spline-surface $-1 {} {} {} {} {} {} {} {} {} ",
            core.degree_u, core.degree_v,
            ncp_u, ncp_v,
            rational, periodic_u, periodic_v,
            nk_u, nk_v
        ));

        for knot in &core.knots_u {
            sat_push_float(&mut s, *knot);
            s.push(' ');
        }
        for knot in &core.knots_v {
            sat_push_float(&mut s, *knot);
            s.push(' ');
        }

        if rational == "rational" {
            for w in &core.weights {
                sat_push_float(&mut s, *w);
                s.push(' ');
            }
        }

        for pt in &core.control_points {
            let tp = matrix_apply_point(*transform, *pt);
            sat_push_float(&mut s, tp.x);
            s.push(' ');
            sat_push_float(&mut s, tp.y);
            s.push(' ');
            sat_push_float(&mut s, tp.z);
            s.push(' ');
        }

        s.push('#');
        self.push(s)
    }

    fn write_surface_entity(
        &mut self,
        core: &NurbsSurfaceCore,
        transform: &[[f64; 4]; 4],
    ) -> usize {
        let spline_idx = self.write_spline_surface(core, transform);
        let rec = format!(
            "surface $-1 {} #",
            Self::sat_ref(spline_idx)
        );
        self.push(rec)
    }

    // ── Topology wrappers for standalone geometry ────────────────────────

    fn write_point_entity(&mut self, pt: RgmPoint3) -> usize {
        let mut s = String::with_capacity(64);
        s.push_str("point $-1 ");
        sat_push_float(&mut s, pt.x);
        s.push(' ');
        sat_push_float(&mut s, pt.y);
        s.push(' ');
        sat_push_float(&mut s, pt.z);
        s.push_str(" #");
        self.push(s)
    }

    fn write_vertex(&mut self, point_idx: usize, edge_idx_ref: &str) -> usize {
        let rec = format!(
            "vertex {} $-1 {} #",
            edge_idx_ref,
            Self::sat_ref(point_idx)
        );
        self.push(rec)
    }

    fn write_edge(
        &mut self,
        v_start: usize,
        v_end: usize,
        coedge_ref: &str,
        curve_idx: usize,
    ) -> usize {
        let rec = format!(
            "edge {} {} {} {} forward #",
            coedge_ref,
            Self::sat_ref(v_start),
            Self::sat_ref(v_end),
            Self::sat_ref(curve_idx)
        );
        self.push(rec)
    }

    fn write_coedge(
        &mut self,
        next_ref: &str,
        prev_ref: &str,
        loop_ref: &str,
        edge_ref: &str,
        sense: &str,
    ) -> usize {
        let rec = format!(
            "coedge {} {} {} {} {} $-1 #",
            next_ref, prev_ref, loop_ref, edge_ref, sense
        );
        self.push(rec)
    }

    fn write_loop(&mut self, first_coedge: usize, face_ref: &str) -> usize {
        let rec = format!(
            "loop $-1 {} {} #",
            face_ref,
            Self::sat_ref(first_coedge)
        );
        self.push(rec)
    }

    #[allow(dead_code)]
    fn write_face(
        &mut self,
        first_loop: usize,
        shell_ref: &str,
        surface_idx: usize,
        sense: &str,
    ) -> usize {
        let rec = format!(
            "face {} {} {} {} double #",
            shell_ref,
            Self::sat_ref(first_loop),
            Self::sat_ref(surface_idx),
            sense
        );
        self.push(rec)
    }

    fn write_shell(&mut self, first_face: usize, lump_ref: &str) -> usize {
        let rec = format!(
            "shell {} $-1 {} #",
            lump_ref,
            Self::sat_ref(first_face)
        );
        self.push(rec)
    }

    fn write_lump(&mut self, shell_idx: usize, body_ref: &str) -> usize {
        let rec = format!(
            "lump {} $-1 {} #",
            body_ref,
            Self::sat_ref(shell_idx)
        );
        self.push(rec)
    }

    fn write_body(&mut self, lump_idx: usize) -> usize {
        let rec = format!(
            "body $-1 {} $-1 #",
            Self::sat_ref(lump_idx)
        );
        self.push(rec)
    }

    // ── High-level: wrap a standalone curve in minimal topology ──────────

    fn add_standalone_curve(&mut self, core: &NurbsCurveCore) {
        let curve_idx = self.write_curve_entity(core);

        let start_pt = core.control_points.first().copied().unwrap_or(RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        let end_pt = core.control_points.last().copied().unwrap_or(start_pt);

        let pt_start_idx = self.write_point_entity(start_pt);
        let pt_end_idx = self.write_point_entity(end_pt);

        let edge_placeholder = self.entities.len();
        let v_start_idx = self.write_vertex(pt_start_idx, &format!("${edge_placeholder}"));
        let v_end_idx = self.write_vertex(pt_end_idx, &format!("${edge_placeholder}"));

        let coedge_placeholder = self.entities.len() + 1;
        let coedge_self_ref = format!("${coedge_placeholder}");
        let loop_placeholder = self.entities.len() + 2;
        let loop_ref_s = format!("${loop_placeholder}");
        let face_placeholder = self.entities.len() + 3;
        let face_ref_s = format!("${face_placeholder}");
        let shell_placeholder = self.entities.len() + 4;
        let shell_ref_s = format!("${shell_placeholder}");
        let lump_placeholder = self.entities.len() + 5;
        let lump_ref_s = format!("${lump_placeholder}");
        let body_placeholder = self.entities.len() + 6;
        let body_ref_s = format!("${body_placeholder}");

        let edge_idx = self.write_edge(
            v_start_idx,
            v_end_idx,
            &coedge_self_ref,
            curve_idx,
        );
        let coedge_idx = self.write_coedge(
            &coedge_self_ref,
            &coedge_self_ref,
            &loop_ref_s,
            &Self::sat_ref(edge_idx),
            "forward",
        );
        let loop_idx = self.write_loop(coedge_idx, &face_ref_s);

        let _ = face_placeholder;
        let _ = shell_placeholder;
        let _ = lump_placeholder;
        let _ = body_placeholder;

        let face_idx = self.push(format!(
            "face {} {} $-1 forward double #",
            shell_ref_s,
            Self::sat_ref(loop_idx)
        ));
        let shell_idx = self.write_shell(face_idx, &lump_ref_s);
        let lump_idx = self.write_lump(shell_idx, &body_ref_s);
        self.write_body(lump_idx);
    }

    // ── High-level: wrap a standalone surface in minimal topology ────────

    fn add_standalone_surface(
        &mut self,
        core: &NurbsSurfaceCore,
        transform: &[[f64; 4]; 4],
    ) {
        let surface_idx = self.write_surface_entity(core, transform);

        let shell_placeholder = self.entities.len() + 1;
        let shell_ref_s = format!("${shell_placeholder}");
        let lump_placeholder = self.entities.len() + 2;
        let lump_ref_s = format!("${lump_placeholder}");
        let body_placeholder = self.entities.len() + 3;
        let body_ref_s = format!("${body_placeholder}");

        let face_idx = self.push(format!(
            "face {} $-1 {} forward double #",
            shell_ref_s,
            Self::sat_ref(surface_idx)
        ));
        let shell_idx = self.write_shell(face_idx, &lump_ref_s);
        let lump_idx = self.write_lump(shell_idx, &body_ref_s);
        self.write_body(lump_idx);
    }

    // ── Serialization ────────────────────────────────────────────────────

    fn finish(self) -> String {
        let mut output = String::with_capacity(4096);

        // Header line 1: version
        output.push_str("700 0 1 0\n");

        // Header line 2: byte count of next line (will compute)
        let header_content = sat_build_header_line();
        output.push_str(&format!("{}\n", header_content.len()));
        output.push_str(&header_content);
        output.push('\n');

        // Header line 4: units and tolerances
        output.push_str("1.0 9.9999999999999995e-007 1e-010\n");

        // Entity records
        for entity in &self.entities {
            output.push_str(&entity.record);
            output.push('\n');
        }

        output.push_str("End-of-ACIS-data\n");
        output
    }
}

// ─── Formatting helpers ──────────────────────────────────────────────────────

fn sat_push_float(buf: &mut String, v: f64) {
    if v == 0.0 {
        buf.push('0');
        return;
    }
    if v == v.floor() && v.abs() < 1e15 {
        buf.push_str(&format!("{}", v as i64));
        return;
    }
    let s = format!("{:.17e}", v);
    buf.push_str(&s);
}

fn sat_build_header_line() -> String {
    let acis_version = "@7 ACIS 7.0";
    let product_id = "@24 rusted-geom ACIS exporter";
    let acis_proc = "@4 none";
    let date_str = "@10 2026-03-04";
    format!("{} {} {} {}", acis_version, product_id, acis_proc, date_str)
}

// ─── Public entry point ──────────────────────────────────────────────────────

pub(crate) fn export_sat_text(
    session: RgmKernelHandle,
    object_ids: &[u64],
) -> Result<String, String> {
    let entry = SESSIONS
        .get(&session.0)
        .ok_or_else(|| "Session not found".to_string())?;
    let state = entry.value().read();

    let mut writer = SatWriter::new();

    for &obj_id in object_ids {
        let obj = state
            .objects
            .get(&obj_id)
            .ok_or_else(|| format!("Object {obj_id} not found"))?;

        match obj {
            GeometryObject::Curve(data) => {
                if let Some(nurbs) = curve_canonical_nurbs(data) {
                    writer.add_standalone_curve(&nurbs.core);
                }
            }
            GeometryObject::Surface(data) => {
                writer.add_standalone_surface(&data.core, &data.transform);
            }
            GeometryObject::LandXmlDoc(doc_data) => {
                export_landxml_curves_as_nurbs_sat(&mut writer, doc_data);
            }
            GeometryObject::Mesh(_)
            | GeometryObject::Intersection(_) => {}
        }
    }

    Ok(writer.finish())
}

fn export_landxml_curves_as_nurbs_sat(
    writer: &mut SatWriter,
    doc_data: &crate::session::objects::LandXmlDocData,
) {
    use crate::landxml::evaluate_alignment_3d;

    let doc = &doc_data.doc;
    let n_steps: usize = 500;

    for alignment in &doc.alignments {
        let sta_start = alignment.sta_start_m;
        let sta_end = sta_start + alignment.length_m;
        if sta_end <= sta_start {
            continue;
        }
        let step = (sta_end - sta_start) / n_steps as f64;

        for profile in &alignment.profiles {
            let mut pts = Vec::new();
            let mut s = sta_start;
            while s <= sta_end + 1e-9 {
                if let Ok(sample) = evaluate_alignment_3d(alignment, profile, s.min(sta_end)) {
                    pts.push(sample.point);
                }
                s += step;
            }

            if pts.len() < 4 {
                continue;
            }

            if let Some(core) = fit_nurbs_through_points(&pts, 3) {
                writer.add_standalone_curve(&core);
            }
        }
    }
}
