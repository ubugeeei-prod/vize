//! Generating plain `.ts` virtual TypeScript for `.jsx`/`.tsx` Vue components
//! (issue #1497, opt-in).
//!
//! This is the JSX/TSX parallel to [`super::vue_codegen`]. It is reached only
//! when the user explicitly enables `typeChecker.jsxTypecheck` (default off):
//! mixed Vue/React repositories may contain React `.tsx` files that must *not*
//! be type-checked as Vue JSX.
//!
//! # Authoring convention (#1502)
//!
//! A Vize JSX/TSX component is a plain function whose parameters carry the
//! component contract, with no macros and no runtime validation:
//!
//! ```tsx
//! const Comp = (
//!     props: { msg: string; count?: number },
//!     { emit }: Ctx<{ change: [value: number] }>,
//! ) => <div>{props.msg}</div>;
//! ```
//!
//! The **typed first parameter is the props type**; the optional typed second
//! parameter is the `Ctx<Emits, Slots>` context. Defaults are plain
//! destructuring defaults.
//!
//! # Why a textual JSX → plain-TS lowering
//!
//! `vize_canon` virtual TypeScript stays plain `.ts` (never JSX-format virtual
//! documents — standing directive). A `.tsx` Vue component is, syntactically,
//! already valid TypeScript *except* for the JSX elements themselves. So this
//! pass keeps every non-JSX byte verbatim (component functions, the typed props
//! parameter, the setup body) and replaces only the JSX render roots with a
//! synthesized plain-TS expression that re-lists every embedded JSX expression.
//!
//! The result type-checks exactly what this first cut promises:
//! - the **typed first parameter** stays verbatim, so every `props.X` access is
//!   checked against the declared props type;
//! - the **typed second parameter** (`{ emit, slots }: Ctx<Emits, Slots>`) stays
//!   verbatim, and an ambient [`Ctx<Emits, Slots>`](CTX_HELPER) type is injected
//!   so `emit(name, ...args)` checks `name` against `keyof Emits` and the payload
//!   against the tuple `Emits[name]` (Vue's emits-as-tuple convention), and
//!   `slots` is typed as `Slots`;
//! - the **setup-scope** statements above the `return <jsx/>` stay verbatim, so
//!   their declarations and uses are checked;
//! - each **JSX expression** (`{props.msg}`, `class={cls}`, `{count + 1}`, …) is
//!   re-emitted as real TypeScript at — and source-mapped back to — its original
//!   byte range, so a wrong type inside a JSX expression is reported at the right
//!   location;
//! - **directive expressions** are checked too (#1497): a `v-model` binding
//!   target is re-emitted as an assignment to itself, so binding to a `const`,
//!   a `readonly`/computed value, or a non-lvalue is reported at the binding; a
//!   `v-for` (idiomatic `items.map(…)`) body is re-emitted *inside* the `.map()`
//!   callback so the loop aliases bind with their inferred element types; and
//!   `v-show`/`v-if` conditions, directive arg/value expressions, and event
//!   handlers are re-emitted as plain reads.
//! - **style-block expressions** are checked too (#1497): a `<style scoped>` JSX
//!   block (#1495) is extracted out of the rendered children, but its
//!   template-literal interpolations (`${expr}`, e.g. `color: ${props.color}`)
//!   reference script values and are re-emitted through the same sink and
//!   component scope as that root's JSX expressions, so a wrong type inside a
//!   style interpolation is reported at the interpolation.
//!
//! Deferred (see issue #1497): CSS `v-bind(expr)` references inside a
//! `<style scoped>` block (their spans live in cooked CSS text whose offsets no
//! longer map to source bytes, so recovering them needs dedicated extraction);
//! the stateful `defineComponent(() => () => VNode)` form; and full source-map
//! fidelity for the synthesized wrapper scaffolding.

use std::path::Path;

