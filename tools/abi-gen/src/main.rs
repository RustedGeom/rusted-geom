use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use quote::ToTokens;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use syn::{Attribute, Fields, FnArg, Item, Meta, ReturnType, Type};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long, default_value = ".")]
    workspace: PathBuf,
    #[arg(long)]
    check: bool,
    #[arg(long, default_value = "abi/baseline/rgm_abi.json")]
    baseline: PathBuf,
    #[arg(long)]
    enforce_semver_major: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AbiManifest {
    crate_name: String,
    version: String,
    abi_hash: String,
    functions: Vec<AbiFunction>,
    ffi_types: Vec<FfiType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AbiFunction {
    rust_name: String,
    c_name: String,
    interop_ts_name: String,
    public_ts_name: Option<String>,
    receiver: Option<String>,
    return_type: String,
    params: Vec<AbiParam>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AbiParam {
    name: String,
    ty: String,
}

#[derive(Debug, Clone, Default)]
struct ExportMeta {
    ts: Option<String>,
    receiver: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
enum FfiType {
    Enum {
        name: String,
        repr: String,
        variants: Vec<FfiVariant>,
    },
    Struct {
        name: String,
        repr: String,
        fields: Vec<FfiField>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FfiVariant {
    name: String,
    value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FfiField {
    name: String,
    ty: String,
}

#[derive(Debug, Clone, Serialize)]
struct HashOnlyManifest<'a> {
    crate_name: &'a str,
    version: &'a str,
    functions: &'a [AbiFunction],
    ffi_types: &'a [FfiType],
}

#[derive(Debug)]
struct OutputFile {
    path: PathBuf,
    content: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let workspace = args
        .workspace
        .canonicalize()
        .context("cannot resolve workspace path")?;

    let crate_name = "kernel-ffi".to_string();
    let version = read_crate_version(&workspace.join("crates/kernel-ffi/Cargo.toml"))?;
    let (mut functions, mut ffi_types) =
        collect_metadata(&workspace.join("crates/kernel-ffi/src"))?;

    functions.sort_by(|a, b| a.c_name.cmp(&b.c_name));
    ffi_types.sort_by_key(type_name);

    let mut manifest = AbiManifest {
        crate_name,
        version,
        abi_hash: String::new(),
        functions,
        ffi_types,
    };
    manifest.abi_hash = compute_abi_hash(&manifest)?;

    validate_manifest(&manifest)?;
    if args.enforce_semver_major {
        let baseline = if args.baseline.is_absolute() {
            args.baseline.clone()
        } else {
            workspace.join(&args.baseline)
        };
        enforce_semver_compatibility(&manifest, &baseline)?;
    }

    let outputs = generate_outputs(&workspace, &manifest)?;
    if args.check {
        check_outputs(&outputs)?;
    } else {
        write_outputs(&outputs)?;
    }

    enforce_catalog_integrity(&manifest)?;

    Ok(())
}

fn enforce_semver_compatibility(current: &AbiManifest, baseline_path: &Path) -> Result<()> {
    if !baseline_path.exists() {
        return Ok(());
    }

    let baseline_json = fs::read_to_string(baseline_path)
        .with_context(|| format!("failed reading baseline {}", baseline_path.display()))?;
    let baseline: AbiManifest = serde_json::from_str(&baseline_json)
        .with_context(|| format!("failed parsing baseline {}", baseline_path.display()))?;

    if baseline.abi_hash == current.abi_hash {
        return Ok(());
    }

    let baseline_major = parse_major(&baseline.version)?;
    let current_major = parse_major(&current.version)?;

    if baseline_major == current_major {
        bail!(
            "ABI hash changed without semver major bump.\nBaseline version: {}\nCurrent version: {}\nBaseline hash: {}\nCurrent hash: {}",
            baseline.version,
            current.version,
            baseline.abi_hash,
            current.abi_hash
        );
    }

    Ok(())
}

fn parse_major(version: &str) -> Result<u64> {
    let major = version
        .split('.')
        .next()
        .ok_or_else(|| anyhow!("invalid semver version: {version}"))?;
    major
        .parse::<u64>()
        .with_context(|| format!("invalid semver major in version: {version}"))
}

fn read_crate_version(cargo_toml: &Path) -> Result<String> {
    let contents = fs::read_to_string(cargo_toml)
        .with_context(|| format!("failed reading {}", cargo_toml.display()))?;
    let value: toml::Value = toml::from_str(&contents)?;

    if let Some(version) = value
        .get("package")
        .and_then(|pkg| pkg.get("version"))
        .and_then(toml::Value::as_str)
    {
        return Ok(version.to_string());
    }

    if value
        .get("package")
        .and_then(|pkg| pkg.get("version"))
        .and_then(toml::Value::as_table)
        .and_then(|tbl| tbl.get("workspace"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
    {
        let root_cargo = cargo_toml
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .ok_or_else(|| anyhow!("unable to locate workspace Cargo.toml"))?
            .join("Cargo.toml");
        let root_contents = fs::read_to_string(&root_cargo)
            .with_context(|| format!("failed reading {}", root_cargo.display()))?;
        let root_value: toml::Value = toml::from_str(&root_contents)?;
        let version = root_value
            .get("workspace")
            .and_then(|w| w.get("package"))
            .and_then(|p| p.get("version"))
            .and_then(toml::Value::as_str)
            .ok_or_else(|| anyhow!("workspace.package.version is missing"))?;
        return Ok(version.to_string());
    }

    bail!("package version not found");
}

fn collect_metadata(src_dir: &Path) -> Result<(Vec<AbiFunction>, Vec<FfiType>)> {
    let mut functions = Vec::new();
    let mut ffi_types = Vec::new();

    let mut files: Vec<_> = WalkDir::new(src_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "rs"))
        .map(|entry| entry.path().to_path_buf())
        .collect();
    files.sort();

    for file in files {
        let src = fs::read_to_string(&file)
            .with_context(|| format!("failed reading source file {}", file.display()))?;
        let syntax = syn::parse_file(&src)
            .with_context(|| format!("failed parsing source file {}", file.display()))?;

        for item in syntax.items {
            match item {
                Item::Fn(item_fn) => {
                    let Some(meta) = extract_export_meta(&item_fn.attrs)? else {
                        continue;
                    };

                    let rust_name = item_fn.sig.ident.to_string();
                    let c_name = if rust_name.starts_with("rgm_") {
                        rust_name.clone()
                    } else {
                        format!("rgm_{rust_name}")
                    };
                    let base_name = c_name.trim_start_matches("rgm_").to_string();

                    let mut params = Vec::new();
                    for (idx, arg) in item_fn.sig.inputs.iter().enumerate() {
                        if let FnArg::Typed(arg_typed) = arg {
                            let name = pat_to_name(&arg_typed.pat, idx);
                            let ty = normalize_type(&arg_typed.ty);
                            params.push(AbiParam { name, ty });
                        }
                    }

                    let return_type = match &item_fn.sig.output {
                        ReturnType::Default => "()".to_string(),
                        ReturnType::Type(_, ty) => normalize_type(ty),
                    };

                    functions.push(AbiFunction {
                        rust_name,
                        c_name,
                        interop_ts_name: format!("interop{}", to_pascal_case(&base_name)),
                        public_ts_name: meta.ts,
                        receiver: meta.receiver,
                        return_type,
                        params,
                    });
                }
                Item::Struct(item_struct) if has_attr(&item_struct.attrs, "rgm_ffi_type") => {
                    let name = item_struct.ident.to_string();
                    let repr = extract_repr(&item_struct.attrs).unwrap_or_else(|| "C".to_string());
                    let fields = match item_struct.fields {
                        Fields::Named(named) => named
                            .named
                            .into_iter()
                            .map(|field| FfiField {
                                name: field
                                    .ident
                                    .map(|ident| ident.to_string())
                                    .unwrap_or_else(|| "value".to_string()),
                                ty: normalize_type(&field.ty),
                            })
                            .collect(),
                        Fields::Unnamed(unnamed) => unnamed
                            .unnamed
                            .into_iter()
                            .enumerate()
                            .map(|(idx, field)| FfiField {
                                name: format!("field{idx}"),
                                ty: normalize_type(&field.ty),
                            })
                            .collect(),
                        Fields::Unit => Vec::new(),
                    };

                    ffi_types.push(FfiType::Struct { name, repr, fields });
                }
                Item::Enum(item_enum) if has_attr(&item_enum.attrs, "rgm_ffi_type") => {
                    let name = item_enum.ident.to_string();
                    let repr = extract_repr(&item_enum.attrs).unwrap_or_else(|| "i32".to_string());
                    let variants = item_enum
                        .variants
                        .into_iter()
                        .map(|variant| FfiVariant {
                            name: variant.ident.to_string(),
                            value: variant
                                .discriminant
                                .map(|(_, expr)| expr.to_token_stream().to_string()),
                        })
                        .collect();
                    ffi_types.push(FfiType::Enum {
                        name,
                        repr,
                        variants,
                    });
                }
                _ => {}
            }
        }
    }

    Ok((functions, ffi_types))
}

fn pat_to_name(pat: &syn::Pat, idx: usize) -> String {
    match pat {
        syn::Pat::Ident(ident) => ident.ident.to_string(),
        _ => format!("arg{idx}"),
    }
}

fn normalize_type(ty: &Type) -> String {
    ty.to_token_stream().to_string().replace(' ', "")
}

fn has_attr(attrs: &[Attribute], attr_name: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(attr_name))
}

fn extract_export_meta(attrs: &[Attribute]) -> Result<Option<ExportMeta>> {
    let Some(attr) = attrs.iter().find(|attr| attr.path().is_ident("rgm_export")) else {
        return Ok(None);
    };

    if matches!(attr.meta, Meta::Path(_)) {
        return Ok(Some(ExportMeta::default()));
    }

    let mut meta = ExportMeta::default();
    attr.parse_nested_meta(|nested| {
        if nested.path.is_ident("ts") {
            let value = nested.value()?.parse::<syn::LitStr>()?;
            meta.ts = Some(value.value());
            return Ok(());
        }

        if nested.path.is_ident("receiver") {
            let value = nested.value()?.parse::<syn::LitStr>()?;
            meta.receiver = Some(value.value());
            return Ok(());
        }

        Ok(())
    })?;

    Ok(Some(meta))
}

fn extract_repr(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("repr") {
            continue;
        }

        let mut repr = None;
        let _ = attr.parse_nested_meta(|meta| {
            repr = Some(meta.path.to_token_stream().to_string());
            Ok(())
        });

        if repr.is_some() {
            return repr;
        }
    }
    None
}

fn compute_abi_hash(manifest: &AbiManifest) -> Result<String> {
    let hash_only = HashOnlyManifest {
        crate_name: &manifest.crate_name,
        version: &manifest.version,
        functions: &manifest.functions,
        ffi_types: &manifest.ffi_types,
    };
    let json = serde_json::to_vec(&hash_only)?;
    let digest = Sha256::digest(json);
    Ok(format!("{:x}", digest))
}

fn validate_manifest(manifest: &AbiManifest) -> Result<()> {
    if manifest.functions.is_empty() {
        bail!("no exported ABI functions found");
    }

    let mut seen = BTreeSet::new();
    for function in &manifest.functions {
        if !seen.insert(function.c_name.clone()) {
            bail!("duplicate ABI symbol detected: {}", function.c_name);
        }

        if !function.c_name.starts_with("rgm_") {
            bail!(
                "exported symbol {} does not use rgm_ prefix",
                function.c_name
            );
        }
    }

    Ok(())
}

fn generate_outputs(workspace: &Path, manifest: &AbiManifest) -> Result<Vec<OutputFile>> {
    let manifest_json = serde_json::to_string_pretty(manifest)? + "\n";

    let ts_types = generate_typescript_types(manifest)?;
    let ts_generated = generate_typescript_api(manifest)?;
    let ts_catalog = serde_json::to_string_pretty(&public_ts_catalog(manifest))? + "\n";

    Ok(vec![
        OutputFile {
            path: workspace.join("target/abi/rgm_abi.json"),
            content: manifest_json,
        },
        OutputFile {
            path: workspace.join("bindings/web/src/generated/native.ts"),
            content: ts_generated,
        },
        OutputFile {
            path: workspace.join("bindings/web/src/generated/types.ts"),
            content: ts_types,
        },
        OutputFile {
            path: workspace.join("bindings/web/src/generated/function_catalog.json"),
            content: ts_catalog,
        },
    ])
}

fn generate_typescript_types(manifest: &AbiManifest) -> Result<String> {
    let mut out = String::new();
    writeln!(out, "// @generated by abi-gen")?;
    writeln!(out, "// version: {}", manifest.version)?;
    writeln!(out, "// abi_hash: {}", manifest.abi_hash)?;
    writeln!(out)?;
    writeln!(out, "export type WasmPtr = number;")?;
    writeln!(out)?;

    for ffi_type in ordered_ffi_types(&manifest.ffi_types) {
        match ffi_type {
            FfiType::Enum { name, variants, .. } => {
                writeln!(out, "export enum {} {{", name)?;
                for variant in variants {
                    if let Some(value) = &variant.value {
                        writeln!(out, "  {} = {},", variant.name, value)?;
                    } else {
                        writeln!(out, "  {},", variant.name)?;
                    }
                }
                writeln!(out, "}}")?;
                writeln!(out)?;
            }
            FfiType::Struct { name, fields, repr } => {
                if let Some(inner) = transparent_newtype_inner(repr, fields) {
                    writeln!(out, "export type {} = {};", name, map_ts_type(inner))?;
                    writeln!(out)?;
                    continue;
                }

                writeln!(out, "export interface {} {{", name)?;
                for field in fields {
                    writeln!(
                        out,
                        "  {}: {};",
                        field.name.trim_start_matches('_'),
                        map_ts_type(&field.ty)
                    )?;
                }
                writeln!(out, "}}")?;
                writeln!(out)?;
            }
        }
    }

    Ok(out)
}

fn generate_typescript_api(manifest: &AbiManifest) -> Result<String> {
    let kernel_create = find_by_c_name(manifest, "rgm_kernel_create")?;
    let kernel_destroy = find_by_c_name(manifest, "rgm_kernel_destroy")?;
    let object_release = find_by_c_name(manifest, "rgm_object_release")?;

    let kernel_methods = public_kernel_methods(manifest);
    let curve_methods = public_curve_methods(manifest);

    let mut out = String::new();
    writeln!(out, "// @generated by abi-gen")?;
    writeln!(out, "// version: {}", manifest.version)?;
    writeln!(out, "// abi_hash: {}", manifest.abi_hash)?;
    writeln!(out)?;
    writeln!(
        out,
        "import type {{ RgmArc3, RgmCircle3, RgmKernelHandle, RgmLine3, RgmObjectHandle, RgmPlane, RgmPoint3, RgmPolycurveSegment, RgmToleranceContext, RgmVec3 }} from \"./types\";"
    )?;
    writeln!(out, "import {{ RgmStatus }} from \"./types\";")?;
    writeln!(out)?;

    writeln!(out, "export interface NativeExports {{")?;
    for function in &manifest.functions {
        write!(out, "  {}: (", function.c_name)?;
        for (idx, param) in function.params.iter().enumerate() {
            if idx > 0 {
                write!(out, ", ")?;
            }
            write!(
                out,
                "{}: {}",
                to_camel_case(&param.name),
                map_ts_native_param_type(&param.ty)
            )?;
        }
        writeln!(
            out,
            ") => {};",
            map_ts_native_return_type(&function.return_type)
        )?;
    }
    writeln!(out, "}}")?;
    writeln!(out)?;

    writeln!(out, "export class RgmError extends Error {{")?;
    writeln!(
        out,
        "  constructor(public readonly status: RgmStatus, message: string) {{"
    )?;
    writeln!(out, "    super(message);")?;
    writeln!(out, "    this.name = \"RgmError\";")?;
    writeln!(out, "  }}")?;
    writeln!(out, "}}")?;
    writeln!(out)?;

    writeln!(out, "export interface NativeAdapter {{")?;
    writeln!(
        out,
        "  {}(): {{ status: RgmStatus; session: bigint }};",
        kernel_create.interop_ts_name
    )?;
    writeln!(
        out,
        "  {}(session: bigint): {{ status: RgmStatus }};",
        kernel_destroy.interop_ts_name
    )?;
    writeln!(
        out,
        "  {}(session: bigint, object: bigint): {{ status: RgmStatus }};",
        object_release.interop_ts_name
    )?;
    writeln!(out, "  lastErrorMessage(session: bigint): string;")?;

    for function in &kernel_methods {
        let input_params = kernel_public_input_params(&function.params);
        let (return_ty, _) = ts_return_type(function)?;

        write!(out, "  {}(session: bigint", function.interop_ts_name)?;
        for param in &input_params {
            write!(out, ", {}", map_ts_kernel_adapter_param(param))?;
        }
        writeln!(out, "): {{ status: RgmStatus; value: {} }};", return_ty)?;
    }

    for function in &curve_methods {
        let (return_ty, _) = ts_return_type(function)?;
        write!(
            out,
            "  {}(session: bigint, curve: bigint",
            function.interop_ts_name
        )?;
        for param in user_input_params(&function.params) {
            write!(
                out,
                ", {}: {}",
                to_camel_case(&param.name),
                map_ts_type(&param.ty)
            )?;
        }
        writeln!(out, "): {{ status: RgmStatus; value: {} }};", return_ty)?;
    }

    writeln!(out, "}}")?;
    writeln!(out)?;

    writeln!(out, "function throwIfError(adapter: NativeAdapter, session: bigint, status: RgmStatus, context: string): void {{")?;
    writeln!(out, "  if (status === RgmStatus.Ok) return;")?;
    writeln!(
        out,
        "  const message = adapter.lastErrorMessage(session) || context;"
    )?;
    writeln!(
        out,
        "  throw new RgmError(status, `${{context}}: ${{message}}`);"
    )?;
    writeln!(out, "}}")?;
    writeln!(out)?;

    writeln!(out, "export class KernelHandle {{")?;
    writeln!(out, "  private disposed = false;")?;
    writeln!(out, "  private constructor(private readonly adapter: NativeAdapter, private readonly session: bigint) {{}}")?;
    writeln!(out)?;
    writeln!(
        out,
        "  static create(adapter: NativeAdapter): KernelHandle {{"
    )?;
    writeln!(
        out,
        "    const result = adapter.{}();",
        kernel_create.interop_ts_name
    )?;
    writeln!(
        out,
        "    throwIfError(adapter, result.session, result.status, \"KernelHandle.create\");"
    )?;
    writeln!(out, "    return new KernelHandle(adapter, result.session);")?;
    writeln!(out, "  }}")?;

    for function in &kernel_methods {
        let method_name = function
            .public_ts_name
            .as_ref()
            .expect("public kernel method should have ts name");
        let out_param = extract_out_param(&function.params)?;
        let out_inner = out_param
            .ty
            .strip_prefix("*mut")
            .ok_or_else(|| anyhow!("invalid TS out parameter type"))?;
        let input_params = kernel_public_input_params(&function.params);
        let return_type = if out_inner == "RgmObjectHandle" {
            "CurveHandle".to_string()
        } else {
            map_ts_type(out_inner)
        };

        writeln!(out)?;
        write!(out, "  {}(", method_name)?;
        for (idx, param) in input_params.iter().enumerate() {
            if idx > 0 {
                write!(out, ", ")?;
            }
            write!(out, "{}", map_ts_kernel_public_param(param))?;
        }
        writeln!(out, "): {} {{", return_type)?;
        writeln!(out, "    this.throwIfDisposed();")?;
        write!(
            out,
            "    const result = this.adapter.{}(this.session",
            function.interop_ts_name
        )?;
        for param in &input_params {
            write!(out, ", {}", kernel_ts_call_arg(param))?;
        }
        writeln!(out, ");")?;
        writeln!(
            out,
            "    throwIfError(this.adapter, this.session, result.status, \"KernelHandle.{}\");",
            method_name
        )?;
        if out_inner == "RgmObjectHandle" {
            writeln!(
                out,
                "    return new CurveHandle(this.adapter, this.session, result.value);"
            )?;
        } else {
            writeln!(out, "    return result.value;")?;
        }
        writeln!(out, "  }}")?;
    }

    writeln!(out)?;
    writeln!(out, "  dispose(): void {{")?;
    writeln!(out, "    if (this.disposed) return;")?;
    writeln!(
        out,
        "    const result = this.adapter.{}(this.session);",
        kernel_destroy.interop_ts_name
    )?;
    writeln!(
        out,
        "    if (result.status !== RgmStatus.Ok && result.status !== RgmStatus.NotFound) {{"
    )?;
    writeln!(
        out,
        "      throwIfError(this.adapter, this.session, result.status, \"KernelHandle.dispose\");"
    )?;
    writeln!(out, "    }}")?;
    writeln!(out, "    this.disposed = true;")?;
    writeln!(out, "  }}")?;
    writeln!(out)?;
    writeln!(out, "  private throwIfDisposed(): void {{")?;
    writeln!(
        out,
        "    if (this.disposed) throw new Error(\"KernelHandle is disposed\");"
    )?;
    writeln!(out, "  }}")?;
    writeln!(out, "}}")?;
    writeln!(out)?;

    writeln!(out, "export class CurveHandle {{")?;
    writeln!(out, "  private disposed = false;")?;
    writeln!(out, "  constructor(private readonly adapter: NativeAdapter, private readonly session: bigint, private readonly curve: bigint) {{}}")?;

    for function in &curve_methods {
        let method_name = function
            .public_ts_name
            .as_ref()
            .expect("curve method should have ts public name");
        let (return_ty, _) = ts_return_type(function)?;
        let input_params = user_input_params(&function.params);

        writeln!(out)?;
        write!(out, "  {}(", method_name)?;
        for (idx, param) in input_params.iter().enumerate() {
            if idx > 0 {
                write!(out, ", ")?;
            }
            write!(
                out,
                "{}: {}",
                to_camel_case(&param.name),
                map_ts_type(&param.ty)
            )?;
        }
        writeln!(out, "): {} {{", return_ty)?;
        writeln!(out, "    this.throwIfDisposed();")?;
        write!(
            out,
            "    const result = this.adapter.{}(this.session, this.curve",
            function.interop_ts_name
        )?;
        for param in &input_params {
            write!(out, ", {}", to_camel_case(&param.name))?;
        }
        writeln!(out, ");")?;
        writeln!(
            out,
            "    throwIfError(this.adapter, this.session, result.status, \"CurveHandle.{}\");",
            method_name
        )?;
        writeln!(out, "    return result.value;")?;
        writeln!(out, "  }}")?;
    }

    writeln!(out)?;
    writeln!(out, "  dispose(): void {{")?;
    writeln!(out, "    if (this.disposed) return;")?;
    writeln!(
        out,
        "    const result = this.adapter.{}(this.session, this.curve);",
        object_release.interop_ts_name
    )?;
    writeln!(
        out,
        "    if (result.status !== RgmStatus.Ok && result.status !== RgmStatus.NotFound) {{"
    )?;
    writeln!(
        out,
        "      throwIfError(this.adapter, this.session, result.status, \"CurveHandle.dispose\");"
    )?;
    writeln!(out, "    }}")?;
    writeln!(out, "    this.disposed = true;")?;
    writeln!(out, "  }}")?;
    writeln!(out)?;
    writeln!(out, "  private throwIfDisposed(): void {{")?;
    writeln!(
        out,
        "    if (this.disposed) throw new Error(\"CurveHandle is disposed\");"
    )?;
    writeln!(out, "  }}")?;
    writeln!(out, "}}")?;

    writeln!(out)?;
    writeln!(out, "export const FUNCTION_CATALOG = [")?;
    for function in &manifest.functions {
        writeln!(
            out,
            "  {{ c: \"{}\", interopTs: \"{}\", ts: {} }},",
            function.c_name,
            function.interop_ts_name,
            function
                .public_ts_name
                .as_ref()
                .map(|value| format!("\"{}\"", value))
                .unwrap_or_else(|| "null".to_string())
        )?;
    }
    writeln!(out, "] as const;")?;

    Ok(out)
}

fn transparent_newtype_inner<'a>(repr: &str, fields: &'a [FfiField]) -> Option<&'a str> {
    if repr == "transparent" && fields.len() == 1 {
        return Some(fields[0].ty.as_str());
    }
    None
}

fn map_ts_type(rust_type: &str) -> String {
    if let Some(inner) = rust_type.strip_prefix("*mut") {
        return map_ts_type(inner);
    }
    if let Some(inner) = rust_type.strip_prefix("*const") {
        return map_ts_type(inner);
    }

    match rust_type {
        "bool" => "boolean".to_string(),
        "u8" | "u32" | "i32" | "f64" => "number".to_string(),
        "u64" => "bigint".to_string(),
        "usize" => "number".to_string(),
        "()" => "void".to_string(),
        other => other.to_string(),
    }
}

fn map_ts_native_param_type(rust_type: &str) -> String {
    if rust_type.starts_with("*mut") || rust_type.starts_with("*const") {
        return "number".to_string();
    }

    map_ts_type(rust_type)
}

fn map_ts_native_return_type(rust_type: &str) -> String {
    map_ts_type(rust_type)
}

fn user_input_params(params: &[AbiParam]) -> Vec<AbiParam> {
    params
        .iter()
        .filter(|param| {
            let name = param.name.trim_start_matches('_');
            name != "session" && name != "curve" && !param.ty.starts_with("*mut")
        })
        .cloned()
        .collect()
}

fn ts_return_type(function: &AbiFunction) -> Result<(String, String)> {
    let out_param = extract_out_param(&function.params)?;
    let inner = out_param
        .ty
        .strip_prefix("*mut")
        .ok_or_else(|| anyhow!("invalid TS out parameter type"))?;
    Ok((map_ts_type(inner), out_param.name.clone()))
}

fn extract_out_param(params: &[AbiParam]) -> Result<&AbiParam> {
    params
        .iter()
        .rev()
        .find(|param| param.ty.starts_with("*mut"))
        .ok_or_else(|| anyhow!("expected out parameter in generated public method"))
}

fn check_outputs(outputs: &[OutputFile]) -> Result<()> {
    let mut mismatches = Vec::new();

    for output in outputs {
        match fs::read_to_string(&output.path) {
            Ok(existing) => {
                if existing != output.content {
                    mismatches.push(output.path.display().to_string());
                }
            }
            Err(_) => mismatches.push(output.path.display().to_string()),
        }
    }

    if mismatches.is_empty() {
        return Ok(());
    }

    bail!(
        "generated outputs are stale or missing:\n{}\nRun: cargo run -p abi-gen -- --workspace .",
        mismatches.join("\n")
    )
}

fn write_outputs(outputs: &[OutputFile]) -> Result<()> {
    for output in outputs {
        if let Some(parent) = output.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed creating directory {}", parent.display()))?;
        }

        fs::write(&output.path, &output.content)
            .with_context(|| format!("failed writing {}", output.path.display()))?;
    }

    Ok(())
}

fn enforce_catalog_integrity(manifest: &AbiManifest) -> Result<()> {
    let public = public_functions(manifest);
    let mut ts = BTreeSet::new();

    for function in public {
        let ts_name = function
            .public_ts_name
            .as_ref()
            .ok_or_else(|| anyhow!("missing ts public name for {}", function.c_name))?;

        if !ts.insert(ts_name.clone()) {
            bail!("duplicate TS public name: {ts_name}");
        }
    }

    Ok(())
}

fn public_functions(manifest: &AbiManifest) -> Vec<&AbiFunction> {
    manifest
        .functions
        .iter()
        .filter(|function| function.public_ts_name.is_some())
        .collect()
}

fn public_curve_methods(manifest: &AbiManifest) -> Vec<&AbiFunction> {
    let mut methods: Vec<_> = manifest
        .functions
        .iter()
        .filter(|function| function.receiver.as_deref() == Some("curve"))
        .filter(|function| function.public_ts_name.is_some())
        .collect();
    methods.sort_by_key(|function| function.public_ts_name.clone().unwrap_or_default());
    methods
}

fn public_kernel_methods(manifest: &AbiManifest) -> Vec<&AbiFunction> {
    let mut methods: Vec<_> = manifest
        .functions
        .iter()
        .filter(|function| function.receiver.as_deref() == Some("kernel"))
        .filter(|function| {
            function
                .params
                .iter()
                .any(|param| param.ty.starts_with("*mut"))
        })
        .filter(|function| function.public_ts_name.is_some())
        .collect();
    methods.sort_by_key(|function| function.public_ts_name.clone().unwrap_or_default());
    methods
}

fn public_ts_catalog(manifest: &AbiManifest) -> Vec<String> {
    let mut names: Vec<String> = public_functions(manifest)
        .into_iter()
        .filter_map(|function| function.public_ts_name.clone())
        .collect();
    names.sort();
    names
}

fn find_by_c_name<'a>(manifest: &'a AbiManifest, c_name: &str) -> Result<&'a AbiFunction> {
    manifest
        .functions
        .iter()
        .find(|function| function.c_name == c_name)
        .ok_or_else(|| anyhow!("required ABI function not found: {c_name}"))
}

