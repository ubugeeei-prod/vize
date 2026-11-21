//! Complete current public API oracles for historical compiler fixes (#6880).
//! Each case compiles one authored input through the public SFC or DOM
//! template entry and snapshots every result field. These snapshots are legacy
//! regression evidence; they confer no native credit.

#![expect(
    clippy::panic,
    clippy::disallowed_macros,
    reason = "tests panic on invalid fixtures and snapshot the complete actual Debug bytes"
)]

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use vize_atelier_core::options::CustomElementMatcher;
use vize_atelier_core::{CodegenMode, CodegenOptions, CompilerError, TemplateSyntaxMode};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_template_syntax};
use vize_atelier_sfc::vite_plugin::{
    resolve_css_imports, rewrite_dynamic_template_imports, rewrite_import_meta_glob_base,
    rewrite_static_asset_urls, scope_css_for_pipeline,
};
use vize_atelier_sfc::{
    CssCompileOptions, CssCompileResult, ScriptCompileOptions, SfcCompileExperimentalOptions,
    SfcCompileOptions, SfcCompileResult, SfcError, SfcParseOptions, SfcScriptOutputMode,
    StyleCompileOptions, TemplateCompileOptions, compile_css, compile_sfc_for_adapter,
    compile_sfc_for_adapter_with_project, compile_sfc_with_template_syntax, parse_css_ast,
    parse_sfc, print_css_ast, script::SfcTypeProject,
};
use vize_l0::{Allocator, String, config::VueVersion};

mod compile_history {
    pub(crate) mod project;
}
use compile_history::project::{relocate, write_project};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    history: String,
    source: String,
    filename: String,
    /// `sfc` compiles a whole `.vue` file; `template` compiles bare markup;
    /// `css` runs the standalone CSS compiler and its AST round trip;
    /// `vite-css` runs the Vite pipeline scoper with `data-v-history`.
    entry: String,
    /// `dom`, `ssr` or `vapor`; a `template` entry supports `dom` only.
    target: String,
    #[serde(default)]
    options: Options,
    /// SFC: project files written beside `filename` (relative path -> text),
    /// for fixes that resolve imported types, re-exports or packages. The
    /// project root's absolute path is replaced by `$PROJECT` in the oracle.
    #[serde(default)]
    files: BTreeMap<String, String>,
    errors: usize,
}

#[derive(Default, Deserialize)]
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
    /// SFC: `inline` (default) or `separate`, the adapter output that emits a
    /// render function beside the setup-state return object.
    script_output: Option<String>,
    /// SFC: custom-element tag globs (`ion-*`) through the adapter entry.
    custom_elements: Option<Vec<String>>,
    /// Compile `v-match`/`v-when` patterned templates.
    patterned_template: Option<bool>,
    /// Accept `//` comments between attributes inside a tag.
    in_tag_comments: Option<bool>,
    /// CSS: scope selectors with `data-v-history`.
    scoped: Option<bool>,
    /// CSS: minify the output.
    minify: Option<bool>,
    /// `vite` entry: `importMetaGlobBase`, `cssImports`, `staticAssetUrls`
    /// or `dynamicTemplateImports`, with `/project/src/App.vue` as importer.
    vite_helper: Option<String>,
    /// Multi-file cases: no project `tsconfig.json`, so the fallback resolver
    /// (`@/` src alias, `node_modules` package types) runs. Probes outside the
    /// project are machine-specific and drop out of the oracle.
    no_project_config: Option<bool>,
}

// The oracle prints every field the original cohort had; fields added for
// later cases print only when set, so earlier snapshots keep their bytes.
impl core::fmt::Debug for Case {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut out = f.debug_struct("Case");
        out.field("id", &self.id)
            .field("history", &self.history)
            .field("source", &self.source)
            .field("filename", &self.filename)
            .field("entry", &self.entry)
            .field("target", &self.target)
            .field("options", &self.options);
        if !self.files.is_empty() {
            out.field("files", &self.files);
        }
        out.field("errors", &self.errors).finish()
    }
}