use vize_atelier_jsx::{JsxDiagnostic, JsxLang, StyleExprSpan, lower_source};
use vize_carton::{Bump, String as CompactString, ToCompactString, cstr};
use vize_relief::{
    ExpressionNode, RootNode, TemplateChildNode,
    elements::PropNode,
    expressions::{CompoundExpressionChild, CompoundExpressionNode},
};

use crate::batch::error::CorsaResult;
use crate::batch::{Diagnostic, SfcBlockType};
use crate::virtual_ts::VizeMapping;

use super::diagnostics::diagnostic_for_offset;

/// The generated plain-`.ts` virtual file for one `.jsx`/`.tsx` source.
pub(super) struct GeneratedJsxFile {
    pub(super) code: CompactString,
    pub(super) mappings: Vec<VizeMapping>,
    pub(super) diagnostics: Vec<Diagnostic>,
}

/// Name of the synthesized helper that swallows every re-emitted JSX
/// expression. Declaring it ambient and `any`-returning lets each argument be
/// type-checked independently while the whole call stays a valid render return.
const JSX_EXPR_SINK: &str = "__vize_jsx_expr__";

/// Ambient `Ctx<Emits, Slots>` type injected at module scope so the typed
/// second parameter of a Vize JSX/TSX component (`{ emit, slots }: Ctx<…>`)
/// resolves and type-checks (#1502).
///
/// `emit` reuses the very same emits-as-tuple convention as the `.vue` path's
/// `defineEmits<E>()` (see `crate::virtual_ts::helpers`): the `__EmitFn<E>`
/// alias resolves `E = { change: [value: number] }` to a callable
/// `<K extends keyof E>(event: K, ...args: E[K]) => void`, so `emit('change', 1)`
/// checks the payload against the declared tuple and an unknown event name or a
/// wrong payload is reported at the `emit(...)` call site. `slots` is typed as
/// the second type argument so slot access/usage type-checks. Both fall back to
/// `{}` when omitted (`Ctx`, `Ctx<Emits>`). The type is purely ambient and fully
/// erased — no runtime is emitted.
///
/// Kept self-contained (the emit trio is duplicated rather than pulling in the
/// broader Vue helper blob) so JSX/TSX virtual TS never depends on resolving the
/// `vue` module, matching the minimal, fully-erased intent of this path.
const CTX_HELPER: &str = "type __EmitShape<T> = T extends (...args: any[]) => any ? T : T extends Record<string, any> ? { [K in keyof T]: T[K] extends (...args: infer A) => any ? A : T[K] extends any[] ? T[K] : any[]; } : Record<string, any[]>;\n\
type __EmitArgs<T, K extends keyof T> = T[K] extends any[] ? T[K] : any[];\n\
type __EmitFn<T> = __EmitShape<T> extends (...args: any[]) => any ? __EmitShape<T> : (<K extends keyof __EmitShape<T>>(event: K, ...args: __EmitArgs<__EmitShape<T>, K>) => void);\n\
type Ctx<Emits = {}, Slots = {}> = { emit: __EmitFn<Emits>; slots: Slots; attrs: Record<string, unknown>; };\n";

/// A dynamic JSX expression recovered from the lowered tree: its original source
/// text plus the byte range it occupied in the `.jsx`/`.tsx` source.
struct JsxExpr {
    content: CompactString,
    start: u32,
    end: u32,
}

