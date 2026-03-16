use std::fmt::Write;

use crate::foundation::{GfMatrix4d, GfVec2d, GfVec3f, SdfPath};
use crate::schema::generated::*;
use crate::stage::{UsdPrim, UsdStage};

/// Serialize a `UsdStage` to USDA text format.
pub fn write_stage(stage: &UsdStage) -> String {
    let mut out = String::with_capacity(4096);
    let root = SdfPath::new("/");
    let root_prims = stage.children(&root);
    if let Some(first) = root_prims.first() {
        let name = first.name();
        let _ = write!(
            out,
            "#usda 1.0\n(\n    defaultPrim = \"{name}\"\n    metersPerUnit = 1\n    upAxis = \"Z\"\n)\n\n"
        );
    } else {
        out.push_str("#usda 1.0\n(\n    upAxis = \"Z\"\n)\n\n");
    }

    write_children(stage, &root, 0, &mut out);

    out
}

/// Serialize a subset of prims to USDA text format.
pub fn write_prims(stage: &UsdStage, paths: &[SdfPath]) -> String {
    let mut out = String::with_capacity(4096);
    out.push_str("#usda 1.0\n(\n    metersPerUnit = 1\n    upAxis = \"Z\"\n)\n\n");

    for path in paths {
        if let Some(prim) = stage.prim(path) {
            write_prim_recursive(stage, prim, 0, &mut out);
        }
    }

    out
}

fn write_children(stage: &UsdStage, parent: &SdfPath, depth: usize, out: &mut String) {
    for child_path in stage.children(parent) {
        if let Some(prim) = stage.prim(child_path) {
            write_prim_recursive(stage, prim, depth, out);
        }
    }
}

fn write_prim_recursive(stage: &UsdStage, prim: &UsdPrim, depth: usize, out: &mut String) {
    let indent = "    ".repeat(depth);
    let name = prim.path.name();
    let full_name = prim.schema.schema_name();
    let schema_name = full_name.strip_prefix("UsdGeom").unwrap_or(full_name);

    let _ = writeln!(out, "{indent}def {schema_name} \"{name}\"");
    let _ = writeln!(out, "{indent}{{");

    write_schema_attributes(&prim.schema, depth + 1, out);

    write_children(stage, &prim.path, depth + 1, out);

    let _ = writeln!(out, "{indent}}}");
    let _ = writeln!(out);
}

