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
    interop_dotnet_name: String,
    interop_ts_name: String,
    compat_ts_name: String,
    public_dotnet_name: Option<String>,
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
    dotnet: Option<String>,
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

    enforce_catalog_parity(&manifest)?;

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
                        interop_dotnet_name: format!("Interop{}", to_pascal_case(&base_name)),
                        interop_ts_name: format!("interop{}", to_pascal_case(&base_name)),
                        compat_ts_name: to_camel_case(&base_name),
                        public_dotnet_name: meta.dotnet,
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
        if nested.path.is_ident("dotnet") {
            let value = nested.value()?.parse::<syn::LitStr>()?;
            meta.dotnet = Some(value.value());
            return Ok(());
        }

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

    for function in &manifest.functions {
        if function.public_dotnet_name.is_some() ^ function.public_ts_name.is_some() {
            bail!(
                "function {} has incomplete public naming metadata",
                function.c_name
            );
        }
    }

    Ok(())
}

fn generate_outputs(workspace: &Path, manifest: &AbiManifest) -> Result<Vec<OutputFile>> {
    let manifest_json = serde_json::to_string_pretty(manifest)? + "\n";
    let c_header = generate_c_header(manifest)?;
    let dotnet_models = generate_dotnet_models(manifest)?;
    let dotnet_native = generate_dotnet_native_methods(manifest)?;
    let dotnet_safe_handles = generate_dotnet_safe_handles(manifest)?;
    let dotnet_public = generate_dotnet_public_api(manifest)?;
    let dotnet_compat = generate_dotnet_compat_api(manifest)?;
    let dotnet_catalog = serde_json::to_string_pretty(&public_dotnet_catalog(manifest))? + "\n";

    let ts_types = generate_typescript_types(manifest)?;
    let ts_generated = generate_typescript_api(manifest)?;
    let ts_catalog = serde_json::to_string_pretty(&public_ts_catalog(manifest))? + "\n";

    Ok(vec![
        OutputFile {
            path: workspace.join("target/abi/rgm_abi.json"),
            content: manifest_json,
        },
        OutputFile {
            path: workspace.join("include/rusted_geom.h"),
            content: c_header,
        },
        OutputFile {
            path: workspace.join("bindings/dotnet/src/Generated/Models.g.cs"),
            content: dotnet_models,
        },
        OutputFile {
            path: workspace.join("bindings/dotnet/src/Generated/NativeMethods.g.cs"),
            content: dotnet_native,
        },
        OutputFile {
            path: workspace.join("bindings/dotnet/src/Generated/SafeHandles.g.cs"),
            content: dotnet_safe_handles,
        },
        OutputFile {
            path: workspace.join("bindings/dotnet/src/Generated/PublicApi.g.cs"),
            content: dotnet_public,
        },
        OutputFile {
            path: workspace.join("bindings/dotnet/src/Generated/RustedGeomApi.g.cs"),
            content: dotnet_compat,
        },
        OutputFile {
            path: workspace.join("bindings/dotnet/src/Generated/function_catalog.json"),
            content: dotnet_catalog,
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

fn generate_c_header(manifest: &AbiManifest) -> Result<String> {
    let mut out = String::new();
    writeln!(out, "// @generated by abi-gen")?;
    writeln!(out, "// version: {}", manifest.version)?;
    writeln!(out, "// abi_hash: {}", manifest.abi_hash)?;
    writeln!(out, "#ifndef RUSTED_GEOM_H")?;
    writeln!(out, "#define RUSTED_GEOM_H")?;
    writeln!(out)?;
    writeln!(out, "#include <stdbool.h>")?;
    writeln!(out, "#include <stddef.h>")?;
    writeln!(out, "#include <stdint.h>")?;
    writeln!(out)?;

    for ffi_type in ordered_ffi_types(&manifest.ffi_types) {
        match ffi_type {
            FfiType::Enum {
                name,
                repr: _,
                variants,
            } => {
                writeln!(out, "typedef enum {name} {{")?;
                for variant in variants {
                    if let Some(value) = &variant.value {
                        writeln!(out, "  {name}_{0} = {1},", variant.name, value)?;
                    } else {
                        writeln!(out, "  {name}_{},", variant.name)?;
                    }
                }
                writeln!(out, "}} {name};")?;
                writeln!(out)?;
            }
            FfiType::Struct { name, fields, .. } => {
                writeln!(out, "typedef struct {name} {{")?;
                if fields.is_empty() {
                    writeln!(out, "  uint8_t _unused;")?;
                } else {
                    for field in fields {
                        let param_name = field.name.trim_start_matches('_');
                        writeln!(out, "  {} {};", map_c_type(&field.ty), param_name)?;
                    }
                }
                writeln!(out, "}} {name};")?;
                writeln!(out)?;
            }
        }
    }

    for function in &manifest.functions {
        write!(
            out,
            "{} {}(",
            map_c_type(&function.return_type),
            function.c_name
        )?;
        for (idx, param) in function.params.iter().enumerate() {
            if idx > 0 {
                write!(out, ", ")?;
            }
            let param_name = param.name.trim_start_matches('_');
            write!(out, "{} {}", map_c_type(&param.ty), param_name)?;
        }
        writeln!(out, ");")?;
    }

    writeln!(out)?;
    writeln!(out, "#endif // RUSTED_GEOM_H")?;

    Ok(out)
}

fn generate_dotnet_models(manifest: &AbiManifest) -> Result<String> {
    let mut out = String::new();
    writeln!(out, "// <auto-generated />")?;
    writeln!(out, "// version: {}", manifest.version)?;
    writeln!(out, "// abi_hash: {}", manifest.abi_hash)?;
    writeln!(out, "using System.Runtime.InteropServices;")?;
    writeln!(out)?;
    writeln!(out, "namespace RustedGeom.Generated;")?;
    writeln!(out)?;

    for ffi_type in ordered_ffi_types(&manifest.ffi_types) {
        match ffi_type {
            FfiType::Enum {
                name,
                repr,
                variants,
            } => {
                writeln!(out, "public enum {name} : {}", map_dotnet_base_type(repr))?;
                writeln!(out, "{{")?;
                for variant in variants {
                    if let Some(value) = &variant.value {
                        writeln!(out, "    {} = {},", variant.name, value)?;
                    } else {
                        writeln!(out, "    {},", variant.name)?;
                    }
                }
                writeln!(out, "}}")?;
                writeln!(out)?;
            }
            FfiType::Struct { name, fields, repr } => {
                writeln!(out, "[StructLayout(LayoutKind.Sequential)]")?;
                writeln!(out, "public partial struct {name}")?;
                writeln!(out, "{{")?;
                if fields.is_empty() {
                    writeln!(out, "    public byte _unused;")?;
                } else {
                    for field in fields {
                        let field_name = field.name.trim_start_matches('_');
                        writeln!(
                            out,
                            "    public {} {};",
                            map_dotnet_field_type(&field.ty),
                            to_pascal_case(field_name)
                        )?;
                    }

                    if let Some(inner) = transparent_newtype_inner(repr, fields) {
                        let storage_name = to_pascal_case(fields[0].name.trim_start_matches('_'));
                        if storage_name != "Value" {
                            writeln!(out)?;
                            writeln!(out, "    public {} Value", map_dotnet_field_type(inner))?;
                            writeln!(out, "    {{")?;
                            writeln!(out, "        readonly get => {storage_name};")?;
                            writeln!(out, "        set => {storage_name} = value;")?;
                            writeln!(out, "    }}")?;
                        }
                    }
                }
                writeln!(out, "}}")?;
                writeln!(out)?;
            }
        }
    }

    Ok(out)
}

fn generate_dotnet_native_methods(manifest: &AbiManifest) -> Result<String> {
    let mut out = String::new();
    writeln!(out, "// <auto-generated />")?;
    writeln!(out, "// version: {}", manifest.version)?;
    writeln!(out, "// abi_hash: {}", manifest.abi_hash)?;
    writeln!(out, "using System.Runtime.InteropServices;")?;
    writeln!(out)?;
    writeln!(out, "namespace RustedGeom.Generated;")?;
    writeln!(out)?;
    writeln!(out, "internal static unsafe partial class NativeMethods")?;
    writeln!(out, "{{")?;
    writeln!(
        out,
        "    internal const string NativeLibrary = \"rusted_geom\";"
    )?;
    writeln!(out)?;

    for function in &manifest.functions {
        writeln!(
            out,
            "    [LibraryImport(NativeLibrary, EntryPoint = \"{}\")]",
            function.c_name
        )?;

        write!(
            out,
            "    internal static partial {} {}(",
            map_dotnet_field_type(&function.return_type),
            function.interop_dotnet_name
        )?;
        for (idx, param) in function.params.iter().enumerate() {
            if idx > 0 {
                write!(out, ", ")?;
            }
            write!(out, "{}", map_dotnet_param(param))?;
        }
        writeln!(out, ");")?;
        writeln!(out)?;
    }

    writeln!(out, "}}")?;
    Ok(out)
}

fn generate_dotnet_safe_handles(manifest: &AbiManifest) -> Result<String> {
    let kernel_destroy = find_by_c_name(manifest, "rgm_kernel_destroy")?;
    let object_release = find_by_c_name(manifest, "rgm_object_release")?;

    let mut out = String::new();
    writeln!(out, "// <auto-generated />")?;
    writeln!(out, "// version: {}", manifest.version)?;
    writeln!(out, "// abi_hash: {}", manifest.abi_hash)?;
    writeln!(out, "using System;")?;
    writeln!(out, "using Microsoft.Win32.SafeHandles;")?;
    writeln!(out)?;
    writeln!(out, "namespace RustedGeom.Generated;")?;
    writeln!(out)?;
    writeln!(
        out,
        "public sealed class RgmKernelSafeHandle : SafeHandleZeroOrMinusOneIsInvalid"
    )?;
    writeln!(out, "{{")?;
    writeln!(out, "    public RgmKernelSafeHandle() : base(true) {{ }}")?;
    writeln!(out, "    public RgmKernelHandle KernelHandle => new() {{ Value = unchecked((ulong)handle.ToInt64()) }};")?;
    writeln!(out)?;
    writeln!(out, "    protected override bool ReleaseHandle()")?;
    writeln!(out, "    {{")?;
    writeln!(
        out,
        "        var status = NativeMethods.{}(KernelHandle);",
        kernel_destroy.interop_dotnet_name
    )?;
    writeln!(
        out,
        "        return status is RgmStatus.Ok or RgmStatus.NotFound;"
    )?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;
    writeln!(out)?;

    writeln!(
        out,
        "public sealed class RgmObjectSafeHandle : SafeHandleZeroOrMinusOneIsInvalid"
    )?;
    writeln!(out, "{{")?;
    writeln!(out, "    private readonly RgmKernelHandle _session;")?;
    writeln!(out)?;
    writeln!(out, "    public RgmObjectSafeHandle(RgmKernelHandle session, RgmObjectHandle objectHandle) : base(true)")?;
    writeln!(out, "    {{")?;
    writeln!(out, "        _session = session;")?;
    writeln!(
        out,
        "        SetHandle(unchecked((IntPtr)(long)objectHandle.Value));"
    )?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    protected override bool ReleaseHandle()")?;
    writeln!(out, "    {{")?;
    writeln!(out, "        var objectHandle = new RgmObjectHandle {{ Value = unchecked((ulong)handle.ToInt64()) }};")?;
    writeln!(
        out,
        "        var status = NativeMethods.{}(_session, objectHandle);",
        object_release.interop_dotnet_name
    )?;
    writeln!(
        out,
        "        return status is RgmStatus.Ok or RgmStatus.NotFound;"
    )?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;

    Ok(out)
}

fn generate_dotnet_public_api(manifest: &AbiManifest) -> Result<String> {
    let kernel_create = find_by_c_name(manifest, "rgm_kernel_create")?;
    let kernel_destroy = find_by_c_name(manifest, "rgm_kernel_destroy")?;
    let object_release = find_by_c_name(manifest, "rgm_object_release")?;
    let last_error_message = find_by_c_name(manifest, "rgm_last_error_message")?;

    let constructor = find_public(manifest, "kernel", "InterpolateNurbsFitPoints").ok();
    let curve_methods = public_curve_methods(manifest);

    let mut out = String::new();
    writeln!(out, "// <auto-generated />")?;
    writeln!(out, "// version: {}", manifest.version)?;
    writeln!(out, "// abi_hash: {}", manifest.abi_hash)?;
    writeln!(out, "using System;")?;
    writeln!(out, "using System.Text;")?;
    writeln!(out)?;
    writeln!(out, "namespace RustedGeom.Generated;")?;
    writeln!(out)?;

    writeln!(out, "public sealed class RgmException : Exception")?;
    writeln!(out, "{{")?;
    writeln!(out, "    public RgmStatus Status {{ get; }}")?;
    writeln!(out, "")?;
    writeln!(
        out,
        "    public RgmException(RgmStatus status, string message) : base(message)"
    )?;
    writeln!(out, "    {{")?;
    writeln!(out, "        Status = status;")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;
    writeln!(out)?;

    writeln!(out, "internal static unsafe class NativeError")?;
    writeln!(out, "{{")?;
    writeln!(out, "    internal static void ThrowIfError(RgmStatus status, RgmKernelHandle session, string context)")?;
    writeln!(out, "    {{")?;
    writeln!(out, "        if (status == RgmStatus.Ok)")?;
    writeln!(out, "        {{")?;
    writeln!(out, "            return;")?;
    writeln!(out, "        }}")?;
    writeln!(out, "")?;
    writeln!(
        out,
        "        var message = TryGetLastErrorMessage(session);"
    )?;
    writeln!(out, "        if (string.IsNullOrWhiteSpace(message))")?;
    writeln!(out, "        {{")?;
    writeln!(out, "            message = context;")?;
    writeln!(out, "        }}")?;
    writeln!(out, "")?;
    writeln!(
        out,
        "        throw new RgmException(status, $\"{{context}}: {{message}}\");"
    )?;
    writeln!(out, "    }}")?;
    writeln!(out, "")?;
    writeln!(
        out,
        "    private static string TryGetLastErrorMessage(RgmKernelHandle session)"
    )?;
    writeln!(out, "    {{")?;
    writeln!(
        out,
        "        var status = NativeMethods.{}(session, null, 0, out nuint written);",
        last_error_message.interop_dotnet_name
    )?;
    writeln!(out, "        if (status != RgmStatus.Ok || written == 0)")?;
    writeln!(out, "        {{")?;
    writeln!(out, "            return string.Empty;")?;
    writeln!(out, "        }}")?;
    writeln!(out, "")?;
    writeln!(out, "        var buffer = new byte[(int)written + 1];")?;
    writeln!(out, "        fixed (byte* ptr = buffer)")?;
    writeln!(out, "        {{")?;
    writeln!(
        out,
        "            status = NativeMethods.{}(session, ptr, (nuint)buffer.Length, out written);",
        last_error_message.interop_dotnet_name
    )?;
    writeln!(
        out,
        "            if (status != RgmStatus.Ok || written == 0)"
    )?;
    writeln!(out, "            {{")?;
    writeln!(out, "                return string.Empty;")?;
    writeln!(out, "            }}")?;
    writeln!(
        out,
        "            return Encoding.UTF8.GetString(buffer, 0, (int)written);"
    )?;
    writeln!(out, "        }}")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;
    writeln!(out)?;

    writeln!(out, "public sealed unsafe class KernelHandle : IDisposable")?;
    writeln!(out, "{{")?;
    writeln!(out, "    private RgmKernelHandle _session;")?;
    writeln!(out, "    private bool _disposed;")?;
    writeln!(out, "")?;
    writeln!(out, "    private KernelHandle(RgmKernelHandle session)")?;
    writeln!(out, "    {{")?;
    writeln!(out, "        _session = session;")?;
    writeln!(out, "    }}")?;
    writeln!(out, "")?;
    writeln!(out, "    internal RgmKernelHandle Session => _session;")?;
    writeln!(out, "")?;
    writeln!(out, "    public static KernelHandle Create()")?;
    writeln!(out, "    {{")?;
    writeln!(
        out,
        "        var status = NativeMethods.{}(out var session);",
        kernel_create.interop_dotnet_name
    )?;
    writeln!(
        out,
        "        NativeError.ThrowIfError(status, session, \"Kernel.Create\");"
    )?;
    writeln!(out, "        return new KernelHandle(session);")?;
    writeln!(out, "    }}")?;
    writeln!(out, "")?;
    writeln!(out, "    public void Dispose()")?;
    writeln!(out, "    {{")?;
    writeln!(out, "        if (_disposed)")?;
    writeln!(out, "        {{")?;
    writeln!(out, "            return;")?;
    writeln!(out, "        }}")?;
    writeln!(out, "")?;
    writeln!(
        out,
        "        var status = NativeMethods.{}(_session);",
        kernel_destroy.interop_dotnet_name
    )?;
    writeln!(
        out,
        "        if (status != RgmStatus.Ok && status != RgmStatus.NotFound)"
    )?;
    writeln!(out, "        {{")?;
    writeln!(
        out,
        "            NativeError.ThrowIfError(status, _session, \"Kernel.Dispose\");"
    )?;
    writeln!(out, "        }}")?;
    writeln!(out, "")?;
    writeln!(out, "        _disposed = true;")?;
    writeln!(out, "    }}")?;

    if let Some(constructor) = constructor {
        writeln!(out, "")?;
        writeln!(out, "    public CurveHandle InterpolateNurbsFitPoints(ReadOnlySpan<RgmPoint3> points, uint degree, bool closed, RgmToleranceContext tol)")?;
        writeln!(out, "    {{")?;
        writeln!(out, "        if (points.Length == 0)")?;
        writeln!(out, "        {{")?;
        writeln!(out, "            throw new ArgumentException(\"At least one point is required\", nameof(points));")?;
        writeln!(out, "        }}")?;
        writeln!(out, "")?;
        writeln!(out, "        fixed (RgmPoint3* pointsPtr = points)")?;
        writeln!(out, "        {{")?;
        writeln!(out, "            var status = NativeMethods.{}(_session, pointsPtr, (nuint)points.Length, degree, closed, tol, out var curve);", constructor.interop_dotnet_name)?;
        writeln!(out, "            NativeError.ThrowIfError(status, _session, \"Kernel.InterpolateNurbsFitPoints\");")?;
        writeln!(out, "            return new CurveHandle(this, curve);")?;
        writeln!(out, "        }}")?;
        writeln!(out, "    }}")?;
    }

    writeln!(out, "}}")?;
    writeln!(out)?;

    writeln!(out, "public sealed unsafe class CurveHandle : IDisposable")?;
    writeln!(out, "{{")?;
    writeln!(out, "    private readonly KernelHandle _kernel;")?;
    writeln!(out, "    private RgmObjectHandle _curve;")?;
    writeln!(out, "    private bool _disposed;")?;
    writeln!(out, "")?;
    writeln!(
        out,
        "    internal CurveHandle(KernelHandle kernel, RgmObjectHandle curve)"
    )?;
    writeln!(out, "    {{")?;
    writeln!(out, "        _kernel = kernel;")?;
    writeln!(out, "        _curve = curve;")?;
    writeln!(out, "    }}")?;
    writeln!(out, "")?;
    writeln!(out, "    public void Dispose()")?;
    writeln!(out, "    {{")?;
    writeln!(out, "        if (_disposed)")?;
    writeln!(out, "        {{")?;
    writeln!(out, "            return;")?;
    writeln!(out, "        }}")?;
    writeln!(out, "")?;
    writeln!(
        out,
        "        var status = NativeMethods.{}(_kernel.Session, _curve);",
        object_release.interop_dotnet_name
    )?;
    writeln!(
        out,
        "        if (status != RgmStatus.Ok && status != RgmStatus.NotFound)"
    )?;
    writeln!(out, "        {{")?;
    writeln!(
        out,
        "            NativeError.ThrowIfError(status, _kernel.Session, \"Curve.Dispose\");"
    )?;
    writeln!(out, "        }}")?;
    writeln!(out, "")?;
    writeln!(out, "        _disposed = true;")?;
    writeln!(out, "    }}")?;
    writeln!(out, "")?;
    writeln!(out, "    private void ThrowIfDisposed()")?;
    writeln!(out, "    {{")?;
    writeln!(out, "        if (_disposed)")?;
    writeln!(out, "        {{")?;
    writeln!(
        out,
        "            throw new ObjectDisposedException(nameof(CurveHandle));"
    )?;
    writeln!(out, "        }}")?;
    writeln!(out, "    }}")?;

    for function in curve_methods {
        let method_name = function
            .public_dotnet_name
            .as_ref()
            .expect("public curve method should have dotnet name");
        let (return_type, out_param_name) = extract_out_return(&function.params)?;
        let input_params = user_input_params(&function.params);

        writeln!(out, "")?;
        write!(out, "    public {} {}(", return_type, method_name)?;
        for (idx, param) in input_params.iter().enumerate() {
            if idx > 0 {
                write!(out, ", ")?;
            }
            write!(out, "{}", map_dotnet_public_param(param))?;
        }
        writeln!(out, ")")?;
        writeln!(out, "    {{")?;
        writeln!(out, "        ThrowIfDisposed();")?;
        write!(
            out,
            "        var status = NativeMethods.{}(_kernel.Session, _curve",
            function.interop_dotnet_name
        )?;
        for param in &input_params {
            write!(
                out,
                ", {}",
                sanitize_csharp_identifier(&to_camel_case(&param.name))
            )?;
        }
        writeln!(
            out,
            ", out var {});",
            sanitize_csharp_identifier(&to_camel_case(out_param_name))
        )?;
        writeln!(
            out,
            "        NativeError.ThrowIfError(status, _kernel.Session, \"Curve.{}\");",
            method_name
        )?;
        writeln!(
            out,
            "        return {};",
            sanitize_csharp_identifier(&to_camel_case(out_param_name))
        )?;
        writeln!(out, "    }}")?;
    }

    writeln!(out, "}}")?;

    Ok(out)
}

fn generate_dotnet_compat_api(_manifest: &AbiManifest) -> Result<String> {
    let mut out = String::new();
    writeln!(out, "// <auto-generated />")?;
    writeln!(out, "namespace RustedGeom.Generated;")?;
    writeln!(out)?;
    writeln!(out, "public static class RustedGeomApi")?;
    writeln!(out, "{{")?;
    writeln!(
        out,
        "    public static KernelHandle CreateKernel() => KernelHandle.Create();"
    )?;
    writeln!(out, "}}")?;
    Ok(out)
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

    let constructor = find_public(manifest, "kernel", "InterpolateNurbsFitPoints").ok();
    let curve_methods = public_curve_methods(manifest);

    let mut out = String::new();
    writeln!(out, "// @generated by abi-gen")?;
    writeln!(out, "// version: {}", manifest.version)?;
    writeln!(out, "// abi_hash: {}", manifest.abi_hash)?;
    writeln!(out)?;
    writeln!(
        out,
        "import type {{ RgmKernelHandle, RgmObjectHandle, RgmPlane, RgmPoint3, RgmToleranceContext, RgmVec3 }} from \"./types\";"
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

    writeln!(out, "export class RustedGeomApi {{")?;
    writeln!(
        out,
        "  constructor(private readonly native: NativeExports) {{}}"
    )?;
    writeln!(out)?;
    for function in &manifest.functions {
        write!(out, "  {}(", function.compat_ts_name)?;
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
            "): {} {{",
            map_ts_native_return_type(&function.return_type)
        )?;
        writeln!(
            out,
            "    return this.native.{}({});",
            function.c_name,
            function
                .params
                .iter()
                .map(|param| to_camel_case(&param.name))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        writeln!(out, "  }}")?;
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

    if let Some(constructor) = constructor {
        writeln!(
            out,
            "  {}(session: bigint, points: RgmPoint3[], degree: number, closed: boolean, tol: RgmToleranceContext): {{ status: RgmStatus; value: bigint }};",
            constructor.interop_ts_name
        )?;
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

    if let Some(constructor) = constructor {
        writeln!(out)?;
        writeln!(out, "  interpolateNurbsFitPoints(points: RgmPoint3[], degree: number, closed: boolean, tol: RgmToleranceContext): CurveHandle {{")?;
        writeln!(
            out,
            "    if (points.length === 0) throw new Error(\"At least one point is required\");"
        )?;
        writeln!(out, "    this.throwIfDisposed();")?;
        writeln!(
            out,
            "    const result = this.adapter.{}(this.session, points, degree, closed, tol);",
            constructor.interop_ts_name
        )?;
        writeln!(out, "    throwIfError(this.adapter, this.session, result.status, \"KernelHandle.interpolateNurbsFitPoints\");")?;
        writeln!(
            out,
            "    return new CurveHandle(this.adapter, this.session, result.value);"
        )?;
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
            "  {{ c: \"{}\", interopDotnet: \"{}\", interopTs: \"{}\", compatTs: \"{}\", dotnet: {}, ts: {} }},",
            function.c_name,
            function.interop_dotnet_name,
            function.interop_ts_name,
            function.compat_ts_name,
            function
                .public_dotnet_name
                .as_ref()
                .map(|value| format!("\"{}\"", value))
                .unwrap_or_else(|| "null".to_string()),
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

fn map_c_type(rust_type: &str) -> String {
    if let Some(inner) = rust_type.strip_prefix("*mut") {
        return format!("{}*", map_c_type(inner));
    }

    if let Some(inner) = rust_type.strip_prefix("*const") {
        return format!("const {}*", map_c_type(inner));
    }

    match rust_type {
        "()" => "void".to_string(),
        "bool" => "bool".to_string(),
        "u8" => "uint8_t".to_string(),
        "u32" => "uint32_t".to_string(),
        "u64" => "uint64_t".to_string(),
        "usize" => "size_t".to_string(),
        "i32" => "int32_t".to_string(),
        "f64" => "double".to_string(),
        other => other.to_string(),
    }
}

fn map_dotnet_base_type(rust_type: &str) -> &'static str {
    match rust_type {
        "i32" => "int",
        "u32" => "uint",
        "u64" => "ulong",
        "usize" => "nuint",
        _ => "int",
    }
}

fn map_dotnet_field_type(rust_type: &str) -> String {
    match rust_type {
        "()" => "void".to_string(),
        "bool" => "bool".to_string(),
        "u8" => "byte".to_string(),
        "u32" => "uint".to_string(),
        "u64" => "ulong".to_string(),
        "usize" => "nuint".to_string(),
        "i32" => "int".to_string(),
        "f64" => "double".to_string(),
        other => other.to_string(),
    }
}

fn map_dotnet_param(param: &AbiParam) -> String {
    let ty = param.ty.as_str();
    let name = sanitize_csharp_identifier(&to_camel_case(&param.name));

    match ty {
        "bool" => format!("[MarshalAs(UnmanagedType.I1)] bool {name}"),
        "u32" => format!("uint {name}"),
        "u64" => format!("ulong {name}"),
        "usize" => format!("nuint {name}"),
        "i32" => format!("int {name}"),
        "f64" => format!("double {name}"),
        _ => {
            if let Some(inner) = ty.strip_prefix("*mut") {
                return map_dotnet_mut_ptr_param(inner, &name);
            }
            if let Some(inner) = ty.strip_prefix("*const") {
                return map_dotnet_const_ptr_param(inner, &name);
            }
            format!("{} {name}", map_dotnet_field_type(ty))
        }
    }
}

fn map_dotnet_public_param(param: &AbiParam) -> String {
    let ty = param.ty.as_str();
    let name = sanitize_csharp_identifier(&to_camel_case(&param.name));

    match ty {
        "bool" => format!("bool {name}"),
        "u32" => format!("uint {name}"),
        "u64" => format!("ulong {name}"),
        "usize" => format!("nuint {name}"),
        "i32" => format!("int {name}"),
        "f64" => format!("double {name}"),
        _ => format!("{} {name}", map_dotnet_field_type(ty)),
    }
}

fn map_dotnet_mut_ptr_param(inner: &str, name: &str) -> String {
    match inner {
        "RgmKernelHandle" => format!("out RgmKernelHandle {name}"),
        "RgmObjectHandle" => format!("out RgmObjectHandle {name}"),
        "RgmPoint3" => format!("out RgmPoint3 {name}"),
        "RgmVec3" => format!("out RgmVec3 {name}"),
        "RgmPlane" => format!("out RgmPlane {name}"),
        "i32" => format!("out int {name}"),
        "usize" => format!("out nuint {name}"),
        "u8" => format!("byte* {name}"),
        _ => format!("IntPtr {name}"),
    }
}

fn map_dotnet_const_ptr_param(inner: &str, name: &str) -> String {
    match inner {
        "RgmPoint3" => format!("RgmPoint3* {name}"),
        "u8" => format!("byte* {name}"),
        _ => format!("IntPtr {name}"),
    }
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

fn extract_out_return(params: &[AbiParam]) -> Result<(String, &str)> {
    let out_param = params
        .iter()
        .rev()
        .find(|param| param.ty.starts_with("*mut"))
        .ok_or_else(|| anyhow!("expected out parameter in generated public method"))?;
    let inner = out_param
        .ty
        .strip_prefix("*mut")
        .ok_or_else(|| anyhow!("invalid out parameter type"))?;
    Ok((map_dotnet_field_type(inner), out_param.name.as_str()))
}

fn ts_return_type(function: &AbiFunction) -> Result<(String, String)> {
    let out_param = function
        .params
        .iter()
        .rev()
        .find(|param| param.ty.starts_with("*mut"))
        .ok_or_else(|| anyhow!("expected out parameter in generated TS method"))?;
    let inner = out_param
        .ty
        .strip_prefix("*mut")
        .ok_or_else(|| anyhow!("invalid TS out parameter type"))?;
    Ok((map_ts_type(inner), out_param.name.clone()))
}

fn sanitize_csharp_identifier(name: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "abstract",
        "as",
        "base",
        "bool",
        "break",
        "byte",
        "case",
        "catch",
        "char",
        "checked",
        "class",
        "const",
        "continue",
        "decimal",
        "default",
        "delegate",
        "do",
        "double",
        "else",
        "enum",
        "event",
        "explicit",
        "extern",
        "false",
        "finally",
        "fixed",
        "float",
        "for",
        "foreach",
        "goto",
        "if",
        "implicit",
        "in",
        "int",
        "interface",
        "internal",
        "is",
        "lock",
        "long",
        "namespace",
        "new",
        "null",
        "object",
        "operator",
        "out",
        "override",
        "params",
        "private",
        "protected",
        "public",
        "readonly",
        "ref",
        "return",
        "sbyte",
        "sealed",
        "short",
        "sizeof",
        "stackalloc",
        "static",
        "string",
        "struct",
        "switch",
        "this",
        "throw",
        "true",
        "try",
        "typeof",
        "uint",
        "ulong",
        "unchecked",
        "unsafe",
        "ushort",
        "using",
        "virtual",
        "void",
        "volatile",
        "while",
    ];

    if KEYWORDS.contains(&name) {
        format!("@{name}")
    } else {
        name.to_string()
    }
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

fn enforce_catalog_parity(manifest: &AbiManifest) -> Result<()> {
    let public = public_functions(manifest);
    let mut dotnet = BTreeSet::new();
    let mut ts = BTreeSet::new();

    for function in public {
        let dotnet_name = function
            .public_dotnet_name
            .as_ref()
            .ok_or_else(|| anyhow!("missing dotnet public name for {}", function.c_name))?;
        let ts_name = function
            .public_ts_name
            .as_ref()
            .ok_or_else(|| anyhow!("missing ts public name for {}", function.c_name))?;

        if !dotnet.insert(dotnet_name.clone()) {
            bail!("duplicate .NET public name: {dotnet_name}");
        }
        if !ts.insert(ts_name.clone()) {
            bail!("duplicate TS public name: {ts_name}");
        }
    }

    if dotnet.len() != ts.len() {
        bail!("catalog parity mismatch between .NET and TS public APIs");
    }

    Ok(())
}

fn public_functions(manifest: &AbiManifest) -> Vec<&AbiFunction> {
    manifest
        .functions
        .iter()
        .filter(|function| {
            function.public_dotnet_name.is_some() || function.public_ts_name.is_some()
        })
        .collect()
}

fn public_curve_methods(manifest: &AbiManifest) -> Vec<&AbiFunction> {
    let mut methods: Vec<_> = manifest
        .functions
        .iter()
        .filter(|function| function.receiver.as_deref() == Some("curve"))
        .filter(|function| {
            function.public_dotnet_name.is_some() && function.public_ts_name.is_some()
        })
        .collect();
    methods.sort_by_key(|function| function.public_dotnet_name.clone().unwrap_or_default());
    methods
}

fn public_dotnet_catalog(manifest: &AbiManifest) -> Vec<String> {
    let mut names: Vec<String> = public_functions(manifest)
        .into_iter()
        .filter_map(|function| function.public_dotnet_name.clone())
        .collect();
    names.sort();
    names
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

fn find_public<'a>(
    manifest: &'a AbiManifest,
    receiver: &str,
    dotnet_name: &str,
) -> Result<&'a AbiFunction> {
    manifest
        .functions
        .iter()
        .find(|function| {
            function.receiver.as_deref() == Some(receiver)
                && function.public_dotnet_name.as_deref() == Some(dotnet_name)
        })
        .ok_or_else(|| {
            anyhow!("required public function not found: receiver={receiver}, dotnet={dotnet_name}")
        })
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
                interop_dotnet_name: "InteropCurvePointAt".to_string(),
                interop_ts_name: "interopCurvePointAt".to_string(),
                compat_ts_name: "curvePointAt".to_string(),
                public_dotnet_name: Some("PointAt".to_string()),
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
        changed.functions[0].public_dotnet_name = Some("D0".to_string());

        let hash_a = compute_abi_hash(&base).expect("hash");
        let hash_b = compute_abi_hash(&changed).expect("hash");
        assert_ne!(hash_a, hash_b);
    }
}
