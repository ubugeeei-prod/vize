//! JSON export of the levels executed by one product compilation.

#![expect(clippy::disallowed_macros, reason = "CLI paths and errors use format")]

mod page_files;

use std::path::Path;

use page_files::write_pages;

use serde_json::{Value, json};
use vize_atelier_core::options::{
    CodegenExperimentalOptions, CodegenOptions, CustomElementMatcher, TemplateSyntaxMode,
};
use vize_atelier_dom::{
    DomCompilerOptions,
    compile_template_with_custom_elements_template_syntax_codegen_and_experimental_options_with_stage_capture,
};
use vize_atelier_sfc::{
    SfcCompileExperimentalOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    compile_sfc_for_adapter_with_stage_capture, parse_sfc,
};
use vize_curator::inspector::{ProductCaptureSource, product_capture_value};
use vize_l0::{
    Allocator,
    dump::capture::{CaptureOutcome, StageCapture},
};
use vize_l1::container::{ContainerFormat, Vue};
use vize_l1_to_l2::lower::pug::derive_template_source;

use super::{DumpArgs, Pipeline};

pub(super) fn run(args: &DumpArgs) {
    let Some(path) = args.source.as_deref() else {
        eprintln!("dump: source argument required");
        std::process::exit(2);
    };
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => fail(path, &format!("{error}")),
    };
    let path_text = path.to_string_lossy();
    let pipeline = args.pipeline.unwrap_or_default();
    let extension = path.extension().and_then(|extension| extension.to_str());
    let (capture, source_meta) = match extension {
        Some("vue") => compile_sfc(path, path_text.as_ref(), &source, pipeline),
        Some("html" | "htm") => {
            if pipeline != Pipeline::Dom {
                fail(
                    path,
                    "raw template capture currently supports only the DOM backend",
                );
            }
            (
                compile_raw(&source),
                ProductCaptureSource {
                    path: Some(path_text.as_ref()),
                    container: "raw-template",
                    authored_syntax: "vue-template",
                    compiled_syntax: "vue-template",
                    template_span: None,
                },
            )
        }
        Some("pug" | "jade") => {
            if pipeline != Pipeline::Dom {
                fail(
                    path,
                    "raw Pug capture currently supports only the DOM backend",
                );
            }
            let derived = derive_template_source(&source);
            if let Some(diagnostic) = derived.first_error() {
                fail(path, diagnostic.message.as_str());
            }
            (
                compile_raw(derived.html.as_str()),
                ProductCaptureSource {
                    path: Some(path_text.as_ref()),
                    container: "raw-template",
                    authored_syntax: "pug",
                    compiled_syntax: "vue-template",
                    template_span: None,
                },
            )
        }
        _ => fail(
            path,
            "expected a .vue, .html, .htm, .pug or .jade source file",
        ),
    };
    let value = product_capture_value("vize-dump", source_meta, &capture);
    if let Some(dir) = args.dump_dir.as_deref() {
        write_pages(dir, &capture, &value, args.dump_after_change, path);
    }
    if let Some(path_out) = args.timing_json.as_deref() {
        write_json(
            path_out,
            &json!({
                "schema_version": 2,
                "target": required_field(&value, "target", path),
                "outcome": required_field(&value, "outcome", path),
                "observed": required_field(required_field(&value, "observed", path), "timings", path),
                "timings": required_field(&value, "timings", path),
            }),
            path,
        );
    }
    if let Some(path_out) = args.remarks.as_deref() {
        write_json(
            path_out,
            &json!({
                "schema_version": 2,
                "target": required_field(&value, "target", path),
                "outcome": required_field(&value, "outcome", path),
                "observed": required_field(required_field(&value, "observed", path), "remarks", path),
                "remarks": required_field(&value, "remarks", path),
            }),
            path,
        );
    }
    println!("{}", value);
    if matches!(capture.outcome, CaptureOutcome::Rejected(_)) {
        std::process::exit(1);
    }
}

