//! Compiling lowered JSX/TSX into Vue Vapor output.
//!
//! Like the VDOM backend ([`crate::dom`]), this reuses existing Vize
//! infrastructure: the shared lowering produces a
//! [`RootNode`](vize_relief::RootNode), which is run through
//! `vize_atelier_core`'s transform (in `vapor: true` mode) and then
//! `vize_atelier_vapor`'s IR lowering + code generation — the same paths the
//! Vue template Vapor compiler uses.
//!
//! JSX render code runs inside the authoring component function's closure, so
//! the Vapor generator is invoked in **closure mode**
//! ([`VaporGenerateOptions::jsx_closure`]): free identifiers stay bare instead
//! of being `_ctx.`-prefixed, matching `vue-jsx-vapor`.

use vize_atelier_core::lane::transform;
use vize_atelier_core::options::TransformOptions;
use vize_atelier_ssr::{SsrCodegenContext, SsrCompilerOptions};
use vize_atelier_vapor::{VaporGenerateOptions, generate_vapor_with_options, transform_to_ir};
use vize_carton::{Bump, String};
use vize_croquis::Croquis;

use crate::diagnostics::JsxDiagnostic;
use crate::scoped::{ScopedStyle, build_scoped_style};
use crate::{JsxLang, JsxOutputMode, LoweredRoot, lower_source};

/// Options controlling JSX/TSX -> Vapor compilation.
#[derive(Debug, Clone, Default)]
pub struct VaporCompileOptions {
    /// Compile in SSR mode. When set, the component is server-rendered: instead
    /// of the client Vapor IR lowering, the lowered template is run through
    /// `vize_atelier_ssr`'s `ssrRender` codegen and [`VaporComponent::code`]
    /// holds an HTML-string render function (first cut, #1533 — see
    /// [`compile_root_to_vapor_ssr`]).
    pub ssr: bool,
}

/// One compiled Vapor component.
pub struct VaporComponent {
    /// Enclosing component-function name, if resolved.
    pub component_name: Option<String>,
    /// Resolved output mode (defaults to [`JsxOutputMode::Vapor`] here).
    pub mode: JsxOutputMode,
    /// Generated Vapor render code (imports + templates + render function).
    pub code: String,
    /// Static template strings referenced by the render code.
    pub templates: Vec<String>,
    /// Extracted `<style scoped>` block (#1495): the generated scope id and the
    /// scoped-rewritten CSS. `None` when the component had no `<style scoped>`.
    /// A bundler emits this CSS to a stylesheet (deferred, #1533); the scope id
    /// is already injected into the generated templates.
    pub scoped_style: Option<ScopedStyle>,
}

/// Result of compiling a JSX/TSX module to Vapor.
pub struct VaporOutput {
    /// One entry per outermost JSX render root, in source order.
    pub components: Vec<VaporComponent>,
    /// Parse and lowering diagnostics.
    pub diagnostics: Vec<JsxDiagnostic>,
}

impl VaporOutput {
    /// Whether any error-severity diagnostic was produced.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(JsxDiagnostic::is_error)
    }
}

/// Compile a JSX/TSX module into Vue Vapor render code.
pub fn compile_to_vapor(
    bump: &Bump,
    source: &str,
    lang: JsxLang,
    options: VaporCompileOptions,
) -> VaporOutput {
    let lowered = lower_source(bump, source, lang);
    let diagnostics = lowered.diagnostics;

    // Move the analysis into the arena so the transform can borrow it.
    let analysis: &Croquis = &*bump.alloc(lowered.analysis);

    let mut components = Vec::with_capacity(lowered.roots.len());
    for lowered_root in lowered.roots {
        components.push(compile_root_to_vapor(
            bump,
            lowered_root,
            analysis,
            &options,
        ));
    }

    VaporOutput {
        components,
        diagnostics,
    }
}