impl core::fmt::Debug for Options {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut out = f.debug_struct("Options");
        out.field("prod", &self.prod)
            .field("preserve_ts", &self.preserve_ts)
            .field("standalone", &self.standalone)
            .field("mode", &self.mode)
            .field("prefix_identifiers", &self.prefix_identifiers)
            .field("hoist_static", &self.hoist_static)
            .field("cache_handlers", &self.cache_handlers)
            .field("comments", &self.comments)
            .field("source_map", &self.source_map)
            .field("template_syntax", &self.template_syntax)
            .field("custom_renderer", &self.custom_renderer)
            .field("vue_version", &self.vue_version);
        let later: [(&str, &dyn core::fmt::Debug, bool); 8] = [
            (
                "script_output",
                &self.script_output,
                self.script_output.is_some(),
            ),
            (
                "custom_elements",
                &self.custom_elements,
                self.custom_elements.is_some(),
            ),
            (
                "patterned_template",
                &self.patterned_template,
                self.patterned_template.is_some(),
            ),
            (
                "in_tag_comments",
                &self.in_tag_comments,
                self.in_tag_comments.is_some(),
            ),
            ("scoped", &self.scoped, self.scoped.is_some()),
            ("minify", &self.minify, self.minify.is_some()),
            ("vite_helper", &self.vite_helper, self.vite_helper.is_some()),
            (
                "no_project_config",
                &self.no_project_config,
                self.no_project_config.is_some(),
            ),
        ];
        for (name, value, set) in later {
            if set {
                out.field(name, value);
            }
        }
        out.finish()
    }
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
    Css {
        compiled: CssCompileResult,
        ast_errors: Vec<String>,
        printed: Option<CssCompileResult>,
    },
    PipelineCss(String),
}

impl Observation {
    fn errors(&self) -> usize {
        match self {
            Self::Sfc(Ok(result)) => result.errors.len(),
            Self::Sfc(Err(_)) | Self::ParseError(_) => 1,
            Self::Template { errors, .. } => errors.len(),
            Self::Css {
                compiled,
                ast_errors,
                printed,
            } => {
                compiled.errors.len()
                    + ast_errors.len()
                    + printed.as_ref().map_or(0, |printed| printed.errors.len())
            }
            Self::PipelineCss(_) => 0,
        }
    }
}