/// One re-emitted unit recovered from a lowered JSX root, in source order.
///
/// The render pass turns these into the arguments of a `__vize_jsx_expr__(…)`
/// call. Most are plain [`Expr`](JsxEmit::Expr) reads, but two directive forms
/// need structured re-emission so their checks match Vue semantics:
///
/// - [`ModelTarget`](JsxEmit::ModelTarget): a `v-model` binding target re-emitted
///   as an assignment to itself so TypeScript checks the target is a writable
///   lvalue (binding to a `const`, a `readonly`/computed value, or a non-lvalue
///   expression is reported at the binding).
/// - [`ForScope`](JsxEmit::ForScope): a `v-for` (idiomatic `items.map(…)`) whose
///   body is re-emitted *inside* the `.map()` callback so the loop aliases are
///   bound with their inferred element types — both fixing a spurious
///   "Cannot find name '<alias>'" and checking the body against the real type.
enum JsxEmit {
    /// A plain dynamic expression (interpolation, bound attribute, directive
    /// value, `v-if`/`v-show` condition, event handler, …).
    Expr(JsxExpr),
    /// A `v-model` binding target, re-emitted as `(<lvalue> = <lvalue>)`.
    ModelTarget(JsxExpr),
    /// A `v-for` scope: the iterated `source` plus the alias patterns and the
    /// body units evaluated with those aliases in scope.
    ForScope {
        source: JsxExpr,
        value_alias: Option<JsxExpr>,
        key_alias: Option<JsxExpr>,
        body: Vec<JsxEmit>,
    },
}

/// Lower a `.jsx`/`.tsx` Vize component to plain virtual TypeScript.
pub(super) fn generate_jsx_virtual_ts(
    path: &Path,
    source: &str,
    lang: JsxLang,
) -> CorsaResult<GeneratedJsxFile> {
    let bump = Bump::new();
    let lowered = lower_source(&bump, source, lang);

    // Collect every outermost JSX root's byte range together with the dynamic
    // expressions inside it, in source order.
    let mut roots: Vec<(u32, u32, Vec<JsxEmit>)> = Vec::with_capacity(lowered.roots.len());
    for root in &lowered.roots {
        let mut exprs = Vec::new();
        collect_root_expressions(&root.root, &mut exprs);
        // The `<style scoped>` block is extracted out of the rendered children
        // (#1495), so its template-literal interpolations (`${expr}`) never reach
        // the lowered tree above. Append them as plain reads so they type-check
        // against the very same component scope (props, setup vars, ctx) as the
        // root's JSX expressions, source-mapped back to their `.tsx` ranges
        // (#1497).
        collect_style_expressions(&root.scoped_style_exprs, &mut exprs);
        roots.push((root.root.loc.start.offset, root.root.loc.end.offset, exprs));
    }
    // Outermost roots never overlap and are produced in source order, but guard
    // the rewrite against any accidental disorder.
    roots.sort_by_key(|(start, _, _)| *start);

    let mut diagnostics = Vec::new();
    for diagnostic in &lowered.diagnostics {
        if !diagnostic.is_error() {
            continue;
        }
        diagnostics.push(diagnostic_for_offset(
            path,
            source,
            diagnostic.start,
            jsx_parse_message(diagnostic),
            SfcBlockType::Script,
        ));
    }

    let (code, mappings) = render_plain_ts(source, &roots);

    Ok(GeneratedJsxFile {
        code,
        mappings,
        diagnostics,
    })
}

fn jsx_parse_message(diagnostic: &JsxDiagnostic) -> CompactString {
    cstr!("JSX parse error: {}", diagnostic.message)
}

/// Build the plain-`.ts` text and its source mappings.
///
/// Every byte outside a JSX render root is copied verbatim; each render root is
/// replaced by `__vize_jsx_expr__(<unit>, <unit>, …)`, with each re-emitted
/// expression mapped back to its original byte range.
fn render_plain_ts(
    source: &str,
    roots: &[(u32, u32, Vec<JsxEmit>)],
) -> (CompactString, Vec<VizeMapping>) {
    let mut out = CompactString::default();
    let mut mappings: Vec<VizeMapping> = Vec::new();

    // Ambient helpers: declared once at module scope so the re-emitted JSX
    // expressions and the synthesized render returns both type-check.
    out.push_str("declare function ");
    out.push_str(JSX_EXPR_SINK);
    out.push_str("(...args: unknown[]): any;\n");
    // Ambient `Ctx<Emits, Slots>` so the typed second parameter resolves and the
    // `emit`/`slots` usages in the setup body and JSX expressions type-check.
    out.push_str(CTX_HELPER);

    let mut cursor = 0usize;
    for (start, end, emits) in roots {
        let start = (*start as usize).min(source.len());
        let end = (*end as usize).min(source.len());
        if start < cursor {
            // Overlapping/disordered root: skip defensively.
            continue;
        }
        // Verbatim prefix (component function header, typed params, setup body).
        // Emit an identity mapping so diagnostics in this region (e.g. a wrong
        // `props.X` use in the setup body) map back to their true source range
        // despite the prepended ambient-helper preamble.
        push_verbatim(&mut out, &mut mappings, source, cursor, start);

        render_sink_call(&mut out, &mut mappings, emits);
        cursor = end.max(start);
    }
    // Trailing verbatim suffix (e.g. `export default Comp;`).
    push_verbatim(&mut out, &mut mappings, source, cursor, source.len());

    (out, mappings)
}

