use anyhow::{Context, Result};
use heck::ToSnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Schema IR
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Property {
    name: String,
    usd_type: String,
    is_uniform: bool,
    is_array: bool,
    default_value: Option<String>,
    allowed_tokens: Vec<String>,
    doc: String,
    is_relationship: bool,
}

#[derive(Debug, Clone)]
struct SchemaClass {
    name: String,
    is_concrete: bool,
    inherits: Option<String>,
    doc: String,
    properties: Vec<Property>,
}

// ---------------------------------------------------------------------------
// Parser — hand-rolled recursive descent for schema.usda
// ---------------------------------------------------------------------------

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
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

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else if c == '#' {
                while let Some(c2) = self.advance() {
                    if c2 == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<()> {
        self.skip_whitespace();
        match self.advance() {
            Some(c) if c == expected => Ok(()),
            Some(c) => anyhow::bail!(
                "expected '{}', got '{}' at pos {}",
                expected,
                c,
                self.pos
            ),
            None => anyhow::bail!("expected '{}', got EOF", expected),
        }
    }

    fn read_quoted_string(&mut self) -> Result<String> {
        self.skip_whitespace();
        let mut result = String::new();
        let triple = self.check_triple_quote();
        if triple {
            loop {
                match self.advance() {
                    Some('"') => {
                        if self.peek() == Some('"')
                            && self.chars.get(self.pos + 1) == Some(&'"')
                        {
                            self.advance();
                            self.advance();
                            break;
                        } else {
                            result.push('"');
                        }
                    }
                    Some(c) => result.push(c),
                    None => anyhow::bail!("unterminated triple-quoted string"),
                }
            }
        } else {
            self.expect_char('"')?;
            loop {
                match self.advance() {
                    Some('"') => break,
                    Some('\\') => {
                        if let Some(c2) = self.advance() {
                            result.push(c2);
                        }
                    }
                    Some(c) => result.push(c),
                    None => anyhow::bail!("unterminated string"),
                }
            }
        }
        Ok(result)
    }

    fn check_triple_quote(&mut self) -> bool {
        if self.peek() == Some('"')
            && self.chars.get(self.pos + 1) == Some(&'"')
            && self.chars.get(self.pos + 2) == Some(&'"')
        {
            self.pos += 3;
            true
        } else {
            false
        }
    }

    fn read_identifier(&mut self) -> Result<String> {
        self.skip_whitespace();
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == ':' {
                self.advance();
            } else {
                break;
            }
        }
        if self.pos == start {
            anyhow::bail!(
                "expected identifier at pos {}, got {:?}",
                self.pos,
                self.peek()
            );
        }
        Ok(self.chars[start..self.pos].iter().collect())
    }

    fn skip_balanced_parens(&mut self) -> Result<String> {
        self.skip_whitespace();
        self.expect_char('(')?;
        let mut depth = 1;
        let start = self.pos;
        while depth > 0 {
            match self.advance() {
                Some('(') => depth += 1,
                Some(')') => depth -= 1,
                Some('"') => {
                    if self.chars.get(self.pos) == Some(&'"')
                        && self.chars.get(self.pos + 1) == Some(&'"')
                    {
                        self.pos += 2;
                        loop {
                            match self.advance() {
                                Some('"') => {
                                    if self.peek() == Some('"')
                                        && self.chars.get(self.pos + 1) == Some(&'"')
                                    {
                                        self.advance();
                                        self.advance();
                                        break;
                                    }
                                }
                                None => break,
                                _ => {}
                            }
                        }
                    } else {
                        loop {
                            match self.advance() {
                                Some('"') => break,
                                Some('\\') => {
                                    self.advance();
                                }
                                None => break,
                                _ => {}
                            }
                        }
                    }
                }
                None => anyhow::bail!("unterminated parenthesized block"),
                _ => {}
            }
        }
        Ok(self.chars[start..self.pos - 1].iter().collect())
    }

    fn skip_balanced_braces(&mut self) -> Result<()> {
        self.skip_whitespace();
        self.expect_char('{')?;
        let mut depth = 1;
        while depth > 0 {
            match self.advance() {
                Some('{') => depth += 1,
                Some('}') => depth -= 1,
                Some('"') => {
                    if self.chars.get(self.pos) == Some(&'"')
                        && self.chars.get(self.pos + 1) == Some(&'"')
                    {
                        self.pos += 2;
                        loop {
                            match self.advance() {
                                Some('"') => {
                                    if self.peek() == Some('"')
                                        && self.chars.get(self.pos + 1) == Some(&'"')
                                    {
                                        self.advance();
                                        self.advance();
                                        break;
                                    }
                                }
                                None => break,
                                _ => {}
                            }
                        }
                    } else {
                        loop {
                            match self.advance() {
                                Some('"') => break,
                                Some('\\') => {
                                    self.advance();
                                }
                                None => break,
                                _ => {}
                            }
                        }
                    }
                }
                None => anyhow::bail!("unterminated braces block"),
                _ => {}
            }
        }
        Ok(())
    }
}