fn compile_whole_sfc(case: &Case, project: Option<&Path>) -> Observation {
    let source_map = case.options.source_map.unwrap_or(false);
    let filename: String = match project {
        Some(root) => root
            .join(case.filename.as_str())
            .to_string_lossy()
            .as_ref()
            .into(),
        None => case.filename.clone(),
    };
    let parse = SfcParseOptions {
        filename: filename.as_str().into(),
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
            id: Some(filename.as_str().into()),
            inline_template: case.options.standalone.unwrap_or(false),
            is_ts,
            ..ScriptCompileOptions::default()
        },
        template: TemplateCompileOptions {
            id: Some(filename.as_str().into()),
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
            compiler_options: (case.options.patterned_template.is_some()
                || case.options.in_tag_comments.is_some())
            .then(|| DomCompilerOptions {
                experimental_patterned_template: case.options.patterned_template.unwrap_or(false),
                experimental_in_tag_comments: case.options.in_tag_comments.unwrap_or(false),
                ..DomCompilerOptions::default()
            }),
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
    let script_output = match case.options.script_output.as_deref().unwrap_or("inline") {
        "inline" => SfcScriptOutputMode::InlineTemplate,
        "separate" => SfcScriptOutputMode::SeparateTemplate,
        mode => panic!("unknown fixture script output: {mode}"),
    };
    if case.options.no_project_config.unwrap_or(false) {
        let root = project.expect("no-config cases require a project");
        let no_project = SfcTypeProject::without_config(root).expect("empty project authority");
        let custom_elements = CustomElementMatcher::from_patterns(
            case.options
                .custom_elements
                .iter()
                .flatten()
                .cloned()
                .collect(),
        );
        return Observation::Sfc(compile_sfc_for_adapter_with_project(
            &descriptor,
            options,
            template_syntax(case),
            custom_elements,
            CodegenOptions::default(),
            script_output,
            SfcCompileExperimentalOptions::default(),
            false,
            Some(&no_project),
        ));
    }
    if script_output == SfcScriptOutputMode::InlineTemplate
        && case.options.custom_elements.is_none()
    {
        return Observation::Sfc(compile_sfc_with_template_syntax(
            &descriptor,
            options,
            template_syntax(case),
        ));
    }
    let custom_elements = CustomElementMatcher::from_patterns(
        case.options
            .custom_elements
            .iter()
            .flatten()
            .cloned()
            .collect(),
    );
    Observation::Sfc(compile_sfc_for_adapter(
        &descriptor,
        options,
        template_syntax(case),
        custom_elements,
        CodegenOptions::default(),
        script_output,
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
            experimental_patterned_template: options.patterned_template.unwrap_or(false),
            experimental_in_tag_comments: options.in_tag_comments.unwrap_or(false),
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

fn compile_standalone_css(case: &Case) -> Observation {
    let scoped = case.options.scoped.unwrap_or(false);
    let options = CssCompileOptions {
        scope_id: scoped.then(|| "data-v-history".into()),
        scoped,
        minify: case.options.minify.unwrap_or(false),
        source_map: case.options.source_map.unwrap_or(false),
        ..CssCompileOptions::default()
    };
    let ast = parse_css_ast(&case.source, &options);
    Observation::Css {
        compiled: compile_css(&case.source, &options),
        ast_errors: ast.errors,
        printed: ast.ast.map(|ast| print_css_ast(ast, &options)),
    }
}

/// The Vite plugin's module and CSS rewriting helpers.
fn vite_helper(case: &Case) -> std::string::String {
    let importer = "/project/src/App.vue";
    let source = case.source.as_str();
    match case.options.vite_helper.as_deref() {
        Some("importMetaGlobBase") => {
            rewrite_import_meta_glob_base(source, importer, "/project").into()
        }
        Some("cssImports") => resolve_css_imports(source, importer, &[], false, None).into(),
        Some("staticAssetUrls") => rewrite_static_asset_urls(source, &[]).into(),
        Some("dynamicTemplateImports") => rewrite_dynamic_template_imports(source, &[]).into(),
        helper => panic!("unknown fixture Vite helper: {helper:?}"),
    }
}

fn observe(case: &Case, project: Option<&Path>) -> Observation {
    match case.entry.as_str() {
        "sfc" => compile_whole_sfc(case, project),
        "template" => compile_bare_template(case),
        "css" => compile_standalone_css(case),
        "vite-css" => Observation::PipelineCss(
            scope_css_for_pipeline(&case.source, "data-v-history")
                .as_str()
                .into(),
        ),
        "vite" => Observation::PipelineCss(vite_helper(case).as_str().into()),
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
        1763,
        "every reviewed case must remain registered"
    );
    // Every case runs before the error counts are judged, so one run reports
    // all mismatches as an `id -> observed` JSON object.
    let mut mismatched = serde_json::Map::new();
    for case in cases {
        assert_eq!(case.history.len(), 40, "full historical commit identity");
        // Snapshot all public fields via Debug; the only rewrite is the
        // temporary project root of a multi-file case.
        let project = (!case.files.is_empty()).then(|| write_project(&case));
        let root = project.as_ref().map(|project| project.path());
        let first = observe(&case, root);
        let first_bytes = relocate(format!("{case:#?}\n{first:#?}"), root);
        let repeat = observe(&case, root);
        assert_eq!(
            first_bytes,
            relocate(format!("{case:#?}\n{repeat:#?}"), root),
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
