use bindgen;
use cc;
use quote::*;
use regex::Regex;
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use syn::*;

const BASE_TYPES: [&'static str; 53] = [
    "GLADloadproc",
    "khronos_int8_t",
    "khronos_uint8_t",
    "khronos_int16_t",
    "khronos_uint16_t",
    "khronos_int32_t",
    "khronos_float_t",
    "khronos_intptr_t",
    "khronos_ssize_t",
    "khronos_int64_t",
    "khronos_uint64_t",
    "GLenum",
    "GLboolean",
    "GLbitfield",
    "GLvoid",
    "GLbyte",
    "GLubyte",
    "GLshort",
    "GLushort",
    "GLint",
    "GLuint",
    "GLclampx",
    "GLsizei",
    "GLfloat",
    "GLclampf",
    "GLdouble",
    "GLclampd",
    "GLeglClientBufferEXT",
    "GLeglImageOES",
    "GLchar",
    "GLcharARB",
    "GLhandleARB",
    "GLhalf",
    "GLhalfARB",
    "GLfixed",
    "GLintptr",
    "GLintptrARB",
    "GLsizeiptr",
    "GLsizeiptrARB",
    "GLint64",
    "GLint64EXT",
    "GLuint64",
    "GLuint64EXT",
    "GLsync",
    "_cl_context",
    "_cl_event",
    "GLDEBUGPROC",
    "GLDEBUGPROCARB",
    "GLDEBUGPROCKHR",
    "GLDEBUGPROCAMD",
    "GLhalfNV",
    "GLvdpauSurfaceNV",
    "GLVULKANPROCNV",
];

// https://stackoverflow.com/questions/55271857/how-can-i-get-the-t-from-an-optiont-when-using-syn#56264023
fn extract_type_from_option(ty: &syn::Type) -> Option<&syn::Type> {
    use syn::{GenericArgument, Path, PathArguments, PathSegment};

    fn extract_type_path(ty: &syn::Type) -> Option<&Path> {
        match ty {
            syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
            _ => None,
        }
    }

    fn extract_option_segment(path: &Path) -> Option<&PathSegment> {
        let idents_of_path = path
            .segments
            .iter()
            .into_iter()
            .fold(String::new(), |mut acc, v| {
                acc.push_str(&v.ident.to_string());
                acc.push('|');
                acc
            });
        vec!["Option|", "std|option|Option|", "core|option|Option|"]
            .into_iter()
            .find(|s| &idents_of_path == *s)
            .and_then(|_| path.segments.last())
    }

    extract_type_path(ty)
        .and_then(|path| extract_option_segment(path))
        .and_then(|path_seg| {
            let type_params = &path_seg.arguments;
            // It should have only on angle-bracketed param ("<String>"):
            match *type_params {
                PathArguments::AngleBracketed(ref params) => params.args.first(),
                _ => None,
            }
        })
        .and_then(|generic_arg| match *generic_arg {
            GenericArgument::Type(ref ty) => Some(ty),
            _ => None,
        })
}