fn parse_metadata_block(content: &str) -> (Option<String>, Option<String>, Vec<String>, String) {
    let mut inherits = None;
    let mut doc = None;
    let mut allowed_tokens = Vec::new();
    let mut custom_data = String::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("inherits") {
            if let Some(start) = trimmed.find("</") {
                if let Some(end) = trimmed[start..].find('>') {
                    inherits = Some(trimmed[start + 2..start + end].to_string());
                }
            }
        } else if trimmed.starts_with("doc") {
            let mut doc_str = String::new();
            if trimmed.contains("\"\"\"") {
                let after = trimmed.splitn(2, "\"\"\"").nth(1).unwrap_or("");
                doc_str.push_str(after);
                if after.contains("\"\"\"") {
                    doc_str = after.split("\"\"\"").next().unwrap_or("").to_string();
                } else {
                    i += 1;
                    while i < lines.len() {
                        if lines[i].contains("\"\"\"") {
                            let before = lines[i].split("\"\"\"").next().unwrap_or("");
                            doc_str.push('\n');
                            doc_str.push_str(before);
                            break;
                        }
                        doc_str.push('\n');
                        doc_str.push_str(lines[i]);
                        i += 1;
                    }
                }
            } else if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    doc_str = trimmed[start + 1..start + 1 + end].to_string();
                }
            }
            doc = Some(doc_str.trim().to_string());
        } else if trimmed.starts_with("allowedTokens") {
            let mut tokens_str = String::new();
            let rest = &content[content.find("allowedTokens").unwrap()..];
            if let Some(bracket_start) = rest.find('[') {
                if let Some(bracket_end) = rest[bracket_start..].find(']') {
                    tokens_str = rest[bracket_start + 1..bracket_start + bracket_end].to_string();
                }
            }
            for tok in tokens_str.split(',') {
                let tok = tok.trim().trim_matches('"').trim();
                if !tok.is_empty() {
                    allowed_tokens.push(tok.to_string());
                }
            }
        } else if trimmed.starts_with("customData") {
            custom_data = trimmed.to_string();
        }
        i += 1;
    }

    (inherits, doc, allowed_tokens, custom_data)
}

fn parse_property_metadata(content: &str) -> (Option<String>, Vec<String>, String) {
    let mut allowed_tokens = Vec::new();
    let mut doc = String::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("allowedTokens") {
            let rest = content;
            if let Some(bracket_start) = rest.find("allowedTokens") {
                let after = &rest[bracket_start..];
                if let Some(bs) = after.find('[') {
                    if let Some(be) = after[bs..].find(']') {
                        let tokens_str = &after[bs + 1..bs + be];
                        for tok in tokens_str.split(',') {
                            let tok = tok.trim().trim_matches('"').trim();
                            if !tok.is_empty() {
                                allowed_tokens.push(tok.to_string());
                            }
                        }
                    }
                }
            }
        } else if trimmed.starts_with("doc") {
            let mut doc_str = String::new();
            if trimmed.contains("\"\"\"") {
                let after = trimmed.splitn(2, "\"\"\"").nth(1).unwrap_or("");
                doc_str.push_str(after);
                if after.contains("\"\"\"") {
                    doc_str = after.split("\"\"\"").next().unwrap_or("").to_string();
                } else {
                    i += 1;
                    while i < lines.len() {
                        if lines[i].contains("\"\"\"") {
                            let before = lines[i].split("\"\"\"").next().unwrap_or("");
                            doc_str.push('\n');
                            doc_str.push_str(before);
                            break;
                        }
                        doc_str.push('\n');
                        doc_str.push_str(lines[i]);
                        i += 1;
                    }
                }
            }
            doc = doc_str.trim().to_string();
        }
        i += 1;
    }

    (None, allowed_tokens, doc)
}

// ---------------------------------------------------------------------------
// High-level schema parser
// ---------------------------------------------------------------------------

