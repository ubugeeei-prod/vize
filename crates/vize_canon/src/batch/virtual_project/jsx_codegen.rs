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
//!   location.
//!
//! Deferred (see issue #1497): directive / `v-model` / style-expression checks,
//! the stateful `defineComponent(() => () => VNode)` form, and full source-map
//! fidelity for the synthesized wrapper scaffolding.

use std::path::Path;

use vize_atelier_jsx::{JsxDiagnostic, JsxLang, lower_source};
use vize_carton::{Bump, String as CompactString, ToCompactString, cstr};
use vize_relief::ast::{
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
    let mut roots: Vec<(u32, u32, Vec<JsxExpr>)> = Vec::with_capacity(lowered.roots.len());
    for root in &lowered.roots {
        let mut exprs = Vec::new();
        collect_root_expressions(&root.root, &mut exprs);
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
/// replaced by `__vize_jsx_expr__(<expr>, <expr>, …)`, with each re-emitted
/// expression mapped back to its original byte range.
fn render_plain_ts(
    source: &str,
    roots: &[(u32, u32, Vec<JsxExpr>)],
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
    for (start, end, exprs) in roots {
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

        out.push_str(JSX_EXPR_SINK);
        out.push('(');
        for (index, expr) in exprs.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            let gen_start = out.len();
            out.push_str(&expr.content);
            let gen_end = out.len();
            mappings.push(VizeMapping {
                gen_range: gen_start..gen_end,
                src_range: expr.start as usize..expr.end as usize,
                sub_spans: Vec::new(),
            });
        }
        out.push(')');
        cursor = end.max(start);
    }
    // Trailing verbatim suffix (e.g. `export default Comp;`).
    push_verbatim(&mut out, &mut mappings, source, cursor, source.len());

    (out, mappings)
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

fn collect_root_expressions(root: &RootNode<'_>, out: &mut Vec<JsxExpr>) {
    for child in &root.children {
        collect_child(child, out);
    }
}

fn collect_child(child: &TemplateChildNode<'_>, out: &mut Vec<JsxExpr>) {
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
            collect_expression(&node.source, out);
            for child in &node.children {
                collect_child(child, out);
            }
        }
        TemplateChildNode::TextCall(node) => {
            collect_text_call(&node.content, out);
        }
        TemplateChildNode::Text(_)
        | TemplateChildNode::Comment(_)
        | TemplateChildNode::Hoisted(_) => {}
    }
}

fn collect_text_call(content: &vize_relief::ast::TextCallContent<'_>, out: &mut Vec<JsxExpr>) {
    use vize_relief::ast::TextCallContent;
    match content {
        TextCallContent::Interpolation(interpolation) => {
            collect_expression(&interpolation.content, out);
        }
        TextCallContent::Compound(compound) => collect_compound(compound, out),
        TextCallContent::Text(_) => {}
    }
}

fn collect_prop(prop: &PropNode<'_>, out: &mut Vec<JsxExpr>) {
    match prop {
        // Static `class="a"` style attributes carry only literal text.
        PropNode::Attribute(_) => {}
        PropNode::Directive(directive) => {
            if let Some(exp) = &directive.exp {
                collect_expression(exp, out);
            }
            if let Some(arg) = &directive.arg {
                collect_expression(arg, out);
            }
        }
    }
}

fn collect_expression(expression: &ExpressionNode<'_>, out: &mut Vec<JsxExpr>) {
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

fn collect_compound(compound: &CompoundExpressionNode<'_>, out: &mut Vec<JsxExpr>) {
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

fn push_expr(content: &str, loc: &vize_relief::ast::core::SourceLocation, out: &mut Vec<JsxExpr>) {
    let content = content.trim();
    if content.is_empty() {
        return;
    }
    out.push(JsxExpr {
        content: content.to_compact_string(),
        start: loc.start.offset,
        end: loc.end.offset,
    });
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
}