fn write_schema_attributes(schema: &SchemaData, depth: usize, out: &mut String) {
    match schema {
        SchemaData::Scope(_) => {}
        SchemaData::Mesh(m) => write_mesh(m, depth, out),
        SchemaData::NurbsPatch(p) => write_nurbs_patch(p, depth, out),
        SchemaData::NurbsCurves(c) => write_nurbs_curves(c, depth, out),
        SchemaData::BasisCurves(c) => write_basis_curves(c, depth, out),
        SchemaData::Points(p) => write_points(p, depth, out),
        SchemaData::Xform(x) => write_xform(x, depth, out),
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Per-type attribute writers
// ---------------------------------------------------------------------------

fn write_mesh(m: &UsdGeomMesh, depth: usize, out: &mut String) {
    let i = "    ".repeat(depth);

    if !m.face_vertex_counts.is_empty() {
        write_int_array(&i, "int[] faceVertexCounts", &m.face_vertex_counts, out);
    }
    if !m.face_vertex_indices.is_empty() {
        write_int_array(&i, "int[] faceVertexIndices", &m.face_vertex_indices, out);
    }
    if !m.points.is_empty() {
        write_point3f_array(&i, "point3f[] points", &m.points, out);
    }
    if !m.normals.is_empty() {
        write_point3f_array(&i, "normal3f[] normals", &m.normals, out);
    }
    if m.subdivision_scheme.as_str() != "catmullClark" {
        let _ = writeln!(
            out,
            "{i}uniform token subdivisionScheme = \"{}\"",
            m.subdivision_scheme
        );
    }
    if !m.extent.is_empty() {
        write_point3f_array(&i, "float3[] extent", &m.extent, out);
    }
}

fn write_nurbs_patch(p: &UsdGeomNurbsPatch, depth: usize, out: &mut String) {
    let i = "    ".repeat(depth);

    if let Some(u) = p.u_vertex_count {
        let _ = writeln!(out, "{i}int uVertexCount = {u}");
    }
    if let Some(v) = p.v_vertex_count {
        let _ = writeln!(out, "{i}int vVertexCount = {v}");
    }
    if let Some(u) = p.u_order {
        let _ = writeln!(out, "{i}int uOrder = {u}");
    }
    if let Some(v) = p.v_order {
        let _ = writeln!(out, "{i}int vOrder = {v}");
    }
    if !p.u_knots.is_empty() {
        write_double_array(&i, "double[] uKnots", &p.u_knots, out);
    }
    if !p.v_knots.is_empty() {
        write_double_array(&i, "double[] vKnots", &p.v_knots, out);
    }
    if let Some(ref r) = p.u_range {
        let _ = writeln!(out, "{i}double2 uRange = ({}, {})", fmt_f64(r.x), fmt_f64(r.y));
    }
    if let Some(ref r) = p.v_range {
        let _ = writeln!(out, "{i}double2 vRange = ({}, {})", fmt_f64(r.x), fmt_f64(r.y));
    }
    if !p.points.is_empty() {
        write_point3f_array(&i, "point3f[] points", &p.points, out);
    }
    if !p.point_weights.is_empty() {
        write_double_array(&i, "double[] pointWeights", &p.point_weights, out);
    }
    if p.u_form.as_str() != "open" {
        let _ = writeln!(out, "{i}uniform token uForm = \"{}\"", p.u_form);
    }
    if p.v_form.as_str() != "open" {
        let _ = writeln!(out, "{i}uniform token vForm = \"{}\"", p.v_form);
    }
    if !p.normals.is_empty() {
        write_point3f_array(&i, "normal3f[] normals", &p.normals, out);
    }
    if !p.extent.is_empty() {
        write_point3f_array(&i, "float3[] extent", &p.extent, out);
    }
}

fn write_nurbs_curves(c: &UsdGeomNurbsCurves, depth: usize, out: &mut String) {
    let i = "    ".repeat(depth);

    if !c.curve_vertex_counts.is_empty() {
        write_int_array(&i, "int[] curveVertexCounts", &c.curve_vertex_counts, out);
    }
    if !c.order.is_empty() {
        write_int_array(&i, "int[] order", &c.order, out);
    }
    if !c.knots.is_empty() {
        write_double_array(&i, "double[] knots", &c.knots, out);
    }
    if !c.ranges.is_empty() {
        write_double2_array(&i, "double2[] ranges", &c.ranges, out);
    }
    if !c.points.is_empty() {
        write_point3f_array(&i, "point3f[] points", &c.points, out);
    }
    if !c.point_weights.is_empty() {
        write_double_array(&i, "double[] pointWeights", &c.point_weights, out);
    }
    if !c.widths.is_empty() {
        write_float_array(&i, "float[] widths", &c.widths, out);
    }
    if !c.normals.is_empty() {
        write_point3f_array(&i, "normal3f[] normals", &c.normals, out);
    }
    if !c.extent.is_empty() {
        write_point3f_array(&i, "float3[] extent", &c.extent, out);
    }
}

fn write_basis_curves(c: &UsdGeomBasisCurves, depth: usize, out: &mut String) {
    let i = "    ".repeat(depth);

    if !c.curve_vertex_counts.is_empty() {
        write_int_array(&i, "int[] curveVertexCounts", &c.curve_vertex_counts, out);
    }
    if c.type_.as_str() != "cubic" {
        let _ = writeln!(out, "{i}uniform token type = \"{}\"", c.type_);
    }
    if c.basis.as_str() != "bezier" {
        let _ = writeln!(out, "{i}uniform token basis = \"{}\"", c.basis);
    }
    if c.wrap.as_str() != "nonperiodic" {
        let _ = writeln!(out, "{i}uniform token wrap = \"{}\"", c.wrap);
    }
    if !c.points.is_empty() {
        write_point3f_array(&i, "point3f[] points", &c.points, out);
    }
    if !c.widths.is_empty() {
        write_float_array(&i, "float[] widths", &c.widths, out);
    }
    if !c.normals.is_empty() {
        write_point3f_array(&i, "normal3f[] normals", &c.normals, out);
    }
    if !c.extent.is_empty() {
        write_point3f_array(&i, "float3[] extent", &c.extent, out);
    }
}

fn write_points(p: &UsdGeomPoints, depth: usize, out: &mut String) {
    let i = "    ".repeat(depth);

    if !p.points.is_empty() {
        write_point3f_array(&i, "point3f[] points", &p.points, out);
    }
    if !p.widths.is_empty() {
        write_float_array(&i, "float[] widths", &p.widths, out);
    }
    if !p.ids.is_empty() {
        let _ = write!(out, "{i}int64[] ids = [");
        for (j, id) in p.ids.iter().enumerate() {
            if j > 0 {
                out.push_str(", ");
            }
            let _ = write!(out, "{id}");
        }
        let _ = writeln!(out, "]");
    }
    if !p.normals.is_empty() {
        write_point3f_array(&i, "normal3f[] normals", &p.normals, out);
    }
    if !p.extent.is_empty() {
        write_point3f_array(&i, "float3[] extent", &p.extent, out);
    }
}

fn write_xform(x: &UsdGeomXform, depth: usize, out: &mut String) {
    let i = "    ".repeat(depth);
    if let Some(matrix) = x.xform_op_transform {
        write_matrix4d(&i, "matrix4d xformOp:transform", &matrix, out);
    }
    if !x.xform_op_order.is_empty() {
        let _ = write!(out, "{i}uniform token[] xformOpOrder = [");
        for (j, tok) in x.xform_op_order.iter().enumerate() {
            if j > 0 {
                out.push_str(", ");
            }
            let _ = write!(out, "\"{}\"", tok);
        }
        let _ = writeln!(out, "]");
    }
}

fn write_matrix4d(indent: &str, decl: &str, matrix: &GfMatrix4d, out: &mut String) {
    let _ = write!(out, "{indent}{decl} = (");
    for (row_index, row) in matrix.iter().enumerate() {
        if row_index > 0 {
            out.push_str(", ");
        }
        let _ = write!(
            out,
            "({}, {}, {}, {})",
            fmt_f64(row[0]),
            fmt_f64(row[1]),
            fmt_f64(row[2]),
            fmt_f64(row[3]),
        );
    }
    let _ = writeln!(out, ")");
}

// ---------------------------------------------------------------------------
// Primitive serialization helpers
// ---------------------------------------------------------------------------

fn write_int_array(indent: &str, decl: &str, values: &[i32], out: &mut String) {
    let _ = write!(out, "{indent}{decl} = [");
    for (j, v) in values.iter().enumerate() {
        if j > 0 {
            out.push_str(", ");
        }
        let _ = write!(out, "{v}");
    }
    let _ = writeln!(out, "]");
}

/// Format a float so it always contains a decimal point (USDA requires float literals).
/// Rust's `{}` for round values like `1.0` produces `"1"`, which some parsers reject.
#[inline]
fn fmt_f32(v: f32) -> String {
    let s = format!("{v}");
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        s + ".0"
    }
}

#[inline]
fn fmt_f64(v: f64) -> String {
    let s = format!("{v}");
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        s + ".0"
    }
}

