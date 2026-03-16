// ─── IGES 5.3 Writer ─────────────────────────────────────────────────────────
//
// Produces an IGES 5.3 text file containing B-spline curve (Entity 126) and
// surface (Entity 128) definitions extracted from the session.
//
// This file is `include!`-ed from ffi_impl.rs; all imports from foundation.rs
// and the session modules are already in scope.

struct IgesWriter {
    start_lines: Vec<String>,
    directory_entries: Vec<[String; 2]>,
    parameter_sections: Vec<String>,
    param_line_counter: usize,
}

impl IgesWriter {
    fn new() -> Self {
        Self {
            start_lines: vec!["rusted-geom IGES export".to_string()],
            directory_entries: Vec::new(),
            parameter_sections: Vec::new(),
            param_line_counter: 0,
        }
    }

    fn add_entity_126(&mut self, core: &NurbsCurveCore) -> usize {
        let k = core.control_points.len() as i32 - 1;
        let m = core.degree as i32;
        let prop1 = 0; // not planar
        let prop2 = if core.periodic { 1 } else { 0 };
        let prop3 = if core.weights.iter().all(|&w| (w - 1.0).abs() < 1e-12) { 1 } else { 0 };
        let prop4 = if core.periodic { 1 } else { 0 };

        let mut params = Vec::new();
        params.push("126".to_string());
        params.push(format!("{k}"));
        params.push(format!("{m}"));
        params.push(format!("{prop1}"));
        params.push(format!("{prop2}"));
        params.push(format!("{prop3}"));
        params.push(format!("{prop4}"));

        for knot in &core.knots {
            params.push(iges_fmt(*knot));
        }
        for w in &core.weights {
            params.push(iges_fmt(*w));
        }
        for pt in &core.control_points {
            params.push(iges_fmt(pt.x));
            params.push(iges_fmt(pt.y));
            params.push(iges_fmt(pt.z));
        }
        params.push(iges_fmt(core.u_start));
        params.push(iges_fmt(core.u_end));
        params.push(iges_fmt(0.0)); // XNORM
        params.push(iges_fmt(0.0)); // YNORM
        params.push(iges_fmt(1.0)); // ZNORM

        self.add_entity(126, 0, &params)
    }

    fn add_entity_128(&mut self, core: &NurbsSurfaceCore, transform: &[[f64; 4]; 4]) -> usize {
        let k1 = core.control_u_count as i32 - 1;
        let k2 = core.control_v_count as i32 - 1;
        let m1 = core.degree_u as i32;
        let m2 = core.degree_v as i32;
        let prop1 = 0; // not closed in u
        let prop2 = 0; // not closed in v
        let prop3 = if core.weights.iter().all(|&w| (w - 1.0).abs() < 1e-12) { 1 } else { 0 };
        let prop4 = if core.periodic_u { 1 } else { 0 };
        let prop5 = if core.periodic_v { 1 } else { 0 };

        let mut params = Vec::new();
        params.push("128".to_string());
        params.push(format!("{k1}"));
        params.push(format!("{k2}"));
        params.push(format!("{m1}"));
        params.push(format!("{m2}"));
        params.push(format!("{prop1}"));
        params.push(format!("{prop2}"));
        params.push(format!("{prop3}"));
        params.push(format!("{prop4}"));
        params.push(format!("{prop5}"));

        for knot in &core.knots_u {
            params.push(iges_fmt(*knot));
        }
        for knot in &core.knots_v {
            params.push(iges_fmt(*knot));
        }

        // IGES Entity 128 stores weights/points in u-major order (u varies fastest):
        //   W(0,0), W(1,0), ..., W(K1,0), W(0,1), ..., W(K1,K2)
        // The kernel stores them in v-major order: idx = u * v_count + v.
        // Transpose here to match the IGES convention.
        let nu = core.control_u_count;
        let nv = core.control_v_count;
        for j in 0..nv {
            for i in 0..nu {
                let idx = i * nv + j;
                params.push(iges_fmt(core.weights[idx]));
            }
        }
        for j in 0..nv {
            for i in 0..nu {
                let idx = i * nv + j;
                let tp = matrix_apply_point(*transform, core.control_points[idx]);
                params.push(iges_fmt(tp.x));
                params.push(iges_fmt(tp.y));
                params.push(iges_fmt(tp.z));
            }
        }

        params.push(iges_fmt(core.u_start));
        params.push(iges_fmt(core.u_end));
        params.push(iges_fmt(core.v_start));
        params.push(iges_fmt(core.v_end));

        self.add_entity(128, 0, &params)
    }


    fn add_entity(&mut self, entity_type: i32, form: i32, params: &[String]) -> usize {
        let de_seq = self.directory_entries.len() * 2 + 1;
        let param_start = self.param_line_counter + 1;
        let param_text = params.join(",") + ";";
        let param_lines = iges_wrap_param_data(&param_text, self.param_line_counter, de_seq);
        let num_param_lines = param_lines.len();
        self.param_line_counter += num_param_lines;
        self.parameter_sections.extend(param_lines);

        let line1 = format!(
            "{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}",
            entity_type, param_start, 0, 0, 0, 0, 0, 0, "00000000"
        );
        let line2 = format!(
            "{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}",
            entity_type, 0, 0, num_param_lines, form, " ", " ", " ", 0
        );

        self.directory_entries.push([line1, line2]);
        de_seq
    }

