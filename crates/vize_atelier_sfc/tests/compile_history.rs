//! Complete current public API oracles for historical compiler fixes (#6880).
//! Each case compiles one authored input through the public SFC or DOM
//! template entry and snapshots every result field. These snapshots are legacy
//! regression evidence; they confer no native credit.

#![expect(
    clippy::panic,
    clippy::disallowed_macros,
    reason = "tests panic on invalid fixtures and snapshot the complete actual Debug bytes"
)]

use serde::Deserialize;
use vize_atelier_core::{CodegenMode, CompilerError, TemplateSyntaxMode};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_template_syntax};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcCompileResult, SfcError, SfcParseOptions,
    StyleCompileOptions, TemplateCompileOptions, compile_sfc_with_template_syntax, parse_sfc,
};
use vize_l0::{Allocator, String, config::VueVersion};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    history: String,
    source: String,
    filename: String,
    /// `sfc` compiles a whole `.vue` file; `template` compiles bare markup.
    entry: String,
    /// `dom`, `ssr` or `vapor`; a `template` entry supports `dom` only.
    target: String,
    #[serde(default)]
    options: Options,
    errors: usize,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    /// SFC: production output.
    prod: Option<bool>,
    /// SFC: keep TypeScript in the emitted script.
    preserve_ts: Option<bool>,
    /// SFC: rewrite the module to the standalone global-runtime output
    /// (`ScriptCompileOptions::inline_template`). The render function is
    /// inlined into setup either way.
    standalone: Option<bool>,
    /// Template: `module` (default) or `function`.
    mode: Option<String>,
    /// Template: prefix identifiers with `_ctx.`.
    prefix_identifiers: Option<bool>,
    /// Template: hoist static nodes.
    hoist_static: Option<bool>,
    /// Template: cache inline event handlers.
    cache_handlers: Option<bool>,
    /// Template: keep HTML comments.
    comments: Option<bool>,
    /// Emit a source map.
    source_map: Option<bool>,
    /// Template syntax recovery: `standard` (default), `strict` or `quirks`.
    template_syntax: Option<String>,
    /// Compile for a custom renderer instead of the DOM runtime.
    custom_renderer: Option<bool>,
    /// SFC: Vue dialect, `3` (default), `2.7` or `2`.
    vue_version: Option<String>,
}

fn template_syntax(case: &Case) -> TemplateSyntaxMode {
    match case
        .options
        .template_syntax
        .as_deref()
        .unwrap_or("standard")
    {
        "standard" => TemplateSyntaxMode::Standard,
        "strict" => TemplateSyntaxMode::Strict,
        "quirks" => TemplateSyntaxMode::Quirks,
        mode => panic!("unknown fixture template syntax: {mode}"),
    }
}

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "every observation field is retained in the complete Debug oracle"
)]
enum Observation {
    Sfc(Result<SfcCompileResult, SfcError>),
    ParseError(SfcError),
    Template {
        errors: Vec<CompilerError>,
        preamble: String,
        code: String,
        map: Option<String>,
    },
}

impl Observation {
    fn errors(&self) -> usize {
        match self {
            Self::Sfc(Ok(result)) => result.errors.len(),
            Self::Sfc(Err(_)) | Self::ParseError(_) => 1,
            Self::Template { errors, .. } => errors.len(),
        }
    }
}