fn parse_schema_file(input: &str) -> Result<Vec<SchemaClass>> {
    let mut classes = Vec::new();
    let mut p = Parser::new(input);

    // Skip the file header (everything before the first `class` or `over`)
    loop {
        p.skip_whitespace();
        if p.peek().is_none() {
            break;
        }
        let saved = p.pos;
        match p.read_identifier() {
            Ok(word) => match word.as_str() {
                "class" => {
                    if let Ok(cls) = parse_class(&mut p) {
                        classes.push(cls);
                    }
                }
                "over" => {
                    // Skip `over` blocks
                    p.skip_whitespace();
                    let _ = p.read_quoted_string();
                    p.skip_whitespace();
                    if p.peek() == Some('(') {
                        let _ = p.skip_balanced_parens();
                    }
                    p.skip_whitespace();
                    if p.peek() == Some('{') {
                        let _ = p.skip_balanced_braces();
                    }
                }
                _ => {
                    // Unknown top-level, try to skip
                    if p.pos == saved + word.len() {
                        // skip forward
                    }
                }
            },
            Err(_) => {
                p.advance();
            }
        }
    }

    Ok(classes)
}

fn parse_class(p: &mut Parser) -> Result<SchemaClass> {
    p.skip_whitespace();

    // Determine if concrete: `class Mesh "Mesh"` (concrete) vs `class "Imageable"` (abstract)
    // Concrete classes have an unquoted type name before the quoted prim name.
    let (schema_name, is_concrete) = if p.peek() == Some('"') {
        // Abstract class: class "Name"
        let quoted = p.read_quoted_string()?;
        (quoted, false)
    } else {
        // Concrete class: class TypeName "PrimName"
        let _type_name = p.read_identifier()?;
        p.skip_whitespace();
        let quoted = p.read_quoted_string()?;
        (quoted, true)
    };

    // Parse metadata block
    let mut inherits = None;
    let mut doc = String::new();

    p.skip_whitespace();
    if p.peek() == Some('(') {
        let meta_content = p.skip_balanced_parens()?;
        let (inh, d, _, _) = parse_metadata_block(&meta_content);
        inherits = inh;
        if let Some(d) = d {
            doc = d;
        }
    }

    // Parse body
    let mut properties = Vec::new();
    p.skip_whitespace();
    if p.peek() == Some('{') {
        p.expect_char('{')?;
        loop {
            p.skip_whitespace();
            if p.peek() == Some('}') {
                p.advance();
                break;
            }
            if p.peek().is_none() {
                break;
            }

            match parse_property(p) {
                Ok(Some(prop)) => properties.push(prop),
                Ok(None) => {}
                Err(_) => {
                    // Try to recover by skipping to next line or brace
                    while let Some(c) = p.peek() {
                        if c == '\n' || c == '}' {
                            break;
                        }
                        p.advance();
                    }
                }
            }
        }
    }

    Ok(SchemaClass {
        name: schema_name,
        is_concrete,
        inherits,
        doc,
        properties,
    })
}

fn is_usd_type(word: &str) -> bool {
    let base = word.trim_end_matches("[]");
    matches!(
        base,
        "int" | "int3" | "int4" | "int64" | "float" | "float2" | "float3"
            | "double" | "double2" | "double3"
            | "point3f" | "normal3f" | "vector3f" | "color3f"
            | "half3" | "quath" | "quatf"
            | "matrix4d" | "token" | "bool" | "string"
    )
}