/// Emit `__vize_jsx_expr__(<unit>, <unit>, …)` for one render scope, recursing
/// into `v-for` bodies so their loop aliases stay in scope.
fn render_sink_call(out: &mut CompactString, mappings: &mut Vec<VizeMapping>, emits: &[JsxEmit]) {
    out.push_str(JSX_EXPR_SINK);
    out.push('(');
    for (index, emit) in emits.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        render_emit(out, mappings, emit);
    }
    out.push(')');
}

/// Re-emit one [`JsxEmit`] unit as a `__vize_jsx_expr__` argument, recording the
/// source mappings that point its diagnostics back at the original JSX.
fn render_emit(out: &mut CompactString, mappings: &mut Vec<VizeMapping>, emit: &JsxEmit) {
    match emit {
        JsxEmit::Expr(expr) => push_mapped_expr(out, mappings, expr),
        JsxEmit::ModelTarget(expr) => {
            // `v-model` binds a writable lvalue. Re-emit the target as an
            // assignment to itself so TypeScript reports binding to a `const`,
            // `readonly`/computed value, or a non-lvalue at the binding. Only the
            // left-hand side is mapped: assignability and name-resolution errors
            // land on the LHS, so the unmapped RHS copy never double-reports.
            out.push('(');
            push_mapped_expr(out, mappings, expr);
            out.push_str(" = ");
            out.push_str(&expr.content);
            out.push(')');
        }
        JsxEmit::ForScope {
            source,
            value_alias,
            key_alias,
            body,
        } => {
            // `(<source>).map((<value>, <key>) => __vize_jsx_expr__(<body…>))`:
            // the body is re-emitted inside the callback so the loop aliases bind
            // with their inferred element types. The `.map` scaffolding is left
            // unmapped (its diagnostics, if any, point at the mapped `source`).
            out.push('(');
            push_mapped_expr(out, mappings, source);
            out.push_str(").map((");
            if let Some(value) = value_alias {
                push_mapped_expr(out, mappings, value);
            } else {
                out.push_str("__vize_v");
            }
            if let Some(key) = key_alias {
                out.push_str(", ");
                push_mapped_expr(out, mappings, key);
            }
            out.push_str(") => ");
            render_sink_call(out, mappings, body);
            out.push(')');
        }
    }
}

/// Copy a re-emitted expression's text into `out` and record the mapping from
/// its generated range back to its original `.jsx`/`.tsx` byte range.
fn push_mapped_expr(out: &mut CompactString, mappings: &mut Vec<VizeMapping>, expr: &JsxExpr) {
    let gen_start = out.len();
    out.push_str(&expr.content);
    let gen_end = out.len();
    mappings.push(VizeMapping {
        gen_range: gen_start..gen_end,
        src_range: expr.start as usize..expr.end as usize,
        sub_spans: Vec::new(),
    });
}