fn write_float_array(indent: &str, decl: &str, values: &[f32], out: &mut String) {
    let _ = write!(out, "{indent}{decl} = [");
    for (j, v) in values.iter().enumerate() {
        if j > 0 {
            out.push_str(", ");
        }
        out.push_str(&fmt_f32(*v));
    }
    let _ = writeln!(out, "]");
}

fn write_double_array(indent: &str, decl: &str, values: &[f64], out: &mut String) {
    let _ = write!(out, "{indent}{decl} = [");
    for (j, v) in values.iter().enumerate() {
        if j > 0 {
            out.push_str(", ");
        }
        out.push_str(&fmt_f64(*v));
    }
    let _ = writeln!(out, "]");
}

fn write_point3f_array(indent: &str, decl: &str, values: &[GfVec3f], out: &mut String) {
    let _ = write!(out, "{indent}{decl} = [");
    for (j, v) in values.iter().enumerate() {
        if j > 0 {
            out.push_str(", ");
        }
        let _ = write!(out, "({}, {}, {})", fmt_f32(v.x), fmt_f32(v.y), fmt_f32(v.z));
    }
    let _ = writeln!(out, "]");
}

fn write_double2_array(indent: &str, decl: &str, values: &[GfVec2d], out: &mut String) {
    let _ = write!(out, "{indent}{decl} = [");
    for (j, v) in values.iter().enumerate() {
        if j > 0 {
            out.push_str(", ");
        }
        let _ = write!(out, "({}, {})", fmt_f64(v.x), fmt_f64(v.y));
    }
    let _ = writeln!(out, "]");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::TfToken;

    #[test]
    fn write_simple_mesh() {
        let mut stage = UsdStage::new();

        let mut mesh = UsdGeomMesh::default();
        mesh.points = vec![
            GfVec3f::new(0.0, 0.0, 0.0),
            GfVec3f::new(1.0, 0.0, 0.0),
            GfVec3f::new(1.0, 1.0, 0.0),
            GfVec3f::new(0.0, 1.0, 0.0),
        ];
        mesh.face_vertex_counts = vec![4];
        mesh.face_vertex_indices = vec![0, 1, 2, 3];
        mesh.subdivision_scheme = TfToken::new("none");

        stage.define_prim(
            SdfPath::new("/Meshes/Quad"),
            SchemaData::Mesh(mesh),
        );

        let usda = write_stage(&stage);
        assert!(usda.contains("def Mesh \"Quad\""));
        assert!(usda.contains("point3f[] points"));
        assert!(usda.contains("int[] faceVertexCounts = [4]"));
        assert!(usda.contains("uniform token subdivisionScheme = \"none\""));
    }

    #[test]
    fn write_nurbs_curve() {
        let mut stage = UsdStage::new();

        let mut curves = UsdGeomNurbsCurves::default();
        curves.curve_vertex_counts = vec![4];
        curves.order = vec![4];
        curves.knots = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
        curves.points = vec![
            GfVec3f::new(0.0, 0.0, 0.0),
            GfVec3f::new(1.0, 1.0, 0.0),
            GfVec3f::new(2.0, 1.0, 0.0),
            GfVec3f::new(3.0, 0.0, 0.0),
        ];
        curves.widths = vec![0.01, 0.01, 0.01, 0.01];
        curves.ranges = vec![crate::foundation::GfVec2d::new(0.0, 1.0)];
        curves.extent = vec![GfVec3f::new(0.0, 0.0, 0.0), GfVec3f::new(3.0, 1.0, 0.0)];

        stage.define_prim(
            SdfPath::new("/Curves/Bezier"),
            SchemaData::NurbsCurves(curves),
        );

        let usda = write_stage(&stage);
        assert!(usda.contains("def NurbsCurves \"Bezier\""));
        assert!(usda.contains("int[] order = [4]"));
        assert!(usda.contains("double[] knots"));
    }

    #[test]
    fn write_nurbs_patch() {
        let mut stage = UsdStage::new();

        let mut patch = UsdGeomNurbsPatch::default();
        patch.u_vertex_count = Some(2);
        patch.v_vertex_count = Some(2);
        patch.u_order = Some(2);
        patch.v_order = Some(2);
        patch.u_knots = vec![0.0, 0.0, 1.0, 1.0];
        patch.v_knots = vec![0.0, 0.0, 1.0, 1.0];
        patch.u_range = Some(crate::foundation::GfVec2d::new(0.0, 1.0));
        patch.v_range = Some(crate::foundation::GfVec2d::new(0.0, 1.0));
        patch.points = vec![
            GfVec3f::new(0.0, 0.0, 0.0),
            GfVec3f::new(0.0, 1.0, 0.0),
            GfVec3f::new(1.0, 0.0, 0.0),
            GfVec3f::new(1.0, 1.0, 0.0),
        ];
        patch.extent = vec![GfVec3f::new(0.0, 0.0, 0.0), GfVec3f::new(1.0, 1.0, 0.0)];

        stage.define_prim(
            SdfPath::new("/Surfaces/Bilinear"),
            SchemaData::NurbsPatch(patch),
        );

        let usda = write_stage(&stage);
        assert!(usda.contains("def NurbsPatch \"Bilinear\""));
        assert!(usda.contains("int uVertexCount = 2"));
        assert!(usda.contains("double[] uKnots"));
    }

    #[test]
    fn write_xform_matrix_op() {
        let mut stage = UsdStage::new();
        let mut xform = UsdGeomXform::default();
        xform.xform_op_transform = Some([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 5.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        xform.xform_op_order = vec![TfToken::new("xformOp:transform")];
        stage.define_prim(SdfPath::new("/World/Part"), SchemaData::Xform(xform));

        let usda = write_stage(&stage);
        assert!(usda.contains("def Xform \"Part\""));
        assert!(usda.contains("matrix4d xformOp:transform"));
        assert!(usda.contains("\"xformOp:transform\""));
    }
}