fn parse_property(p: &mut Parser) -> Result<Option<Property>> {
    p.skip_whitespace();
    let saved_pos = p.pos;
    let mut is_uniform = false;
    let mut is_relationship = false;

    let first = p.read_identifier()?;
    if first == "uniform" {
        is_uniform = true;
    } else if first == "rel" {
        is_relationship = true;
    }

    if is_relationship {
        p.skip_whitespace();
        let name = p.read_identifier()?;
        p.skip_whitespace();
        if p.peek() == Some('(') {
            let _ = p.skip_balanced_parens()?;
        }
        return Ok(Some(Property {
            name,
            usd_type: "rel".to_string(),
            is_uniform: false,
            is_array: false,
            default_value: None,
            allowed_tokens: Vec::new(),
            doc: String::new(),
            is_relationship: true,
        }));
    }

    let mut type_word = if is_uniform {
        p.skip_whitespace();
        p.read_identifier()?
    } else {
        first
    };

    // Handle array suffix `[]` which isn't captured by read_identifier
    if p.peek() == Some('[') && p.chars.get(p.pos + 1) == Some(&']') {
        type_word.push_str("[]");
        p.advance();
        p.advance();
    }

    if !is_usd_type(&type_word) {
        p.pos = saved_pos;
        while let Some(c) = p.peek() {
            if c == '\n' {
                p.advance();
                break;
            }
            p.advance();
        }
        return Ok(None);
    }

    let is_arr = type_word.ends_with("[]");
    let clean_type = type_word.trim_end_matches("[]").to_string();
    let type_str = if is_arr {
        format!("{clean_type}[]")
    } else {
        clean_type
    };

    p.skip_whitespace();
    let prop_name = match p.read_identifier() {
        Ok(name) => name,
        Err(_) => {
            p.pos = saved_pos;
            while let Some(c) = p.peek() {
                if c == '\n' {
                    p.advance();
                    break;
                }
                p.advance();
            }
            return Ok(None);
        }
    };

    let is_array = type_str.ends_with("[]");

    // Parse optional default value
    let mut default_value = None;
    p.skip_whitespace();
    if p.peek() == Some('=') {
        p.advance(); // skip '='
        p.skip_whitespace();
        let mut val = String::new();
        if p.peek() == Some('[') {
            val.push('[');
            p.advance();
            let mut depth = 1;
            while depth > 0 {
                match p.advance() {
                    Some('[') => {
                        depth += 1;
                        val.push('[');
                    }
                    Some(']') => {
                        depth -= 1;
                        if depth > 0 {
                            val.push(']');
                        }
                    }
                    Some(c) => val.push(c),
                    None => break,
                }
            }
            val.push(']');
            default_value = Some(val.trim().to_string());
        } else if p.peek() == Some('(') {
            val.push('(');
            p.advance();
            let mut depth = 1;
            while depth > 0 {
                match p.advance() {
                    Some('(') => {
                        depth += 1;
                        val.push('(');
                    }
                    Some(')') => {
                        depth -= 1;
                        if depth > 0 {
                            val.push(')');
                        }
                    }
                    Some(c) => val.push(c),
                    None => break,
                }
            }
            val.push(')');
            default_value = Some(val.trim().to_string());
        } else if p.peek() == Some('"') {
            let s = p.read_quoted_string()?;
            default_value = Some(format!("\"{s}\""));
        } else {
            while let Some(c) = p.peek() {
                if c.is_whitespace() || c == '(' || c == ')' || c == '{' || c == '}' {
                    break;
                }
                val.push(c);
                p.advance();
            }
            default_value = Some(val.trim().to_string());
        }
    }

    // Parse optional metadata
    let mut allowed_tokens = Vec::new();
    let mut doc = String::new();
    p.skip_whitespace();
    if p.peek() == Some('(') {
        let meta = p.skip_balanced_parens()?;
        let (_, at, d) = parse_property_metadata(&meta);
        allowed_tokens = at;
        doc = d;
    }

    Ok(Some(Property {
        name: prop_name,
        usd_type: type_str,
        is_uniform,
        is_array,
        default_value,
        allowed_tokens,
        doc,
        is_relationship: false,
    }))
}

// ---------------------------------------------------------------------------
// Code generation
// ---------------------------------------------------------------------------