fn compile_raw(source: &str) -> StageCapture {
    let allocator = Allocator::default();
    let mut capture = StageCapture::new("dom");
    let _ = compile_template_with_custom_elements_template_syntax_codegen_and_experimental_options_with_stage_capture(
        &allocator,
        source,
        DomCompilerOptions::default(),
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        CodegenExperimentalOptions::default(),
        &mut capture,
    );
    capture
}

fn compile_sfc<'a>(
    path: &Path,
    path_text: &'a str,
    source: &str,
    pipeline: Pipeline,
) -> (StageCapture, ProductCaptureSource<'a>) {
    let allocator = Allocator::default();
    let native = Vue.split(&allocator, source);
    if let Some(error) = native.errors.first() {
        fail(
            path,
            &format!(
                "native SFC block boundary is uncertain: {:?} at byte {}",
                error.code, error.offset
            ),
        );
    }
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: path_text.into(),
            ..SfcParseOptions::default()
        },
    )
    .unwrap_or_else(|error| fail(path, error.message.as_str()));
    if descriptor
        .template
        .as_ref()
        .is_some_and(|template| template.src.is_some())
    {
        fail(
            path,
            "external <template src> is not supported by native dump",
        );
    }
    if descriptor
        .script
        .as_ref()
        .is_some_and(|script| script.src.is_some())
        || descriptor
            .script_setup
            .as_ref()
            .is_some_and(|script| script.src.is_some())
        || descriptor.styles.iter().any(|style| style.src.is_some())
    {
        fail(
            path,
            "external SFC block src is not supported by native dump",
        );
    }
    let native_template = native
        .blocks
        .iter()
        .find(|block| block.name.eq_ignore_ascii_case("template"));
    let template_span = match (native_template, descriptor.template.as_ref()) {
        (None, None) => None,
        (Some(native), Some(product))
            if native.content.start as usize == product.loc.start
                && native.content.end as usize == product.loc.end
                && native.content.slice(source) == product.content.as_ref()
                && native.attr("lang").and_then(|attr| attr.value) == product.lang.as_deref() =>
        {
            Some(native.content)
        }
        _ => fail(path, "native and product SFC template boundaries differ"),
    };
    let authored_syntax = if native_template
        .and_then(|block| block.attr("lang"))
        .and_then(|attr| attr.value)
        .is_some_and(|lang| lang.eq_ignore_ascii_case("pug"))
    {
        "pug"
    } else {
        "vue-template"
    };
    let mut options = SfcCompileOptions::default();
    options.parse.filename = path_text.into();
    options.template.ssr = pipeline == Pipeline::Ssr;
    options.vapor = pipeline == Pipeline::Vapor;
    let (_, capture) = compile_sfc_for_adapter_with_stage_capture(
        &descriptor,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
        SfcCompileExperimentalOptions::default(),
    )
    .unwrap_or_else(|error| fail(path, error.message.as_str()));
    (
        capture,
        ProductCaptureSource {
            path: Some(path_text),
            container: "vue-sfc",
            authored_syntax,
            compiled_syntax: "vue-template",
            template_span,
        },
    )
}

fn write_json(path: &Path, value: &Value, source: &Path) {
    let mut text = serde_json::to_string(value)
        .unwrap_or_else(|error| fail(source, &format!("cannot serialize capture feed: {error}")));
    text.push('\n');
    std::fs::write(path, text)
        .unwrap_or_else(|error| fail(source, &format!("cannot write {}: {error}", path.display())));
}

fn required_field<'a>(value: &'a Value, key: &str, source: &Path) -> &'a Value {
    value
        .get(key)
        .unwrap_or_else(|| fail(source, &format!("capture feed missing `{key}`")))
}

fn fail(path: &Path, message: &str) -> ! {
    eprintln!("dump: {}: {message}", path.display());
    std::process::exit(1);
}
