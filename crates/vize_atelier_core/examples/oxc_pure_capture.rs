//! Bounded actual original/candidate parser, OXC printer and legacy compiler capture.
use std::{fs, path::PathBuf};

use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde_json::json;
use vize_atelier_core::{
    codegen::generate,
    lane::transform,
    options::{CodegenMode, CodegenOptions, ParserOptions, TransformOptions},
    parser::parse_with_options,
};
use vize_l0::{Allocator, cstr};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("capture directory is required")?;
    fs::create_dir_all(&directory)?;
    let profiles = [
        SourceType::mjs(),
        SourceType::mjs().with_script(true),
        SourceType::jsx(),
        SourceType::jsx().with_script(true),
        SourceType::ts(),
        SourceType::ts().with_script(true),
        SourceType::tsx(),
        SourceType::tsx().with_script(true),
    ];
    let mut observations = Vec::new();
    for source in [
        "f<>/((\nd=//#__PURE__0",
        "/*#__PURE__*/:",
        "(/*#__PURE__*/\n/* other */\nf",
        "(/*#__PURE__*/ /* ordinary */ value)",
        "(/*@__PURE__*/ invoke(/* ordinary */))",
        "export const X = /* @__PURE__ */ foo(/* comment */);",
    ] {
        for source_type in profiles {
            let allocator = Allocator::default();
            let parsed = Parser::new(allocator.as_oxc(), source, source_type).parse();
            let printed = Codegen::new().build(&parsed.program).code;
            observations.push(json!({
                "source": source, "profile": cstr!("{source_type:?}"),
                "ast": cstr!("{:?}", parsed.program),
                "diagnostics": cstr!("{:?}", parsed.diagnostics),
                "panicked": parsed.panicked, "flow": parsed.is_flow_language,
                "oxc_code": printed,
            }));
            drop(parsed);
        }
    }
    fs::write(
        directory.join("parser.json"),
        serde_json::to_vec_pretty(&observations)?,
    )?;
    let mut compiler = Vec::new();
    for input in [
        "<div>{{ (/*#__PURE__*/ /* ordinary */ value) }}</div>",
        "<div>{{ (/*@__PURE__*/ invoke(/* ordinary */)) }}</div>",
    ] {
        let allocator = Allocator::default();
        let (mut root, errors) = parse_with_options(
            &allocator,
            input,
            ParserOptions {
                is_native_tag: Some(vize_l0::is_native_tag),
                ..Default::default()
            },
        );
        let transform_errors = transform(
            &allocator,
            &mut root,
            TransformOptions {
                prefix_identifiers: true,
                ..Default::default()
            },
            None,
        );
        let generated = generate(
            &root,
            CodegenOptions {
                mode: CodegenMode::Module,
                ..Default::default()
            },
        );
        let preamble = generated.preamble.trim();
        let code = if preamble.is_empty() {
            generated.code.clone()
        } else {
            cstr!("{preamble}\n\n{}", generated.code)
        };
        compiler.push(json!({
            "source": input,
            "parse_errors": cstr!("{errors:?}"),
            "transform_errors": cstr!("{transform_errors:?}"),
            "code": code,
        }));
    }
    fs::write(
        directory.join("compiler.json"),
        serde_json::to_vec_pretty(&compiler)?,
    )?;
    Ok(())
}