/// Copy `source[src_start..src_end)` verbatim into `out`, recording an identity
/// mapping (generated range -> original range) for diagnostics in the region.
fn push_verbatim(
    out: &mut CompactString,
    mappings: &mut Vec<VizeMapping>,
    source: &str,
    src_start: usize,
    src_end: usize,
) {
    if src_start >= src_end {
        return;
    }
    let gen_start = out.len();
    out.push_str(&source[src_start..src_end]);
    let gen_end = out.len();
    mappings.push(VizeMapping {
        gen_range: gen_start..gen_end,
        src_range: src_start..src_end,
        sub_spans: Vec::new(),
    });
}

// ---------------------------------------------------------------------------
// Expression collection: walk the lowered relief tree and gather every dynamic
// (non-static) expression's source text and byte range.
// ---------------------------------------------------------------------------

fn collect_root_expressions(root: &RootNode<'_>, out: &mut Vec<JsxEmit>) {
    for child in &root.children {
        collect_child(child, out);
    }
}

/// Append a component's `<style scoped>` template-literal interpolations as
/// plain reads.
///
/// The style block is extracted out of the rendered tree (#1495), so its
/// `${expr}` interpolations are recovered separately on
/// [`LoweredRoot::scoped_style_exprs`](vize_atelier_jsx::LoweredRoot). Each
/// references script values in the component scope (`props`, setup-scope
/// bindings, the `Ctx` context), so re-emitting it through the same sink and
/// scope as the root's JSX expressions type-checks it, with its mapping pointing
/// diagnostics back at the original `${…}` byte range (#1497).
///
/// CSS `v-bind(expr)` references are *not* handled here (see the deferral note in
/// the module docs): they live in the cooked CSS text whose offsets no longer map
/// to source bytes, so recovering their spans needs dedicated extraction infra.
fn collect_style_expressions(style_exprs: &[StyleExprSpan], out: &mut Vec<JsxEmit>) {
    for style_expr in style_exprs {
        if let Some(expr) = jsx_expr(&style_expr.content, style_expr.start, style_expr.end) {
            out.push(JsxEmit::Expr(expr));
        }
    }
}

fn collect_child(child: &TemplateChildNode<'_>, out: &mut Vec<JsxEmit>) {
    match child {
        TemplateChildNode::Element(element) => {
            for prop in &element.props {
                collect_prop(prop, out);
            }
            for child in &element.children {
                collect_child(child, out);
            }
        }
        TemplateChildNode::Interpolation(interpolation) => {
            collect_expression(&interpolation.content, out);
        }
        TemplateChildNode::CompoundExpression(compound) => {
            collect_compound(compound, out);
        }
        TemplateChildNode::If(node) => {
            for branch in &node.branches {
                if let Some(condition) = &branch.condition {
                    collect_expression(condition, out);
                }
                for child in &branch.children {
                    collect_child(child, out);
                }
            }
        }
        TemplateChildNode::IfBranch(branch) => {
            if let Some(condition) = &branch.condition {
                collect_expression(condition, out);
            }
            for child in &branch.children {
                collect_child(child, out);
            }
        }
        TemplateChildNode::For(node) => {
            // The loop body is re-emitted *inside* the `.map()` callback so its
            // aliases (`value`, `key`) bind with their inferred element types,
            // both fixing a spurious "Cannot find name '<alias>'" and checking
            // the body against the real type. The `source` is the iterated value.
            let Some(source) = expr_of(&node.source) else {
                // A static/empty source cannot be iterated meaningfully; fall
                // back to just walking the body so nothing is silently dropped.
                for child in &node.children {
                    collect_child(child, out);
                }
                return;
            };
            let mut body = Vec::new();
            for child in &node.children {
                collect_child(child, &mut body);
            }
            out.push(JsxEmit::ForScope {
                source,
                value_alias: node.value_alias.as_ref().and_then(alias_expr),
                key_alias: node.key_alias.as_ref().and_then(alias_expr),
                body,
            });
        }
        TemplateChildNode::TextCall(node) => {
            collect_text_call(&node.content, out);
        }
        TemplateChildNode::Text(_)
        | TemplateChildNode::Comment(_)
        | TemplateChildNode::Hoisted(_) => {}
    }
}