fn usd_type_to_rust(usd_type: &str, is_array: bool) -> TokenStream {
    let base = usd_type.trim_end_matches("[]");
    let inner: TokenStream = match base {
        "int" => quote! { i32 },
        "int4" => quote! { [i32; 4] },
        "int3" => quote! { [i32; 3] },
        "int64" => quote! { i64 },
        "float" => quote! { f32 },
        "double" => quote! { f64 },
        "double2" => quote! { crate::foundation::GfVec2d },
        "double3" => quote! { crate::foundation::GfVec3d },
        "float2" => quote! { [f32; 2] },
        "float3" => quote! { crate::foundation::GfVec3f },
        "point3f" => quote! { crate::foundation::GfVec3f },
        "normal3f" => quote! { crate::foundation::GfVec3f },
        "vector3f" => quote! { crate::foundation::GfVec3f },
        "half3" => quote! { [u16; 3] },
        "quath" => quote! { [u16; 4] },
        "quatf" => quote! { [f32; 4] },
        "color3f" => quote! { crate::foundation::GfVec3f },
        "matrix4d" => quote! { crate::foundation::GfMatrix4d },
        "token" => quote! { crate::foundation::TfToken },
        "bool" => quote! { bool },
        "string" => quote! { String },
        "rel" => quote! { Vec<crate::foundation::SdfPath> },
        _ => quote! { String },
    };

    if is_array {
        quote! { crate::foundation::VtArray<#inner> }
    } else {
        inner
    }
}

fn usd_type_name_for_attr(usd_type: &str) -> &str {
    let base = usd_type.trim_end_matches("[]");
    match base {
        "int" | "int4" | "int3" | "int64" | "float" | "double" | "double2" | "double3"
        | "float2" | "float3" | "point3f" | "normal3f" | "vector3f" | "half3" | "quath"
        | "quatf" | "color3f" | "matrix4d" | "token" | "bool" | "string" => usd_type,
        _ => usd_type,
    }
}

fn field_name_ident(name: &str) -> Ident {
    let snake = name.replace(':', "_").to_snake_case();
    let safe = escape_rust_keyword(&snake);
    Ident::new(&safe, Span::call_site())
}

fn escape_rust_keyword(s: &str) -> String {
    match s {
        "type" | "mod" | "ref" | "in" | "fn" | "let" | "mut" | "use" | "pub" | "self"
        | "super" | "crate" | "as" | "break" | "const" | "continue" | "else" | "enum"
        | "extern" | "false" | "for" | "if" | "impl" | "loop" | "match" | "move"
        | "return" | "static" | "struct" | "trait" | "true" | "unsafe" | "where" | "while"
        | "async" | "await" | "dyn" | "abstract" | "become" | "box" | "do" | "final"
        | "macro" | "override" | "priv" | "typeof" | "unsized" | "virtual" | "yield" => {
            format!("{s}_")
        }
        other => other.to_string(),
    }
}

fn sanitize_doc(doc: &str) -> String {
    doc.lines()
        .map(|l| l.trim())
        .collect::<Vec<_>>()
        .join("\n")
        .replace("\\em ", "*")
        .replace("\\ref ", "")
        .replace("\\note", "Note:")
        .replace("\\li ", "- ")
        .replace("\\sa ", "See: ")
        .replace("\\section ", "## ")
        .replace("\\anchor ", "")
        .replace("\\deprecated", "**Deprecated:**")
        .replace("\\code", "```")
        .replace("\\endcode", "```")
        .replace("\\image html", "[image]")
}

/// Collect all properties for a class by flattening inherited properties.
fn collect_all_properties(
    class: &SchemaClass,
    class_map: &HashMap<String, &SchemaClass>,
) -> Vec<Property> {
    let mut props = Vec::new();

    if let Some(ref parent_name) = class.inherits {
        if let Some(parent) = class_map.get(parent_name) {
            props.extend(collect_all_properties(parent, class_map));
        }
    }

    for prop in &class.properties {
        if prop.is_relationship {
            continue;
        }
        if !props.iter().any(|p: &Property| p.name == prop.name) {
            props.push(prop.clone());
        }
    }

    props
}

fn generate_struct(
    class: &SchemaClass,
    all_props: &[Property],
) -> TokenStream {
    let struct_name = format_ident!("UsdGeom{}", &class.name);
    let doc_comment = sanitize_doc(&class.doc);

    let fields: Vec<TokenStream> = all_props
        .iter()
        .filter(|p| !p.is_relationship)
        .map(|p| {
            let field = field_name_ident(&p.name);
            let ty = usd_type_to_rust(&p.usd_type, p.is_array);
            let pdoc = sanitize_doc(&p.doc);

            let field_ty = if is_required_type(p) {
                ty.clone()
            } else {
                quote! { Option<#ty> }
            };

            quote! {
                #[doc = #pdoc]
                pub #field: #field_ty
            }
        })
        .collect();

    let default_fields: Vec<TokenStream> = all_props
        .iter()
        .filter(|p| !p.is_relationship)
        .map(|p| {
            let field = field_name_ident(&p.name);
            let default_expr = default_expr_for_property(p);
            quote! { #field: #default_expr }
        })
        .collect();

    let schema_name_str = format!("UsdGeom{}", &class.name);

    let attr_entries: Vec<TokenStream> = all_props
        .iter()
        .filter(|p| !p.is_relationship)
        .map(|p| {
            let usd_name = &p.name;
            let usd_type_str = usd_type_name_for_attr(&p.usd_type);
            let is_uniform = p.is_uniform;
            quote! {
                crate::schema::generated::AttributeMetadata {
                    usd_name: #usd_name,
                    usd_type: #usd_type_str,
                    is_uniform: #is_uniform,
                }
            }
        })
        .collect();

    quote! {
        #[doc = #doc_comment]
        #[derive(Clone, Debug)]
        pub struct #struct_name {
            #(#fields,)*
        }

        impl Default for #struct_name {
            fn default() -> Self {
                Self {
                    #(#default_fields,)*
                }
            }
        }

        impl crate::schema::generated::UsdSchemaInfo for #struct_name {
            fn schema_name(&self) -> &'static str {
                #schema_name_str
            }

            fn attribute_metadata(&self) -> &'static [crate::schema::generated::AttributeMetadata] {
                static META: &[crate::schema::generated::AttributeMetadata] = &[
                    #(#attr_entries,)*
                ];
                META
            }
        }
    }
}