fn compile_whole_sfc(case: &Case) -> Observation {
    let source_map = case.options.source_map.unwrap_or(false);
    let parse = SfcParseOptions {
        filename: case.filename.as_str().into(),
        source_map,
        ..SfcParseOptions::default()
    };
    let descriptor = match parse_sfc(&case.source, parse.clone()) {
        Ok(descriptor) => descriptor,
        Err(error) => return Observation::ParseError(error),
    };
    let (ssr, vapor) = match case.target.as_str() {
        "dom" => (false, false),
        "ssr" => (true, false),
        "vapor" => (false, true),
        target => panic!("unknown fixture target: {target}"),
    };
    let scoped = descriptor.styles.iter().any(|style| style.scoped);
    let is_ts = case.options.preserve_ts.unwrap_or(false);
    let is_prod = case.options.prod.unwrap_or(false);
    let options = SfcCompileOptions {
        parse,
        script: ScriptCompileOptions {
            id: Some(case.filename.as_str().into()),
            inline_template: case.options.standalone.unwrap_or(false),
            is_ts,
            ..ScriptCompileOptions::default()
        },
        template: TemplateCompileOptions {
            id: Some(case.filename.as_str().into()),
            scoped,
            ssr,
            vapor,
            is_prod,
            is_ts,
            custom_renderer: case.options.custom_renderer.unwrap_or(false),
            dialect: match case.options.vue_version.as_deref().unwrap_or("3") {
                "3" => VueVersion::V3,
                "2.7" => VueVersion::V2_7,
                "2" => VueVersion::V2,
                version => panic!("unknown fixture Vue version: {version}"),
            },
            ..TemplateCompileOptions::default()
        },
        style: StyleCompileOptions {
            id: case.filename.as_str().into(),
            scoped,
            source_map,
            ..StyleCompileOptions::default()
        },
        vapor,
        scope_id: None,
    };
    Observation::Sfc(compile_sfc_with_template_syntax(
        &descriptor,
        options,
        template_syntax(case),
    ))
}

fn compile_bare_template(case: &Case) -> Observation {
    assert_eq!(
        case.target, "dom",
        "{}: template entries compile for DOM",
        case.id
    );
    let options = &case.options;
    let mode = match options.mode.as_deref().unwrap_or("module") {
        "module" => CodegenMode::Module,
        "function" => CodegenMode::Function,
        mode => panic!("unknown fixture codegen mode: {mode}"),
    };
    let allocator = Allocator::new();
    let (_, errors, codegen) = compile_template_with_template_syntax(
        &allocator,
        &case.source,
        DomCompilerOptions {
            mode,
            prefix_identifiers: options.prefix_identifiers.unwrap_or(false),
            hoist_static: options.hoist_static.unwrap_or(false),
            cache_handlers: options.cache_handlers.unwrap_or(false),
            comments: options.comments.unwrap_or(false),
            source_map: options.source_map.unwrap_or(false),
            custom_renderer: options.custom_renderer.unwrap_or(false),
            ..DomCompilerOptions::default()
        },
        template_syntax(case),
    );
    Observation::Template {
        errors,
        preamble: codegen.preamble,
        code: codegen.code,
        map: codegen.map,
    }
}

fn observe(case: &Case) -> Observation {
    match case.entry.as_str() {
        "sfc" => compile_whole_sfc(case),
        "template" => compile_bare_template(case),
        entry => panic!("unknown fixture entry: {entry}"),
    }
}

#[test]
fn full_current_compile_results() {
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("fixtures/compile-history/cases.json"))
            .expect("strict fixture schema");
    assert_eq!(
        cases.len(),
        1092,
        "every reviewed case must remain registered"
    );
    // Every case runs before the error counts are judged, so one run reports
    // all mismatches as an `id -> observed` JSON object.
    let mut mismatched = serde_json::Map::new();
    for case in cases {
        assert_eq!(case.history.len(), 40, "full historical commit identity");
        // Snapshot all public fields via Debug; no filtering or normalization.
        let first = observe(&case);
        let first_bytes = format!("{case:#?}\n{first:#?}");
        let repeat = observe(&case);
        assert_eq!(
            first_bytes,
            format!("{case:#?}\n{repeat:#?}"),
            "{}",
            case.id
        );
        if first.errors() != case.errors {
            mismatched.insert(case.id.as_str().into(), first.errors().into());
        }
        insta::assert_snapshot!(case.id.as_str(), first_bytes);
    }
    assert!(
        mismatched.is_empty(),
        "observed error counts differ: {}",
        serde_json::Value::Object(mismatched)
    );
}