#[derive(Debug, Clone)]
enum KernelInputParam {
    Scalar(AbiParam),
    Array { ptr: AbiParam, inner: String },
}

fn kernel_public_input_params(params: &[AbiParam]) -> Vec<KernelInputParam> {
    let mut result = Vec::new();
    let mut index = 0;
    while index < params.len() {
        let param = &params[index];
        let name = param.name.trim_start_matches('_');

        if name == "session" || param.ty.starts_with("*mut") {
            index += 1;
            continue;
        }

        if let Some(inner) = param.ty.strip_prefix("*const") {
            if let Some(next) = params.get(index + 1) {
                let next_name = next.name.trim_start_matches('_');
                if next.ty == "usize" && next_name.ends_with("_count") {
                    result.push(KernelInputParam::Array {
                        ptr: param.clone(),
                        inner: inner.to_string(),
                    });
                    index += 2;
                    continue;
                }
            }
        }

        result.push(KernelInputParam::Scalar(param.clone()));
        index += 1;
    }

    result
}

fn map_ts_kernel_adapter_param(param: &KernelInputParam) -> String {
    match param {
        KernelInputParam::Scalar(param) => {
            format!("{}: {}", to_camel_case(&param.name), map_ts_type(&param.ty))
        }
        KernelInputParam::Array { ptr, inner, .. } => {
            format!("{}: {}[]", to_camel_case(&ptr.name), map_ts_type(inner))
        }
    }
}