    fn finish(self) -> String {
        let mut output = String::with_capacity(8192);

        let num_start = self.start_lines.len();
        for (i, line) in self.start_lines.iter().enumerate() {
            output.push_str(&iges_pad_line(line, 'S', i + 1));
            output.push('\n');
        }

        let global = build_global_section();
        let global_lines = iges_wrap_section_data(&global, 'G');
        let num_global = global_lines.len();
        for line in &global_lines {
            output.push_str(line);
            output.push('\n');
        }

        let num_dir = self.directory_entries.len() * 2;
        for (i, [l1, l2]) in self.directory_entries.iter().enumerate() {
            output.push_str(&iges_pad_line(l1, 'D', i * 2 + 1));
            output.push('\n');
            output.push_str(&iges_pad_line(l2, 'D', i * 2 + 2));
            output.push('\n');
        }

        let num_param = self.parameter_sections.len();
        for line in &self.parameter_sections {
            output.push_str(line);
            output.push('\n');
        }

        let term = format!(
            "{:>8}{:>8}{:>8}{:>8}",
            format!("S{:>7}", num_start),
            format!("G{:>7}", num_global),
            format!("D{:>7}", num_dir),
            format!("P{:>7}", num_param),
        );
        output.push_str(&iges_pad_line(&term, 'T', 1));
        output.push('\n');

        output
    }
}

fn iges_hollerith(s: &str) -> String {
    format!("{}H{}", s.len(), s)
}

fn iges_fmt(v: f64) -> String {
    if v == 0.0 {
        return "0.".to_string();
    }
    if v == v.floor() && v.abs() < 1e12 {
        return format!("{}.", v as i64);
    }
    format!("{:.15E}", v)
}

fn iges_pad_line(content: &str, section: char, seq: usize) -> String {
    let trimmed = if content.len() > 72 {
        &content[..72]
    } else {
        content
    };
    format!("{:<72}{}{:>7}", trimmed, section, seq)
}

fn iges_wrap_param_data(text: &str, start_offset: usize, de_seq: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut remaining = text;
    let mut seq = start_offset + 1;

    while !remaining.is_empty() {
        let chunk_len = remaining.len().min(64);
        let chunk = &remaining[..chunk_len];
        remaining = &remaining[chunk_len..];

        let line = format!("{:<64}{:>8}P{:>7}", chunk, de_seq, seq);
        lines.push(line);
        seq += 1;
    }

    lines
}

fn iges_wrap_section_data(text: &str, section: char) -> Vec<String> {
    let mut lines = Vec::new();
    let mut remaining = text;
    let mut seq = 1;

    while !remaining.is_empty() {
        let chunk_len = remaining.len().min(72);
        let chunk = &remaining[..chunk_len];
        remaining = &remaining[chunk_len..];

        lines.push(format!("{:<72}{}{:>7}", chunk, section, seq));
        seq += 1;
    }

    if lines.is_empty() {
        lines.push(format!("{:<72}{}{:>7}", "", section, 1));
    }

    lines
}

fn build_global_section() -> String {
    let fields = vec![
        "1H,".to_string(),
        "1H;".to_string(),
        iges_hollerith("rusted-geom"),
        iges_hollerith("export.igs"),
        iges_hollerith("rusted-geom"),
        iges_hollerith("rusted-geom"),
        "32".to_string(),
        "38".to_string(),
        "15".to_string(),
        "308".to_string(),
        "15".to_string(),
        iges_hollerith("rusted-geom"),
        "1.0".to_string(),
        "6".to_string(),
        iges_hollerith("M"),
        "1".to_string(),
        "1.0".to_string(),
        iges_hollerith("20260304.120000"),
        "1.0E-06".to_string(),
        "1000.0".to_string(),
        iges_hollerith("rusted-geom"),
        iges_hollerith("rusted-geom"),
        "11".to_string(),
        "0".to_string(),
    ];
    fields.join(",") + ";"
}

// ─── Public entry point ──────────────────────────────────────────────────────

pub(crate) fn export_iges_text(
    session: RgmKernelHandle,
    object_ids: &[u64],
) -> Result<String, String> {
    let entry = SESSIONS
        .get(&session.0)
        .ok_or_else(|| "Session not found".to_string())?;
    let state = entry.value().read();

    let mut writer = IgesWriter::new();
    let stage_paths = collect_stage_subtree_paths(&state.stage, &collect_export_root_paths(&state, object_ids));

    for path in stage_paths {
        if let Some(curves_prim) = state.stage.get::<rusted_usd::schema::generated::UsdGeomNurbsCurves>(&path) {
            let world = world_transform_for_path(&state.stage, &path);
            for index in 0..curves_prim.curve_vertex_counts.len() {
                if let Ok(core) = nurbs_core_from_curves_prim(curves_prim, index) {
                    writer.add_entity_126(&transform_curve_core(&core, world));
                }
            }
            continue;
        }
        if let Some(patch_prim) = state.stage.get::<rusted_usd::schema::generated::UsdGeomNurbsPatch>(&path) {
            if let Ok(core) = nurbs_core_from_patch_prim(patch_prim) {
                let world = world_transform_for_path(&state.stage, &path);
                writer.add_entity_128(&core, &world);
            }
        }
    }

    Ok(writer.finish())
}
