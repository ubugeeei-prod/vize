//! Generating plain `.ts` virtual TypeScript for `.jsx`/`.tsx` Vue components
//! (issue #1497, opt-in).
//!
//! This is the JSX/TSX parallel to [`super::vue_codegen`]. It is reached only
//! when the user explicitly enables `typeChecker.jsxTypecheck` (default off):
//! JSX support is experimental and a repository may contain React `.tsx` files
//! that must *not* be type-checked as Vue JSX.
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
        value_alias: Option<CompactString>,
        key_alias: Option<CompactString>,
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
                out.push_str(value);
            } else {
                out.push_str("__vize_v");
            }
            if let Some(key) = key_alias {
                out.push_str(", ");
                out.push_str(key);
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
                value_alias: node.value_alias.as_ref().and_then(alias_text),
                key_alias: node.key_alias.as_ref().and_then(alias_text),
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
fn alias_text(alias: &ExpressionNode<'_>) -> Option<CompactString> {
    match alias {
        ExpressionNode::Simple(simple) => {
            let content = simple.content.trim();
            (!content.is_empty()).then(|| content.to_compact_string())
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

    fn generate(source: &str) -> GeneratedJsxFile {
        generate_jsx_virtual_ts(Path::new("Comp.tsx"), source, JsxLang::Tsx).unwrap()
    }

    #[test]
    fn keeps_typed_props_param_verbatim_and_reemits_jsx_expr() {
        let source =
            "const Comp = (props: { msg: string }) => <div class=\"a\">{props.msg}</div>;\n";
        let generated = generate(source);
        // The typed first parameter stays verbatim so `props.msg` type-checks.
        assert!(
            generated.code.contains("props: { msg: string }"),
            "typed props param dropped: {}",
            generated.code
        );
        // The JSX expression is re-emitted as plain TS through the sink helper.
        assert!(
            generated.code.contains("__vize_jsx_expr__(props.msg)"),
            "jsx expr not re-emitted: {}",
            generated.code
        );
        // Output is plain TS: no JSX element syntax survives.
        assert!(
            !generated.code.contains("<div"),
            "JSX element leaked into virtual TS: {}",
            generated.code
        );
        // A mapping points the re-emitted `props.msg` back at its source range.
        let src = source.find("props.msg").unwrap();
        assert!(
            generated
                .mappings
                .iter()
                .any(|mapping| mapping.src_range.start == src),
            "no source mapping for re-emitted expression"
        );
    }

    #[test]
    fn setup_scope_statements_stay_verbatim() {
        let source = "const Comp = (props: { n: number }) => {\n  const doubled = props.n * 2;\n  return <span>{doubled}</span>;\n};\n";
        let generated = generate(source);
        assert!(generated.code.contains("const doubled = props.n * 2;"));
        assert!(generated.code.contains("__vize_jsx_expr__(doubled)"));
    }

    #[test]
    fn collects_bound_attribute_expressions() {
        let source = "const Comp = (props: { cls: string }) => <div class={props.cls}>hi</div>;\n";
        let generated = generate(source);
        assert!(generated.code.contains("__vize_jsx_expr__(props.cls)"));
    }

    #[test]
    fn injects_ambient_ctx_helper() {
        // The ambient `Ctx<Emits, Slots>` is declared at module scope, with
        // `emit` typed via the same `__EmitFn` emits-as-tuple convention as the
        // `.vue` path so a component's typed second parameter resolves.
        let generated = generate("const Comp = () => <div>hi</div>;\n");
        assert!(
            generated
                .code
                .contains("type Ctx<Emits = {}, Slots = {}> = { emit: __EmitFn<Emits>;"),
            "ambient Ctx not injected: {}",
            generated.code
        );
        assert!(
            generated.code.contains("type __EmitFn<T>"),
            "emit-typing helper not injected: {}",
            generated.code
        );
    }

    #[test]
    fn keeps_typed_ctx_param_verbatim_and_reemits_emit_call() {
        // `props` is the typed first parameter; `{ emit }: Ctx<…>` is the typed
        // second parameter. Both stay verbatim, and the `emit(...)` call in a
        // JSX expression is re-emitted as plain TS so its payload type-checks.
        let source = "const Comp = (\n  props: { msg: string },\n  { emit }: Ctx<{ change: [value: number] }>,\n) => <button onClick={() => emit('change', 1)}>{props.msg}</button>;\n";
        let generated = generate(source);
        // The typed second parameter is preserved verbatim.
        assert!(
            generated
                .code
                .contains("{ emit }: Ctx<{ change: [value: number] }>"),
            "typed Ctx param dropped: {}",
            generated.code
        );
        // The `emit(...)` call inside the JSX handler is re-emitted as plain TS.
        assert!(
            generated.code.contains("emit('change', 1)"),
            "emit call not re-emitted: {}",
            generated.code
        );
        // Output stays plain TS.
        assert!(
            !generated.code.contains("<button"),
            "JSX element leaked into virtual TS: {}",
            generated.code
        );
    }

    #[test]
    fn reemits_slots_usage_from_typed_ctx_param() {
        // `slots` comes from the typed second parameter and is re-emitted inside
        // the JSX expression so slot access type-checks against `Slots`.
        let source = "const Comp = (\n  _props: {},\n  { slots }: Ctx<{}, { default: () => unknown }>,\n) => <div>{slots.default()}</div>;\n";
        let generated = generate(source);
        assert!(
            generated
                .code
                .contains("{ slots }: Ctx<{}, { default: () => unknown }>"),
            "typed Ctx slots param dropped: {}",
            generated.code
        );
        assert!(
            generated
                .code
                .contains("__vize_jsx_expr__(slots.default())"),
            "slots usage not re-emitted: {}",
            generated.code
        );
    }

    #[test]
    fn reemits_v_model_target_as_self_assignment() {
        // A `v-model` binding target is re-emitted as `(target = target)` so a
        // readonly/const/non-lvalue binding is reported at the binding, while the
        // mapped left-hand side keeps name-resolution errors at the right place.
        let source = "const Comp = (model: { value: string }) => <input v-model={model.value}/>;\n";
        let generated = generate(source);
        assert!(
            generated
                .code
                .contains("__vize_jsx_expr__((model.value = model.value))"),
            "v-model target not re-emitted as assignment: {}",
            generated.code
        );
        // Output stays plain TS.
        assert!(
            !generated.code.contains("<input"),
            "JSX element leaked into virtual TS: {}",
            generated.code
        );
        // Only the left-hand side is mapped back to source (the unmapped RHS copy
        // exists so the assignment is well-formed but never double-reports).
        let lvalue_start = source.find("model.value").unwrap();
        assert_eq!(
            generated
                .mappings
                .iter()
                .filter(|mapping| mapping.src_range.start == lvalue_start)
                .count(),
            1,
            "expected exactly one mapping for the v-model lvalue: {:?}",
            generated.mappings
        );
    }

    #[test]
    fn reemits_v_for_body_inside_map_callback_binding_alias() {
        // The `items.map((item) => …)` body is re-emitted *inside* the callback so
        // `item` binds with its inferred element type — fixing the spurious
        // "Cannot find name 'item'" the flat collection produced.
        let source = "const Comp = (props: { items: number[] }) => <ul>{props.items.map((item) => <li>{item}</li>)}</ul>;\n";
        let generated = generate(source);
        assert!(
            generated
                .code
                .contains("(props.items).map((item) => __vize_jsx_expr__(item))"),
            "v-for body not bound inside .map callback: {}",
            generated.code
        );
        // The bare alias must not leak as a sink argument at the outer scope.
        assert!(
            !generated
                .code
                .contains("__vize_jsx_expr__(props.items, item)"),
            "v-for alias leaked as unbound outer argument: {}",
            generated.code
        );
        // Stays plain TS.
        assert!(
            !generated.code.contains("<li"),
            "JSX element leaked into virtual TS: {}",
            generated.code
        );
    }

    #[test]
    fn reemits_v_for_with_index_alias() {
        // A two-arg `.map((value, index) => …)` binds both aliases in the callback.
        let source = "const Comp = (props: { xs: string[] }) => <ul>{props.xs.map((x, i) => <li>{x + i}</li>)}</ul>;\n";
        let generated = generate(source);
        assert!(
            generated
                .code
                .contains("(props.xs).map((x, i) => __vize_jsx_expr__(x + i))"),
            "v-for value+index aliases not bound: {}",
            generated.code
        );
    }

    #[test]
    fn collects_v_show_and_v_if_conditions_as_plain_reads() {
        // Directive conditions stay plain reads (no lvalue rewrite) so an unknown
        // identifier in a `v-show`/`v-if` condition is reported at the condition.
        let show = generate("const Comp = (props: { ok: boolean }) => <div v-show={props.ok}/>;\n");
        assert!(
            show.code.contains("__vize_jsx_expr__(props.ok)"),
            "v-show condition not re-emitted: {}",
            show.code
        );
        let if_attr =
            generate("const Comp = (props: { ok: boolean }) => <div v-if={props.ok}>x</div>;\n");
        assert!(
            if_attr.code.contains("__vize_jsx_expr__(props.ok)"),
            "v-if condition not re-emitted: {}",
            if_attr.code
        );
    }

    #[test]
    fn reemits_style_block_interpolation_through_sink() {
        // A `<style scoped>` template-literal interpolation references a script
        // value (`props.color`); it is extracted out of the rendered children but
        // re-emitted through the sink so it type-checks against the component
        // scope, source-mapped back to its `${…}` range.
        let source = "const Comp = (props: { color: string }) => (\n  <>\n    <div class=\"box\">hi</div>\n    <style scoped>{`.box { color: ${props.color}; }`}</style>\n  </>\n);\n";
        let generated = generate(source);
        assert!(
            generated.code.contains("__vize_jsx_expr__(props.color)"),
            "style interpolation not re-emitted: {}",
            generated.code
        );
        // Output stays plain TS: no `<style>` element survives.
        assert!(
            !generated.code.contains("<style"),
            "style element leaked into virtual TS: {}",
            generated.code
        );
        // A mapping points the re-emitted interpolation back at its source range.
        let src = source.find("props.color").unwrap();
        assert!(
            generated
                .mappings
                .iter()
                .any(|mapping| mapping.src_range.start == src),
            "no source mapping for re-emitted style interpolation: {:?}",
            generated.mappings
        );
    }

    #[test]
    fn reemits_multiple_style_block_interpolations() {
        // Every `${expr}` in the style block is re-emitted, in source order.
        let source = "const Comp = (props: { fg: string; bg: string }) => (\n  <>\n    <div class=\"box\"/>\n    <style scoped>{`.box { color: ${props.fg}; background: ${props.bg}; }`}</style>\n  </>\n);\n";
        let generated = generate(source);
        assert!(
            generated.code.contains("props.fg") && generated.code.contains("props.bg"),
            "not all style interpolations re-emitted: {}",
            generated.code
        );
    }

    #[test]
    fn static_style_block_emits_no_extra_sink_args() {
        // A static `<style scoped>` (no `${}`) contributes no re-emitted
        // expressions: the sink call stays empty for an otherwise-static root.
        let source = "const Comp = () => (\n  <>\n    <div class=\"box\"/>\n    <style scoped>{`.box { color: red; }`}</style>\n  </>\n);\n";
        let generated = generate(source);
        assert!(
            generated.code.contains("__vize_jsx_expr__()"),
            "static style block should not add sink arguments: {}",
            generated.code
        );
        assert!(
            !generated.code.contains("<style"),
            "style element leaked into virtual TS: {}",
            generated.code
        );
    }

    #[test]
    fn reemits_event_handler_and_custom_directive_value() {
        // Event handlers and custom directive values remain plain reads.
        let handler = generate(
            "const Comp = (props: { f: () => void }) => <button onClick={props.f}>x</button>;\n",
        );
        assert!(
            handler.code.contains("__vize_jsx_expr__(props.f)"),
            "event handler not re-emitted: {}",
            handler.code
        );
        let custom = generate(
            "const Comp = (props: { o: string }) => <div v-focus:lazy={props.o}>x</div>;\n",
        );
        assert!(
            custom.code.contains("__vize_jsx_expr__(props.o)"),
            "custom directive value not re-emitted: {}",
            custom.code
        );
    }
}