fn map_ts_kernel_public_param(param: &KernelInputParam) -> String {
    map_ts_kernel_adapter_param(param)
}

fn kernel_ts_call_arg(param: &KernelInputParam) -> String {
    match param {
        KernelInputParam::Scalar(param) => to_camel_case(&param.name),
        KernelInputParam::Array { ptr, .. } => to_camel_case(&ptr.name),
    }
}

fn ordered_ffi_types(ffi_types: &[FfiType]) -> Vec<&FfiType> {
    let mut result = Vec::new();
    let mut defined = BTreeSet::new();

    let mut enums: Vec<&FfiType> = ffi_types
        .iter()
        .filter(|ffi_type| matches!(ffi_type, FfiType::Enum { .. }))
        .collect();
    enums.sort_by_key(|ffi_type| type_name(ffi_type));

    for ffi_type in enums {
        defined.insert(type_name(ffi_type));
        result.push(ffi_type);
    }

    let known_type_names: BTreeSet<String> = ffi_types.iter().map(type_name).collect();
    let mut unresolved: Vec<&FfiType> = ffi_types
        .iter()
        .filter(|ffi_type| matches!(ffi_type, FfiType::Struct { .. }))
        .collect();
    unresolved.sort_by_key(|ffi_type| type_name(ffi_type));

    while !unresolved.is_empty() {
        let mut progressed = false;
        let mut idx = 0;
        while idx < unresolved.len() {
            let ffi_type = unresolved[idx];
            let deps = ffi_type_dependencies(ffi_type, &known_type_names);
            if deps.iter().all(|dep| defined.contains(dep)) {
                defined.insert(type_name(ffi_type));
                result.push(ffi_type);
                unresolved.remove(idx);
                progressed = true;
            } else {
                idx += 1;
            }
        }

        if !progressed {
            for ffi_type in unresolved.drain(..) {
                result.push(ffi_type);
            }
        }
    }

    result
}