fn main() {
    let cc_path = format!("{}/cc", env::var("CARGO_MANIFEST_DIR").unwrap());
    // Append `:` and add to the include_path if needed
    let include_path = format!("{}/include", cc_path.as_str());
    let src_path = format!("{}/src", cc_path.as_str());
    println!("cargo::rerun-if-changed={}", cc_path.as_str());

    // Bindgen
    {
        let mut builder = bindgen::Builder::default();

        // include path could be `:`-seperated
        for x in include_path.split(":") {
            builder = builder.clang_arg(format!("{}{}", "-I", x).as_str());
        }

        // Allow for all the basic types
        for basic_type in BASE_TYPES.iter() {
            builder = builder.allowlist_item(basic_type);
        }

        // Allow for OpenGL functions and defines
        builder = builder
            .allowlist_item("glad_gl.*")
            .allowlist_item("gladLoadGLLoader")
            .allowlist_item("GL_.*");

        let bindings = builder
            .header_contents(
                "bindgen_includer.h",
                "#define BINDGEN\n#include <glad/glad.h>\n",
            )
            .generate()
            .expect("Unable to generate bindings");
        let mut generated_code = bindings.to_string();

        // include a copy of the bindings as string constant
        // let mut debug_string = String::new();
        // debug_string.push_str("pub const BINDINGS : &str = r##\"");
        // debug_string.push_str(generated_code.as_str());
        // debug_string.push_str("\"##;");
        // generated_code.push_str(debug_string.as_str());

        // Generate helper struct type GL that contains all OpenGL functions,
        // with a constructor called unwrap() that unwraps and initializes all fields
        let bindings_syn = parse_file(generated_code.as_str()).unwrap();
        let mut gl_struct = String::from("pub struct GL {\n");
        let mut gl_unwrap =
            String::from("impl GL { pub fn unwrap() -> Self { unsafe { return GL {\n");
        let mut type_aliases: BTreeMap<String, Type> = BTreeMap::new();
        for item in bindings_syn.items.iter() {
            // Assert we are not seeing something we don't expect
            match item {
                // Skip OpenGL Defines
                Item::Const(_) => continue,
                // Handle OpenGL functions after this match
                Item::ForeignMod(_) => (),
                // Check typedefs, register OpenGL function types
                Item::Type(x) => {
                    let type_name = x.ident.to_string();
                    let type_name = type_name.as_str();

                    // All GLAD OpenGL functions types start with PFN
                    let pfncheck = Regex::new("^PFN.*").unwrap();

                    if BASE_TYPES.contains(&type_name) {
                        continue;
                    } else if pfncheck.is_match(type_name) {
                        type_aliases.insert(String::from(type_name), *x.clone().ty);
                        println!("{:#?}", type_aliases[type_name]);
                    } else {
                        // At this point everything should be handled- if not,
                        eprintln!("UNIMPLEMENTED {}", type_name);
                        todo!("Unimplemented type! Was GLAD updated?");
                    }
                    continue;
                }
                // TODO
                // Some OpenGL types are structs. Such as:
                //      GLsync
                //      Arguments to glCreateSyncFromCLeventARB (_cl_context and _cl_event)
                Item::Struct(_) => {
                    continue;
                }
                _ => todo!("Unimplemented item! GLAD Update?"),
            }

            // TODO simplify control flow
            let Item::ForeignMod(s) = item else {
                unreachable!()
            };

            // This depends on a quirk of bindgen where all generated extern blocks only contain one item
            // This can totally break in the future!
            assert!(
                    s.items.len() == 1,
                    "Expected lone function pointer static inside foreign block! This may be due to bindgen behavior having changed."
                );
            let item = s.items.last().unwrap();
            match item {
                ForeignItem::Fn(foreign_item_fn) => {
                    // Only one actual function should be present (the loader)
                    // OpenGL stuff are always global static function pointers
                    // No need for special handling, bindgen already dealt with it
                    assert!(
                        foreign_item_fn.sig.ident.to_string().as_str() == "gladLoadGLLoader",
                        "Unexpected C Function!"
                    )
                }
                ForeignItem::Static(pfn) => {
                    let symbol_name = pfn.ident.to_string();
                    let symbol_name = symbol_name.as_str();
                    // Assert it is a glad function pointer
                    let namecheck = Regex::new("^glad_gl.*").unwrap();
                    assert!(namecheck.is_match(symbol_name), "Unexpected Static!");
                    // get rid of the prefix
                    let symbol_name = &symbol_name[("glad_gl".len())..];
                    let symbol_type = extract_type_from_option(
                        &type_aliases[pfn.ty.to_token_stream().to_string().as_str()],
                    )
                    .unwrap();

                    gl_struct.push_str(
                        format!("pub {}: {},\n", symbol_name, symbol_type.to_token_stream())
                            .as_str(),
                    );
                    gl_unwrap.push_str(
                        format!(
                            "{}: {}.unwrap(),\n",
                            symbol_name,
                            pfn.ident.to_token_stream()
                        )
                        .as_str(),
                    );
                }
                _ => todo!("Unexpected item inside foreign block!"),
            }
        }
        gl_unwrap.push_str("};}}}\n");
        gl_struct.push_str("}\n");
        generated_code.push_str(&gl_struct);
        generated_code.push_str(&gl_unwrap);

        // Write File
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        let out_file = out_path.join("bindings.rs");
        let mut out_file = File::create(out_file).unwrap();
        out_file.write(generated_code.as_bytes()).unwrap();
    }

    // GLAD Loader Static Library
    {
        let src_path = src_path.as_str();
        let mut builder = cc::Build::new();
        builder.file(format!("{}/glad.c", src_path));
        for x in include_path.split(":") {
            builder.include(format!("{}", x).as_str());
        }
        builder.compile("glad");
    }
}