fn collect_text_call(content: &vize_relief::TextCallContent<'_>, out: &mut Vec<JsxEmit>) {
    use vize_relief::TextCallContent;
    match content {
        TextCallContent::Interpolation(interpolation) => {
            collect_expression(&interpolation.content, out);
        }
        TextCallContent::Compound(compound) => collect_compound(compound, out),
        TextCallContent::Text(_) => {}
    }
}

fn collect_prop(prop: &PropNode<'_>, out: &mut Vec<JsxEmit>) {
    match prop {
        // Static `class="a"` style attributes carry only literal text.
        PropNode::Attribute(_) => {}
        PropNode::Directive(directive) => {
            // `v-model`'s value expression is the binding target: re-emit it as
            // an assignment so a `const`/`readonly`/non-lvalue binding is reported
            // at the binding. Other directive values (`v-show`, `v-if`, custom
            // `v-x:arg={…}`, `v-on` handlers, bound attributes) are plain reads.
            if directive.name.as_str() == "model" {
                if let Some(exp) = &directive.exp
                    && let Some(target) = expr_of(exp)
                {
                    out.push(JsxEmit::ModelTarget(target));
                }
            } else if let Some(exp) = &directive.exp {
                collect_expression(exp, out);
            }
            if let Some(arg) = &directive.arg {
                collect_expression(arg, out);
            }
        }
    }
}

fn collect_expression(expression: &ExpressionNode<'_>, out: &mut Vec<JsxEmit>) {
    match expression {
        ExpressionNode::Simple(simple) => {
            if simple.is_static {
                return;
            }
            push_expr(&simple.content, &simple.loc, out);
        }
        ExpressionNode::Compound(compound) => collect_compound(compound, out),
    }
}

fn collect_compound(compound: &CompoundExpressionNode<'_>, out: &mut Vec<JsxEmit>) {
    for child in &compound.children {
        match child {
            CompoundExpressionChild::Simple(simple) => {
                if !simple.is_static {
                    push_expr(&simple.content, &simple.loc, out);
                }
            }
            CompoundExpressionChild::Compound(compound) => collect_compound(compound, out),
            CompoundExpressionChild::Interpolation(interpolation) => {
                collect_expression(&interpolation.content, out);
            }
            CompoundExpressionChild::Text(_)
            | CompoundExpressionChild::String(_)
            | CompoundExpressionChild::Symbol(_) => {}
        }
    }
}

fn push_expr(content: &str, loc: &vize_relief::SourceLocation, out: &mut Vec<JsxEmit>) {
    if let Some(expr) = jsx_expr(content, loc.start.offset, loc.end.offset) {
        out.push(JsxEmit::Expr(expr));
    }
}

/// Build a [`JsxExpr`] from a dynamic simple [`ExpressionNode`], or `None` when
/// the expression is static or trims to empty (e.g. a directive with no value).
fn expr_of(expression: &ExpressionNode<'_>) -> Option<JsxExpr> {
    match expression {
        ExpressionNode::Simple(simple) if !simple.is_static => jsx_expr(
            &simple.content,
            simple.loc.start.offset,
            simple.loc.end.offset,
        ),
        _ => None,
    }
}

/// The source text of a `v-for` alias binding pattern (the lowering stores each
/// alias as a dynamic simple expression of the pattern), or `None` when absent.
fn alias_expr(alias: &ExpressionNode<'_>) -> Option<JsxExpr> {
    match alias {
        ExpressionNode::Simple(simple) => {
            let content = simple.content.trim();
            (!content.is_empty()).then(|| JsxExpr {
                content: content.to_compact_string(),
                start: simple.loc.start.offset,
                end: simple.loc.end.offset,
            })
        }
        ExpressionNode::Compound(_) => None,
    }
}