/// Compile a single already-lowered root to a [`VaporComponent`]. Shared by
/// [`compile_to_vapor`] and the mode-aware dispatcher in [`crate::compile`].
pub(crate) fn compile_root_to_vapor(
    bump: &Bump,
    lowered: LoweredRoot,
    analysis: &Croquis,
    options: &VaporCompileOptions,
) -> VaporComponent {
    let LoweredRoot {
        mut root,
        mode,
        component_name,
        scoped_css,
        // The style interpolation spans are consumed by the type checker
        // (`vize_canon`), not the Vapor scoping backend.
        scoped_style_exprs: _,
    } = lowered;

    // Extract + rewrite the `<style scoped>` CSS and derive the scope id, reusing
    // the SFC scope infrastructure. Unlike VDOM (where the codegen injects the
    // scope attribute via `CodegenOptions.scope_id`), the Vapor generator emits
    // static `_template("…")` strings, so — mirroring the SFC Vapor path — the
    // `data-v-<hash>` attribute is injected into those strings post-generation.
    let scoped_style =
        scoped_css.map(|css| build_scoped_style(component_name.as_deref(), css.as_str()));

    // SSR (#1533, first cut): when SSR is requested, the lowered root is the same
    // `vize_relief` `RootNode` the SSR crate consumes, so we run the SSR-flavored
    // core transform and `vize_atelier_ssr`'s `ssrRender` codegen instead of the
    // client Vapor IR lowering. This produces an HTML-string render function
    // (`_push(`…`)`) covering static elements, static/dynamic attributes, and
    // text interpolation. See `compile_root_to_vapor_ssr` for the deferred scope.
    if options.ssr {
        return compile_root_to_vapor_ssr(bump, root, analysis, mode, component_name, scoped_style);
    }

    let transform_opts = TransformOptions {
        // JSX render fns close over the setup scope; don't prefix `_ctx.`.
        prefix_identifiers: false,
        ssr: options.ssr,
        // Vapor IR lowering requires the core transform to run in vapor mode
        // (e.g. it skips v-model desugaring, handled by the Vapor backend).
        vapor: true,
        binding_metadata: None,
        ..Default::default()
    };
    transform(bump, &mut root, transform_opts, Some(analysis));

    let ir = transform_to_ir(bump, &root);
    let generated =
        generate_vapor_with_options(&ir, None, VaporGenerateOptions { jsx_closure: true });

    let (code, templates) = if let Some(style) = scoped_style.as_ref() {
        inject_scope_id(&generated.code, &generated.templates, &style.scope_id)
    } else {
        (generated.code, generated.templates)
    };

    VaporComponent {
        component_name,
        mode: mode.unwrap_or(JsxOutputMode::Vapor),
        code,
        templates,
        scoped_style,
    }
}

/// First-cut JSX/TSX -> Vapor **SSR** render codegen (#1533).
///
/// Reuses [`vize_atelier_ssr`] end to end: the lowered [`RootNode`] is run
/// through the SSR-flavored core transform (the same options
/// `vize_atelier_ssr::compile_ssr` uses) and then `SsrCodegenContext`, which
/// emits an `ssrRender(_ctx, _push, _parent, _attrs)` function that server-
/// renders the component to an HTML string via `_push(`…`)`.
///
/// Identifiers stay **bare** (no `_ctx.` prefix) to match the JSX closure model
/// the client Vapor path documents: a JSX component is a plain function whose
/// render — client or server — closes over the setup scope. The generated
/// `code` is the SSR preamble (`@vue/server-renderer` / `vue` imports) followed
/// by the render function, mirroring the shape of the client Vapor `code`.
///
/// Implemented subset: static elements, static + dynamic (`v-bind`) attributes,
/// dynamic class/style, and text interpolation. Control flow (`v-if`/`v-for`),
/// slots, nested components, and `renderToString` runtime wiring flow through
/// the shared SSR codegen too but are **not** part of this first cut's tested
/// contract — see the PR's "Deferred" section.
fn compile_root_to_vapor_ssr<'a>(
    bump: &'a Bump,
    mut root: vize_atelier_core::RootNode<'a>,
    analysis: &'a Croquis,
    mode: Option<JsxOutputMode>,
    component_name: Option<String>,
    scoped_style: Option<ScopedStyle>,
) -> VaporComponent {
    let transform_opts = TransformOptions {
        // JSX render fns close over the setup scope; keep identifiers bare so the
        // SSR render function references the closure's bindings directly (no
        // `_ctx.` prefix), consistent with the client Vapor path above.
        prefix_identifiers: false,
        // SSR transforms disable DOM-only optimizations (hoisting, handler
        // caching) and desugar directives for string output instead of vnodes.
        hoist_static: false,
        cache_handlers: false,
        ssr: true,
        binding_metadata: None,
        ..Default::default()
    };
    transform(bump, &mut root, transform_opts, Some(analysis));

    // The scope id is already embedded in the SSR options so `ssrRender` emits
    // the `data-v-<hash>` attribute inline, matching the SFC SSR scope path
    // (no post-generation string injection needed, unlike the client templates).
    let ssr_options = SsrCompilerOptions {
        component_name: component_name.clone(),
        scope_id: scoped_style.as_ref().map(|style| style.scope_id.clone()),
        ..SsrCompilerOptions::default()
    };
    let generated = SsrCodegenContext::new(bump, &ssr_options).generate(&root);

    // Combine preamble (imports) + render function into a single module, mirroring
    // the client Vapor `code` field which also inlines its imports.
    let mut code = generated.preamble;
    if !code.is_empty() && !generated.code.is_empty() {
        code.push('\n');
    }
    code.push_str(&generated.code);

    VaporComponent {
        component_name,
        mode: mode.unwrap_or(JsxOutputMode::Vapor),
        code,
        // SSR output is a single HTML-string render function; the client-only
        // static `_template("…")` strings do not apply.
        templates: Vec::new(),
        scoped_style,
    }
}