fn is_required_type(p: &Property) -> bool {
    if p.is_array {
        return true;
    }
    if p.default_value.is_some() {
        return true;
    }
    matches!(
        p.usd_type.trim_end_matches("[]"),
        "token" | "bool" | "string"
    )
}

fn default_expr_for_property(p: &Property) -> TokenStream {
    let is_required = is_required_type(p);

    if p.is_array {
        return quote! { Vec::new() };
    }

    match p.usd_type.trim_end_matches("[]") {
        "token" => {
            if let Some(ref dv) = p.default_value {
                let val = dv.trim_matches('"');
                quote! { crate::foundation::TfToken::new(#val) }
            } else {
                quote! { crate::foundation::TfToken::default() }
            }
        }
        "bool" => {
            if let Some(ref dv) = p.default_value {
                if dv == "1" || dv == "true" {
                    quote! { true }
                } else {
                    quote! { false }
                }
            } else {
                quote! { false }
            }
        }
        "string" => quote! { String::new() },
        "int" | "int64" => {
            if let Some(ref dv) = p.default_value {
                if let Ok(v) = dv.parse::<i64>() {
                    let v = v as i32;
                    quote! { #v }
                } else {
                    quote! { None }
                }
            } else {
                quote! { None }
            }
        }
        "float" => {
            if let Some(ref dv) = p.default_value {
                if let Ok(v) = dv.parse::<f32>() {
                    quote! { #v }
                } else if is_required { quote! { 0.0f32 } } else { quote! { None } }
            } else if is_required { quote! { 0.0f32 } } else {
                quote! { None }
            }
        }
        "double" => {
            if let Some(ref dv) = p.default_value {
                if let Ok(v) = dv.parse::<f64>() {
                    quote! { #v }
                } else if is_required { quote! { 0.0f64 } } else { quote! { None } }
            } else if is_required { quote! { 0.0f64 } } else {
                quote! { None }
            }
        }
        "float2" => {
            if is_required {
                quote! { [0.0f32; 2] }
            } else {
                quote! { None }
            }
        }
        "float3" | "point3f" | "normal3f" | "vector3f" | "color3f" => {
            if is_required {
                quote! { crate::foundation::GfVec3f::default() }
            } else {
                quote! { None }
            }
        }
        "double2" => {
            if is_required {
                quote! { crate::foundation::GfVec2d::default() }
            } else {
                quote! { None }
            }
        }
        "double3" => {
            if is_required {
                quote! { crate::foundation::GfVec3d::default() }
            } else {
                quote! { None }
            }
        }
        "matrix4d" => quote! { None },
        _ => quote! { None },
    }
}

fn generate_schema_data_enum(concrete_classes: &[(&SchemaClass, Vec<Property>)]) -> TokenStream {
    let variants: Vec<TokenStream> = concrete_classes
        .iter()
        .map(|(cls, _)| {
            let variant = format_ident!("{}", &cls.name);
            let ty = format_ident!("UsdGeom{}", &cls.name);
            quote! { #variant(#ty) }
        })
        .collect();

    let schema_name_arms: Vec<TokenStream> = concrete_classes
        .iter()
        .map(|(cls, _)| {
            let variant = format_ident!("{}", &cls.name);
            let name = format!("UsdGeom{}", &cls.name);
            quote! { SchemaData::#variant(_) => #name }
        })
        .collect();

    let from_impls: Vec<TokenStream> = concrete_classes
        .iter()
        .map(|(cls, _)| {
            let variant = format_ident!("{}", &cls.name);
            let ty = format_ident!("UsdGeom{}", &cls.name);
            quote! {
                impl From<#ty> for SchemaData {
                    fn from(v: #ty) -> Self {
                        SchemaData::#variant(v)
                    }
                }
            }
        })
        .collect();

    let usd_schema_impls: Vec<TokenStream> = concrete_classes
        .iter()
        .map(|(cls, _)| {
            let variant = format_ident!("{}", &cls.name);
            let ty = format_ident!("UsdGeom{}", &cls.name);
            quote! {
                impl UsdSchema for #ty {
                    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
                        match d {
                            SchemaData::#variant(ref v) => Some(v),
                            _ => None,
                        }
                    }

                    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
                        match d {
                            SchemaData::#variant(ref mut v) => Some(v),
                            _ => None,
                        }
                    }
                }
            }
        })
        .collect();

    quote! {
        /// Scope prim — no geometry, just a grouping node.
        #[derive(Clone, Debug, Default)]
        pub struct UsdGeomScope;

        impl UsdSchemaInfo for UsdGeomScope {
            fn schema_name(&self) -> &'static str { "Scope" }
            fn attribute_metadata(&self) -> &'static [AttributeMetadata] { &[] }
        }

        /// Discriminated union of all concrete USD geometry schema types.
        #[derive(Clone, Debug)]
        pub enum SchemaData {
            Scope(UsdGeomScope),
            #(#variants,)*
        }

        impl Default for SchemaData {
            fn default() -> Self {
                SchemaData::Scope(UsdGeomScope)
            }
        }

        impl SchemaData {
            pub fn schema_name(&self) -> &'static str {
                match self {
                    SchemaData::Scope(_) => "Scope",
                    #(#schema_name_arms,)*
                }
            }
        }

        impl From<UsdGeomScope> for SchemaData {
            fn from(v: UsdGeomScope) -> Self {
                SchemaData::Scope(v)
            }
        }

        #(#from_impls)*

        impl UsdSchema for UsdGeomScope {
            fn from_schema_data(d: &SchemaData) -> Option<&Self> {
                match d {
                    SchemaData::Scope(ref v) => Some(v),
                    _ => None,
                }
            }

            fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
                match d {
                    SchemaData::Scope(ref mut v) => Some(v),
                    _ => None,
                }
            }
        }

        #(#usd_schema_impls)*
    }
}

fn generate_token_constants(all_classes: &[SchemaClass]) -> TokenStream {
    let mut seen = std::collections::HashSet::new();
    let mut consts = Vec::new();

    for cls in all_classes {
        for prop in &cls.properties {
            for tok in &prop.allowed_tokens {
                if seen.insert(tok.clone()) {
                    let raw_name = tok.to_snake_case().to_uppercase();
                    let safe_name = escape_rust_keyword(&raw_name);
                    let const_name = format_ident!("{}", safe_name);
                    let tok_val = tok.as_str();
                    consts.push(quote! {
                        pub const #const_name: &str = #tok_val;
                    });
                }
            }
        }
    }

    quote! {
        /// Token constants from allowed values in the USD schema.
        pub mod tokens {
            #(#consts)*
        }
    }
}

/// Convenience accessor methods for batched curve/patch types.
fn generate_convenience_methods(class: &SchemaClass, _all_props: &[Property]) -> TokenStream {
    let struct_name = format_ident!("UsdGeom{}", &class.name);
    let methods: Vec<TokenStream> = match class.name.as_str() {
        "NurbsCurves" => {
            vec![
                quote! {
                    /// Degree of curve `i` (order - 1).
                    #[inline]
                    pub fn degree(&self, i: usize) -> usize {
                        (self.order[i] as usize).saturating_sub(1)
                    }
                },
                quote! {
                    /// Knot slice for curve `i`.
                    #[inline]
                    pub fn knot_slice(&self, i: usize) -> &[f64] {
                        let mut offset = 0usize;
                        for j in 0..i {
                            offset += self.curve_vertex_counts[j] as usize + self.order[j] as usize;
                        }
                        let len = self.curve_vertex_counts[i] as usize + self.order[i] as usize;
                        &self.knots[offset..offset + len]
                    }
                },
                quote! {
                    /// Control-point slice for curve `i`.
                    #[inline]
                    pub fn control_points(&self, i: usize) -> &[crate::foundation::GfVec3f] {
                        let mut offset = 0usize;
                        for j in 0..i {
                            offset += self.curve_vertex_counts[j] as usize;
                        }
                        let len = self.curve_vertex_counts[i] as usize;
                        &self.points[offset..offset + len]
                    }
                },
                quote! {
                    /// Weight slice for curve `i`, or `None` if unweighted.
                    #[inline]
                    pub fn weight_slice(&self, i: usize) -> Option<&[f64]> {
                        if self.point_weights.is_empty() {
                            return None;
                        }
                        let mut offset = 0usize;
                        for j in 0..i {
                            offset += self.curve_vertex_counts[j] as usize;
                        }
                        let len = self.curve_vertex_counts[i] as usize;
                        Some(&self.point_weights[offset..offset + len])
                    }
                },
            ]
        }
        "NurbsPatch" => {
            vec![
                quote! {
                    #[inline]
                    pub fn degree_u(&self) -> usize {
                        self.u_order.map(|o| (o as usize).saturating_sub(1)).unwrap_or(0)
                    }
                },
                quote! {
                    #[inline]
                    pub fn degree_v(&self) -> usize {
                        self.v_order.map(|o| (o as usize).saturating_sub(1)).unwrap_or(0)
                    }
                },
            ]
        }
        _ => Vec::new(),
    };

    if methods.is_empty() {
        return quote! {};
    }

    quote! {
        impl #struct_name {
            #(#methods)*
        }
    }
}

