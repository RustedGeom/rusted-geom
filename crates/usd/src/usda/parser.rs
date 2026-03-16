use std::collections::HashMap;

use crate::foundation::{GfMatrix4d, GfVec2d, GfVec3f, SdfPath, TfToken};
use crate::schema::generated::*;
use crate::stage::UsdStage;

#[derive(Debug)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}

/// Parse a USDA text string and return a populated `UsdStage`.
pub fn parse_usda(input: &str) -> Result<UsdStage, ParseError> {
    let mut p = UsdaParser::new(input);
    p.parse_file()
}

struct UsdaParser {
    chars: Vec<char>,
    pos: usize,
}

impl UsdaParser {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.advance();
                }
                Some('#') => {
                    while let Some(c) = self.advance() {
                        if c == '\n' {
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
    }

    fn expect_char(&mut self, ch: char) -> Result<(), ParseError> {
        self.skip_whitespace_and_comments();
        match self.advance() {
            Some(c) if c == ch => Ok(()),
            Some(c) => Err(ParseError(format!("expected '{}', got '{}' at {}", ch, c, self.pos))),
            None => Err(ParseError(format!("expected '{}', got EOF", ch))),
        }
    }

    fn read_word(&mut self) -> String {
        self.skip_whitespace_and_comments();
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == ':' || c == '.' {
                self.advance();
            } else {
                break;
            }
        }
        self.chars[start..self.pos].iter().collect()
    }

    fn read_quoted(&mut self) -> Result<String, ParseError> {
        self.skip_whitespace_and_comments();
        if self.peek() != Some('"') {
            return Err(ParseError(format!("expected '\"' at {}", self.pos)));
        }
        self.advance();
        let mut result = String::new();
        loop {
            match self.advance() {
                Some('"') => return Ok(result),
                Some('\\') => {
                    if let Some(c2) = self.advance() {
                        result.push(c2);
                    }
                }
                Some(c) => result.push(c),
                None => return Err(ParseError("unterminated string".into())),
            }
        }
    }

    fn skip_parens(&mut self) -> Result<(), ParseError> {
        self.expect_char('(')?;
        let mut depth = 1;
        while depth > 0 {
            match self.advance() {
                Some('(') => depth += 1,
                Some(')') => depth -= 1,
                Some('"') => {
                    loop {
                        match self.advance() {
                            Some('"') => break,
                            Some('\\') => { self.advance(); }
                            None => return Err(ParseError("unterminated string".into())),
                            _ => {}
                        }
                    }
                }
                None => return Err(ParseError("unterminated parens".into())),
                _ => {}
            }
        }
        Ok(())
    }

    fn parse_file(&mut self) -> Result<UsdStage, ParseError> {
        let mut stage = UsdStage::new();

        self.skip_whitespace_and_comments();

        // Skip file header
        let first = self.read_word();
        if first == "#usda" || first.starts_with('#') {
            while let Some(c) = self.peek() {
                if c == '\n' {
                    self.advance();
                    break;
                }
                self.advance();
            }
        }

        // Skip metadata block
        self.skip_whitespace_and_comments();
        if self.peek() == Some('(') {
            self.skip_parens()?;
        }

        // Parse top-level prims
        self.skip_whitespace_and_comments();
        while self.peek().is_some() {
            self.skip_whitespace_and_comments();
            if self.peek().is_none() {
                break;
            }
            let word = self.read_word();
            match word.as_str() {
                "def" => {
                    self.parse_prim_def(&mut stage, &SdfPath::new("/"))?;
                }
                "over" | "class" => {
                    self.skip_prim_block()?;
                }
                "" => {
                    self.advance();
                }
                _ => {
                    // Skip unknown top-level content
                }
            }
        }

        Ok(stage)
    }

    fn parse_prim_def(&mut self, stage: &mut UsdStage, parent: &SdfPath) -> Result<(), ParseError> {
        self.skip_whitespace_and_comments();
        let schema_type = self.read_word();
        self.skip_whitespace_and_comments();
        let prim_name = self.read_quoted()?;

        let path = parent.child(&prim_name);

        // Skip optional metadata
        self.skip_whitespace_and_comments();
        if self.peek() == Some('(') {
            self.skip_parens()?;
        }

        self.skip_whitespace_and_comments();
        self.expect_char('{')?;

        let mut attrs: HashMap<String, AttrValue> = HashMap::new();

        loop {
            self.skip_whitespace_and_comments();
            if self.peek() == Some('}') {
                self.advance();
                break;
            }
            if self.peek().is_none() {
                return Err(ParseError("unterminated prim block".into()));
            }

            // Check if this is a nested def
            let word = self.read_word();

            if word == "def" {
                self.parse_prim_def(stage, &path)?;
                continue;
            }
            if word == "over" || word == "class" {
                self.skip_prim_block()?;
                continue;
            }

            // Parse attribute: [uniform] type name [= value]
            let is_uniform = word == "uniform";
            let type_str = if is_uniform {
                let mut t = self.read_word();
                self.skip_whitespace_and_comments();
                if self.peek() == Some('[') {
                    self.advance();
                    self.expect_char(']')?;
                    t.push_str("[]");
                }
                t
            } else {
                let mut t = word;
                self.skip_whitespace_and_comments();
                if self.peek() == Some('[') {
                    self.advance();
                    self.expect_char(']')?;
                    t.push_str("[]");
                }
                t
            };

            if type_str.is_empty() {
                self.advance();
                continue;
            }

            let attr_name = self.read_word();
            if attr_name.is_empty() {
                continue;
            }

            self.skip_whitespace_and_comments();
            if self.peek() == Some('=') {
                self.advance();
                self.skip_whitespace_and_comments();
                let value = self.parse_value(&type_str)?;
                attrs.insert(attr_name, value);
            }

            // Skip trailing metadata
            self.skip_whitespace_and_comments();
            if self.peek() == Some('(') {
                self.skip_parens()?;
            }
        }

        let schema = build_schema_data(&schema_type, &attrs);
        stage.define_prim(path, schema);

        Ok(())
    }

    fn skip_prim_block(&mut self) -> Result<(), ParseError> {
        self.skip_whitespace_and_comments();
        // Skip type name
        let _ = self.read_word();
        self.skip_whitespace_and_comments();
        // Skip prim name
        if self.peek() == Some('"') {
            let _ = self.read_quoted()?;
        }
        // Skip metadata
        self.skip_whitespace_and_comments();
        if self.peek() == Some('(') {
            self.skip_parens()?;
        }
        // Skip body
        self.skip_whitespace_and_comments();
        if self.peek() == Some('{') {
            self.expect_char('{')?;
            let mut depth = 1;
            while depth > 0 {
                match self.advance() {
                    Some('{') => depth += 1,
                    Some('}') => depth -= 1,
                    Some('"') => {
                        loop {
                            match self.advance() {
                                Some('"') => break,
                                Some('\\') => { self.advance(); }
                                None => break,
                                _ => {}
                            }
                        }
                    }
                    None => break,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn parse_value(&mut self, type_str: &str) -> Result<AttrValue, ParseError> {
        self.skip_whitespace_and_comments();

        match type_str {
            "int" => {
                let v = self.parse_number()?;
                Ok(AttrValue::Int(v as i32))
            }
            "int[]" => {
                let arr = self.parse_int_array()?;
                Ok(AttrValue::IntArray(arr))
            }
            "int64[]" => {
                let arr = self.parse_int_array()?;
                Ok(AttrValue::Int64Array(arr.into_iter().map(|v| v as i64).collect()))
            }
            "float" | "float[]" if type_str.ends_with("[]") => {
                let arr = self.parse_float_array()?;
                Ok(AttrValue::FloatArray(arr.into_iter().map(|v| v as f32).collect()))
            }
            "float" => {
                let v = self.parse_number()?;
                Ok(AttrValue::Float(v as f32))
            }
            "double" => {
                let v = self.parse_number()?;
                Ok(AttrValue::Double(v))
            }
            "double[]" => {
                let arr = self.parse_float_array()?;
                Ok(AttrValue::DoubleArray(arr))
            }
            "double2" => {
                let (a, b) = self.parse_tuple2()?;
                Ok(AttrValue::Double2(GfVec2d::new(a, b)))
            }
            "double2[]" => {
                let arr = self.parse_tuple2_array()?;
                Ok(AttrValue::Double2Array(arr))
            }
            "matrix4d" => {
                let matrix = self.parse_matrix4d()?;
                Ok(AttrValue::Matrix4d(matrix))
            }
            "point3f[]" | "normal3f[]" | "vector3f[]" | "float3[]" | "color3f[]" => {
                let arr = self.parse_tuple3_array()?;
                Ok(AttrValue::Vec3fArray(arr))
            }
            "token" | "token[]" if type_str.ends_with("[]") => {
                let arr = self.parse_token_array()?;
                Ok(AttrValue::TokenArray(arr))
            }
            "token" => {
                let s = self.read_quoted()?;
                Ok(AttrValue::Token(TfToken::new(&s)))
            }
            "string" => {
                let s = self.read_quoted()?;
                Ok(AttrValue::Str(s))
            }
            "bool" => {
                let w = self.read_word();
                Ok(AttrValue::Bool(w == "1" || w == "true"))
            }
            _ => {
                // Unknown type, try to skip value
                self.skip_value()?;
                Ok(AttrValue::Unknown)
            }
        }
    }

    fn parse_number(&mut self) -> Result<f64, ParseError> {
        self.skip_whitespace_and_comments();
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E' {
                self.advance();
            } else {
                break;
            }
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        s.parse::<f64>().map_err(|_| ParseError(format!("invalid number: {s}")))
    }

    fn parse_int_array(&mut self) -> Result<Vec<i32>, ParseError> {
        self.expect_char('[')?;
        let mut values = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.peek() == Some(']') {
                self.advance();
                break;
            }
            let v = self.parse_number()?;
            values.push(v as i32);
            self.skip_whitespace_and_comments();
            if self.peek() == Some(',') {
                self.advance();
            }
        }
        Ok(values)
    }

    fn parse_float_array(&mut self) -> Result<Vec<f64>, ParseError> {
        self.expect_char('[')?;
        let mut values = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.peek() == Some(']') {
                self.advance();
                break;
            }
            let v = self.parse_number()?;
            values.push(v);
            self.skip_whitespace_and_comments();
            if self.peek() == Some(',') {
                self.advance();
            }
        }
        Ok(values)
    }

    fn parse_tuple2(&mut self) -> Result<(f64, f64), ParseError> {
        self.expect_char('(')?;
        let a = self.parse_number()?;
        self.skip_whitespace_and_comments();
        if self.peek() == Some(',') {
            self.advance();
        }
        let b = self.parse_number()?;
        self.skip_whitespace_and_comments();
        self.expect_char(')')?;
        Ok((a, b))
    }

    fn parse_tuple2_array(&mut self) -> Result<Vec<GfVec2d>, ParseError> {
        self.expect_char('[')?;
        let mut values = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.peek() == Some(']') {
                self.advance();
                break;
            }
            let (a, b) = self.parse_tuple2()?;
            values.push(GfVec2d::new(a, b));
            self.skip_whitespace_and_comments();
            if self.peek() == Some(',') {
                self.advance();
            }
        }
        Ok(values)
    }

    fn parse_tuple4(&mut self) -> Result<[f64; 4], ParseError> {
        self.expect_char('(')?;
        let mut out = [0.0; 4];
        for (index, slot) in out.iter_mut().enumerate() {
            *slot = self.parse_number()?;
            self.skip_whitespace_and_comments();
            if index < 3 && self.peek() == Some(',') {
                self.advance();
            }
        }
        self.skip_whitespace_and_comments();
        self.expect_char(')')?;
        Ok(out)
    }

    fn parse_matrix4d(&mut self) -> Result<GfMatrix4d, ParseError> {
        self.expect_char('(')?;
        let mut rows = [[0.0; 4]; 4];
        for (index, row) in rows.iter_mut().enumerate() {
            *row = self.parse_tuple4()?;
            self.skip_whitespace_and_comments();
            if index < 3 && self.peek() == Some(',') {
                self.advance();
            }
        }
        self.skip_whitespace_and_comments();
        self.expect_char(')')?;
        Ok(rows)
    }

    fn parse_tuple3_array(&mut self) -> Result<Vec<GfVec3f>, ParseError> {
        self.expect_char('[')?;
        let mut values = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.peek() == Some(']') {
                self.advance();
                break;
            }
            self.expect_char('(')?;
            let x = self.parse_number()? as f32;
            self.skip_whitespace_and_comments();
            if self.peek() == Some(',') {
                self.advance();
            }
            let y = self.parse_number()? as f32;
            self.skip_whitespace_and_comments();
            if self.peek() == Some(',') {
                self.advance();
            }
            let z = self.parse_number()? as f32;
            self.skip_whitespace_and_comments();
            self.expect_char(')')?;
            values.push(GfVec3f::new(x, y, z));
            self.skip_whitespace_and_comments();
            if self.peek() == Some(',') {
                self.advance();
            }
        }
        Ok(values)
    }

    fn parse_token_array(&mut self) -> Result<Vec<TfToken>, ParseError> {
        self.expect_char('[')?;
        let mut values = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.peek() == Some(']') {
                self.advance();
                break;
            }
            let s = self.read_quoted()?;
            values.push(TfToken::new(&s));
            self.skip_whitespace_and_comments();
            if self.peek() == Some(',') {
                self.advance();
            }
        }
        Ok(values)
    }

    fn skip_value(&mut self) -> Result<(), ParseError> {
        self.skip_whitespace_and_comments();
        match self.peek() {
            Some('[') => {
                self.advance();
                let mut depth = 1;
                while depth > 0 {
                    match self.advance() {
                        Some('[') => depth += 1,
                        Some(']') => depth -= 1,
                        None => break,
                        _ => {}
                    }
                }
            }
            Some('(') => {
                self.advance();
                let mut depth = 1;
                while depth > 0 {
                    match self.advance() {
                        Some('(') => depth += 1,
                        Some(')') => depth -= 1,
                        None => break,
                        _ => {}
                    }
                }
            }
            Some('"') => {
                let _ = self.read_quoted()?;
            }
            _ => {
                while let Some(c) = self.peek() {
                    if c.is_whitespace() || c == ')' || c == '}' || c == '(' {
                        break;
                    }
                    self.advance();
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Attribute value IR and schema builder
// ---------------------------------------------------------------------------

#[allow(dead_code)]
enum AttrValue {
    Int(i32),
    IntArray(Vec<i32>),
    Int64Array(Vec<i64>),
    Float(f32),
    FloatArray(Vec<f32>),
    Double(f64),
    DoubleArray(Vec<f64>),
    Double2(GfVec2d),
    Double2Array(Vec<GfVec2d>),
    Matrix4d(GfMatrix4d),
    Vec3fArray(Vec<GfVec3f>),
    Token(TfToken),
    TokenArray(Vec<TfToken>),
    Str(String),
    Bool(bool),
    Unknown,
}

fn build_schema_data(schema_type: &str, attrs: &HashMap<String, AttrValue>) -> SchemaData {
    match schema_type {
        "Mesh" => SchemaData::Mesh(build_mesh(attrs)),
        "NurbsCurves" => SchemaData::NurbsCurves(build_nurbs_curves(attrs)),
        "NurbsPatch" => SchemaData::NurbsPatch(build_nurbs_patch(attrs)),
        "BasisCurves" => SchemaData::BasisCurves(build_basis_curves(attrs)),
        "Points" => SchemaData::Points(build_points(attrs)),
        "Xform" => SchemaData::Xform(build_xform(attrs)),
        "Scope" | _ => SchemaData::Scope(UsdGeomScope),
    }
}

fn get_int_array(attrs: &HashMap<String, AttrValue>, key: &str) -> Vec<i32> {
    match attrs.get(key) {
        Some(AttrValue::IntArray(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn get_double_array(attrs: &HashMap<String, AttrValue>, key: &str) -> Vec<f64> {
    match attrs.get(key) {
        Some(AttrValue::DoubleArray(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn get_vec3f_array(attrs: &HashMap<String, AttrValue>, key: &str) -> Vec<GfVec3f> {
    match attrs.get(key) {
        Some(AttrValue::Vec3fArray(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn get_double2_array(attrs: &HashMap<String, AttrValue>, key: &str) -> Vec<GfVec2d> {
    match attrs.get(key) {
        Some(AttrValue::Double2Array(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn get_token(attrs: &HashMap<String, AttrValue>, key: &str) -> Option<TfToken> {
    match attrs.get(key) {
        Some(AttrValue::Token(t)) => Some(t.clone()),
        _ => None,
    }
}

fn get_token_array(attrs: &HashMap<String, AttrValue>, key: &str) -> Vec<TfToken> {
    match attrs.get(key) {
        Some(AttrValue::TokenArray(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn get_int(attrs: &HashMap<String, AttrValue>, key: &str) -> Option<i32> {
    match attrs.get(key) {
        Some(AttrValue::Int(v)) => Some(*v),
        _ => None,
    }
}

fn get_double2(attrs: &HashMap<String, AttrValue>, key: &str) -> Option<GfVec2d> {
    match attrs.get(key) {
        Some(AttrValue::Double2(v)) => Some(*v),
        _ => None,
    }
}

fn get_matrix4d(attrs: &HashMap<String, AttrValue>, key: &str) -> Option<GfMatrix4d> {
    match attrs.get(key) {
        Some(AttrValue::Matrix4d(v)) => Some(*v),
        _ => None,
    }
}

fn build_xform(attrs: &HashMap<String, AttrValue>) -> UsdGeomXform {
    let mut x = UsdGeomXform::default();
    if let Some(t) = get_token(attrs, "visibility") {
        x.visibility = t;
    }
    if let Some(t) = get_token(attrs, "purpose") {
        x.purpose = t;
    }
    x.xform_op_transform = get_matrix4d(attrs, "xformOp:transform");
    x.xform_op_order = get_token_array(attrs, "xformOpOrder");
    x
}

fn build_mesh(attrs: &HashMap<String, AttrValue>) -> UsdGeomMesh {
    let mut m = UsdGeomMesh::default();
    m.face_vertex_counts = get_int_array(attrs, "faceVertexCounts");
    m.face_vertex_indices = get_int_array(attrs, "faceVertexIndices");
    m.points = get_vec3f_array(attrs, "points");
    m.normals = get_vec3f_array(attrs, "normals");
    if let Some(t) = get_token(attrs, "subdivisionScheme") {
        m.subdivision_scheme = t;
    }
    m
}

fn build_nurbs_curves(attrs: &HashMap<String, AttrValue>) -> UsdGeomNurbsCurves {
    let mut c = UsdGeomNurbsCurves::default();
    c.curve_vertex_counts = get_int_array(attrs, "curveVertexCounts");
    c.order = get_int_array(attrs, "order");
    c.knots = get_double_array(attrs, "knots");
    c.ranges = get_double2_array(attrs, "ranges");
    c.points = get_vec3f_array(attrs, "points");
    c.point_weights = get_double_array(attrs, "pointWeights");
    c
}

fn build_nurbs_patch(attrs: &HashMap<String, AttrValue>) -> UsdGeomNurbsPatch {
    let mut p = UsdGeomNurbsPatch::default();
    p.u_vertex_count = get_int(attrs, "uVertexCount");
    p.v_vertex_count = get_int(attrs, "vVertexCount");
    p.u_order = get_int(attrs, "uOrder");
    p.v_order = get_int(attrs, "vOrder");
    p.u_knots = get_double_array(attrs, "uKnots");
    p.v_knots = get_double_array(attrs, "vKnots");
    p.u_range = get_double2(attrs, "uRange");
    p.v_range = get_double2(attrs, "vRange");
    p.points = get_vec3f_array(attrs, "points");
    p.point_weights = get_double_array(attrs, "pointWeights");
    if let Some(t) = get_token(attrs, "uForm") {
        p.u_form = t;
    }
    if let Some(t) = get_token(attrs, "vForm") {
        p.v_form = t;
    }
    p
}

fn build_basis_curves(attrs: &HashMap<String, AttrValue>) -> UsdGeomBasisCurves {
    let mut c = UsdGeomBasisCurves::default();
    c.curve_vertex_counts = get_int_array(attrs, "curveVertexCounts");
    c.points = get_vec3f_array(attrs, "points");
    if let Some(t) = get_token(attrs, "type") {
        c.type_ = t;
    }
    if let Some(t) = get_token(attrs, "basis") {
        c.basis = t;
    }
    if let Some(t) = get_token(attrs, "wrap") {
        c.wrap = t;
    }
    c
}

fn build_points(attrs: &HashMap<String, AttrValue>) -> UsdGeomPoints {
    let mut p = UsdGeomPoints::default();
    p.points = get_vec3f_array(attrs, "points");
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_nurbs_curves() {
        let mut stage = UsdStage::new();
        let mut curves = UsdGeomNurbsCurves::default();
        curves.curve_vertex_counts = vec![4];
        curves.order = vec![4];
        curves.knots = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
        curves.ranges = vec![GfVec2d::new(0.0, 1.0)];
        curves.points = vec![
            GfVec3f::new(0.0, 0.0, 0.0),
            GfVec3f::new(1.0, 2.0, 0.0),
            GfVec3f::new(3.0, 2.0, 0.0),
            GfVec3f::new(4.0, 0.0, 0.0),
        ];

        let path = SdfPath::new("/Curves/TestCurve");
        stage.define_prim(path.clone(), SchemaData::NurbsCurves(curves));

        let usda = crate::usda::writer::write_stage(&stage);

        let parsed = parse_usda(&usda).expect("parse failed");
        let restored = parsed
            .get::<UsdGeomNurbsCurves>(&SdfPath::new("/Curves/TestCurve"))
            .expect("prim not found");

        assert_eq!(restored.curve_vertex_counts, vec![4]);
        assert_eq!(restored.order, vec![4]);
        assert_eq!(restored.knots.len(), 8);
        assert_eq!(restored.points.len(), 4);
    }

    #[test]
    fn round_trip_mesh() {
        let mut stage = UsdStage::new();
        let mut mesh = UsdGeomMesh::default();
        mesh.points = vec![
            GfVec3f::new(0.0, 0.0, 0.0),
            GfVec3f::new(1.0, 0.0, 0.0),
            GfVec3f::new(1.0, 1.0, 0.0),
        ];
        mesh.face_vertex_counts = vec![3];
        mesh.face_vertex_indices = vec![0, 1, 2];
        mesh.subdivision_scheme = TfToken::new("none");

        let path = SdfPath::new("/Meshes/Tri");
        stage.define_prim(path.clone(), SchemaData::Mesh(mesh));

        let usda = crate::usda::writer::write_stage(&stage);

        let parsed = parse_usda(&usda).expect("parse failed");
        let restored = parsed
            .get::<UsdGeomMesh>(&SdfPath::new("/Meshes/Tri"))
            .expect("prim not found");

        assert_eq!(restored.face_vertex_counts, vec![3]);
        assert_eq!(restored.face_vertex_indices, vec![0, 1, 2]);
        assert_eq!(restored.points.len(), 3);
    }

    #[test]
    fn round_trip_xform_matrix() {
        let mut stage = UsdStage::new();
        let mut xform = UsdGeomXform::default();
        xform.xform_op_transform = Some([
            [1.0, 0.0, 0.0, 2.5],
            [0.0, 1.0, 0.0, -1.0],
            [0.0, 0.0, 1.0, 5.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        xform.xform_op_order = vec![TfToken::new("xformOp:transform")];

        stage.define_prim(SdfPath::new("/World/Part"), SchemaData::Xform(xform));

        let usda = crate::usda::writer::write_stage(&stage);
        let parsed = parse_usda(&usda).expect("parse failed");
        let restored = parsed
            .get::<UsdGeomXform>(&SdfPath::new("/World/Part"))
            .expect("xform not found");

        assert_eq!(
            restored.xform_op_order.iter().map(TfToken::as_str).collect::<Vec<_>>(),
            vec!["xformOp:transform"]
        );
        let matrix = restored.xform_op_transform.expect("xform matrix");
        assert!((matrix[0][3] - 2.5).abs() < 1e-9);
        assert!((matrix[2][3] - 5.0).abs() < 1e-9);
    }
}