/// Trim `content` and pair it with its byte range, or `None` when empty.
fn jsx_expr(content: &str, start: u32, end: u32) -> Option<JsxExpr> {
    let content = content.trim();
    (!content.is_empty()).then(|| JsxExpr {
        content: content.to_compact_string(),
        start,
        end,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range;

    fn generate(source: &str) -> GeneratedJsxFile {
        generate_jsx_virtual_ts(Path::new("Comp.tsx"), source, JsxLang::Tsx).unwrap()
    }

    fn assert_generated_snapshot(name: &str, source: &str) {
        let generated = generate(source);
        insta::assert_snapshot!(format!("{name}_code"), generated.code.as_str());
        insta::assert_debug_snapshot!(
            format!("{name}_mappings"),
            mapping_summary(source, &generated)
        );
        insta::assert_debug_snapshot!(format!("{name}_diagnostics"), generated.diagnostics);
    }

    fn mapping_summary<'a>(
        source: &'a str,
        generated: &'a GeneratedJsxFile,
    ) -> Vec<MappingSummary<'a>> {
        generated
            .mappings
            .iter()
            .map(|mapping| MappingSummary {
                generated: &generated.code[mapping.gen_range.clone()],
                source: &source[mapping.src_range.clone()],
                gen_range: mapping.gen_range.clone(),
                src_range: mapping.src_range.clone(),
            })
            .collect()
    }

    #[allow(dead_code)]
    #[derive(Debug)]
    struct MappingSummary<'a> {
        generated: &'a str,
        source: &'a str,
        gen_range: Range<usize>,
        src_range: Range<usize>,
    }

    #[test]
    fn typed_component_with_jsx_control_flow_directives_and_styles_is_exact() {
        let source = "import { computed, ref } from 'vue';\n\nconst Comp = (\n  { items, ok, tone, gap }: { items: Array<{ id: string; label: string }>; ok: boolean; tone: string; gap: number },\n  { emit, slots }: Ctx<{ select: [id: string] }, { footer: () => unknown }>,\n) => {\n  const selected = ref(items[0]?.id);\n  const activeItem = computed(() => items.find((item) => item.id === selected.value));\n  return (\n    <>\n      <ul class={tone} v-show={ok}>\n        {items.map((item, index) => (\n          <li key={item.id} onClick={() => emit('select', item.id)} data-index={index}>\n            {item.label}{selected.value === item.id ? <strong>Selected</strong> : <em>{index}</em>}\n          </li>\n        ))}\n      </ul>\n      <input v-model={selected.value} v-focus:lazy={tone} />\n      <footer>{activeItem.value?.label}{slots.footer()}</footer>\n      <style scoped>{`.row { gap: ${gap}px; }`}</style>\n    </>\n  );\n};\n";

        assert_generated_snapshot(
            "typed_component_with_jsx_control_flow_directives_and_styles",
            source,
        );
    }

    #[test]
    fn multiple_roots_and_static_style_are_exact() {
        let source = "const First = (props: { msg: string }) => <section>{props.msg}</section>;\nconst Second = () => (\n  <>\n    <div class=\"box\" />\n    <style scoped>{`.box { color: red; }`}</style>\n  </>\n);\n";

        assert_generated_snapshot("multiple_roots_and_static_style", source);
    }

    #[test]
    fn jsx_file_mode_is_exact() {
        let source =
            "export const Plain = ({ msg }) => <button onClick={() => save(msg)}>{msg}</button>;\n";
        let generated =
            generate_jsx_virtual_ts(Path::new("Plain.jsx"), source, JsxLang::Jsx).unwrap();

        insta::assert_snapshot!("jsx_file_mode_code", generated.code.as_str());
        insta::assert_debug_snapshot!(
            "jsx_file_mode_mappings",
            mapping_summary(source, &generated)
        );
        insta::assert_debug_snapshot!("jsx_file_mode_diagnostics", generated.diagnostics);
    }
}