// ---------------------------------------------------------------------------
// File generation
// ---------------------------------------------------------------------------

fn generate_all(classes: &[SchemaClass]) -> Result<String> {
    let class_map: HashMap<String, &SchemaClass> =
        classes.iter().map(|c| (c.name.clone(), c)).collect();

    // Only generate concrete classes, excluding Scope (handled separately in the enum)
    let concrete: Vec<(&SchemaClass, Vec<Property>)> = classes
        .iter()
        .filter(|c| c.is_concrete && c.name != "Scope")
        .map(|c| {
            let props = collect_all_properties(c, &class_map);
            (c, props)
        })
        .collect();

    let mut all_tokens = TokenStream::new();

    // Header
    all_tokens.extend(quote! {
        //! DO NOT EDIT — generated from usd/schema.usda by `cargo xtask codegen`.
        //! Schema version: OpenUSD 24.11
        //!
        //! Re-run `cargo xtask codegen` to regenerate.
        #![allow(non_camel_case_types)]

        /// Attribute metadata for generic serialization.
        #[derive(Clone, Debug)]
        pub struct AttributeMetadata {
            pub usd_name: &'static str,
            pub usd_type: &'static str,
            pub is_uniform: bool,
        }

        /// Trait providing schema metadata for generic serialization.
        pub trait UsdSchemaInfo {
            fn schema_name(&self) -> &'static str;
            fn attribute_metadata(&self) -> &'static [AttributeMetadata];
        }

        /// Trait for type-safe prim access.
        pub trait UsdSchema: Sized {
            fn from_schema_data(d: &SchemaData) -> Option<&Self>;
            fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self>;
        }
    });

    // Generate struct + Default + UsdSchemaInfo for each concrete class
    for (cls, props) in &concrete {
        all_tokens.extend(generate_struct(cls, props));
        all_tokens.extend(generate_convenience_methods(cls, props));
    }

    // Generate SchemaData enum + UsdSchema impls
    all_tokens.extend(generate_schema_data_enum(&concrete));

    // Generate token constants
    all_tokens.extend(generate_token_constants(classes));

    let file = syn::parse2(all_tokens).context("failed to parse generated tokens as a syn File")?;
    Ok(prettyplease::unparse(&file))
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(check: bool) -> Result<()> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();

    let schema_path = workspace_root.join("usd/schema.usda");
    let output_path = workspace_root.join("crates/usd/src/schema/generated/mod.rs");

    eprintln!("Parsing {}", schema_path.display());
    let input = std::fs::read_to_string(&schema_path)
        .with_context(|| format!("reading {}", schema_path.display()))?;

    let classes = parse_schema_file(&input)?;
    eprintln!(
        "Parsed {} classes ({} concrete)",
        classes.len(),
        classes.iter().filter(|c| c.is_concrete).count()
    );

    let generated = generate_all(&classes)?;

    ensure_file_contents(&output_path, &generated, check)?;

    eprintln!("Done. Output: {}", output_path.display());
    Ok(())
}

fn ensure_file_contents(path: &Path, contents: &str, check: bool) -> Result<()> {
    let norm = |s: &str| s.replace("\r\n", "\n");
    if std::fs::read_to_string(path)
        .map(|existing| norm(&existing) == norm(contents))
        .unwrap_or(false)
    {
        eprintln!("  {} is up to date", path.display());
        return Ok(());
    }
    if check {
        anyhow::bail!(
            "{} is stale. Run `cargo xtask codegen`.",
            path.display()
        );
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, contents)?;
    eprintln!("  Wrote {}", path.display());
    Ok(())
}