/// Inject the `data-v-<hash>` scope attribute into every Vapor `_template("…")`
/// declaration's first element, mirroring the SFC Vapor scope path
/// (`vize_atelier_sfc::compile_template::vapor::add_scope_id_to_template`).
///
/// Both the generated `code` (which inlines the `_template(...)` declarations)
/// and the separately-collected `templates` are rewritten so the two stay in
/// sync.
fn inject_scope_id(code: &str, templates: &[String], scope_id: &str) -> (String, Vec<String>) {
    let mut out_code = String::default();
    for (index, line) in code.lines().enumerate() {
        if index > 0 {
            out_code.push('\n');
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with("const t") && trimmed.contains("_template(") {
            out_code.push_str(&add_scope_id_to_template_line(line, scope_id));
        } else {
            out_code.push_str(line);
        }
    }
    if code.ends_with('\n') {
        out_code.push('\n');
    }

    let out_templates = templates
        .iter()
        .map(|template| add_scope_id_to_template_html(template, scope_id))
        .collect();

    (out_code, out_templates)
}

/// Inject the scope attribute into a `const tN = _template("<tag…>…")` line.
fn add_scope_id_to_template_line(line: &str, scope_id: &str) -> String {
    let Some(start) = line.find("\"<") else {
        return String::from(line);
    };
    let Some(end_rel) = line[start..].find(">\"") else {
        return String::from(line);
    };
    let end = start + end_rel;

    let prefix = &line[..start + 2]; // up to and including the opening `<`
    let content = &line[start + 2..end + 1]; // element content (no closing quote)
    let suffix = &line[end + 1..]; // closing quote + remainder

    let Some(tag_end) = content.find(|c: char| c.is_whitespace() || c == '>') else {
        return String::from(line);
    };
    let tag_name = &content[..tag_end];
    let rest = &content[tag_end..];

    let mut result = String::default();
    result.push_str(prefix);
    result.push_str(tag_name);
    result.push(' ');
    result.push_str(scope_id);
    result.push_str(rest);
    result.push_str(suffix);
    result
}

/// Inject the scope attribute into a raw template HTML string (no quoting), used
/// for the `templates` vector.
fn add_scope_id_to_template_html(template: &str, scope_id: &str) -> String {
    let Some(open) = template.find('<') else {
        return String::from(template);
    };
    let after_open = open + 1;
    let Some(tag_end_rel) = template[after_open..].find(|c: char| c.is_whitespace() || c == '>')
    else {
        return String::from(template);
    };
    let tag_end = after_open + tag_end_rel;

    let mut result = String::default();
    result.push_str(&template[..tag_end]);
    result.push(' ');
    result.push_str(scope_id);
    result.push_str(&template[tag_end..]);
    result
}