fn ffi_type_dependencies(ffi_type: &FfiType, known_type_names: &BTreeSet<String>) -> Vec<String> {
    let mut deps = BTreeSet::new();
    if let FfiType::Struct { name, fields, .. } = ffi_type {
        for field in fields {
            let base = strip_pointer_type(&field.ty);
            if known_type_names.contains(&base) && base != *name {
                deps.insert(base);
            }
        }
    }
    deps.into_iter().collect()
}

fn strip_pointer_type(ty: &str) -> String {
    if let Some(inner) = ty.strip_prefix("*mut") {
        return strip_pointer_type(inner);
    }
    if let Some(inner) = ty.strip_prefix("*const") {
        return strip_pointer_type(inner);
    }
    ty.to_string()
}

fn type_name(ffi_type: &FfiType) -> String {
    match ffi_type {
        FfiType::Enum { name, .. } => name.clone(),
        FfiType::Struct { name, .. } => name.clone(),
    }
}

fn to_pascal_case(input: &str) -> String {
    input
        .split('_')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect()
}

fn to_camel_case(input: &str) -> String {
    let pascal = to_pascal_case(input);
    let mut chars = pascal.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_ascii_lowercase(), chars.as_str()),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn naming_rules_are_stable() {
        assert_eq!(to_pascal_case("rgm_curve_point_at"), "RgmCurvePointAt");
        assert_eq!(to_camel_case("curve_point_at"), "curvePointAt");
    }

    #[test]
    fn manifest_hash_changes_when_public_names_change() {
        let base = AbiManifest {
            crate_name: "kernel-ffi".to_string(),
            version: "1.0.0".to_string(),
            abi_hash: String::new(),
            functions: vec![AbiFunction {
                rust_name: "rgm_curve_point_at".to_string(),
                c_name: "rgm_curve_point_at".to_string(),
                interop_ts_name: "interopCurvePointAt".to_string(),
                public_ts_name: Some("pointAt".to_string()),
                receiver: Some("curve".to_string()),
                return_type: "RgmStatus".to_string(),
                params: vec![AbiParam {
                    name: "t_norm".to_string(),
                    ty: "f64".to_string(),
                }],
            }],
            ffi_types: Vec::new(),
        };

        let mut changed = base.clone();
        changed.functions[0].public_ts_name = Some("d0".to_string());

        let hash_a = compute_abi_hash(&base).expect("hash");
        let hash_b = compute_abi_hash(&changed).expect("hash");
        assert_ne!(hash_a, hash_b);
    }
}
