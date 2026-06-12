//! Per-document JSX/TSX → plain-`.ts` virtual TypeScript for the LSP (#1498).
//!
//! This mirrors `vize_canon`'s batch `jsx_codegen` lowering so the editor's
//! virtual TypeScript matches the type-checker's byte-for-byte. The standing
//! maintainer directive is that JSX/TSX virtual TypeScript stays **plain
//! `.ts`** — never a TSX-format virtual document — so this pass keeps every
//! non-JSX byte of the source verbatim (component functions, the typed props
//! parameter, the setup body) and replaces only the JSX render roots with a
//! synthesized `__vize_jsx_expr__(<expr>, …)` call that re-lists every embedded
//! dynamic JSX expression as plain TypeScript at — and source-mapped back to —
//! its original byte range.
//!
//! The canon batch path (`crates/vize_canon/src/batch/virtual_project/
//! jsx_codegen.rs`) owns the same lowering for `vize check`. Both consume
//! [`vize_atelier_jsx::lower_source`] and re-emit the same expressions, so a
//! diagnostic Corsa reports against this document lands at the identical source
//! range it would land at on the CLI. The two implementations are deliberately
//! kept small and in lock-step rather than sharing a cross-crate export: the
//! canon generator is module-private and its surrounding `Diagnostic`/block
//! machinery is batch-specific.

use vize_atelier_jsx::{JsxLang, lower_source};
use vize_canon::virtual_ts::VizeMapping;
use vize_carton::Bump;
use vize_relief::ast::{
    ExpressionNode, RootNode, TemplateChildNode,
    elements::PropNode,
    expressions::{CompoundExpressionChild, CompoundExpressionNode},
};

/// Name of the synthesized helper that swallows every re-emitted JSX
/// expression. Declaring it ambient and `any`-returning lets each argument be
/// type-checked independently while the whole call stays a valid render return.
const JSX_EXPR_SINK: &str = "__vize_jsx_expr__";

/// Ambient `Ctx<Emits, Slots>` type injected at module scope so the typed
/// second parameter of a Vize JSX/TSX component (`{ emit, slots }: Ctx<…>`)
/// resolves and type-checks (#1502).
///
/// Copied verbatim from `vize_canon`'s batch `jsx_codegen` `CTX_HELPER` so the
/// LSP virtual TS resolves `Ctx`, `emit`, and `slots` identically to the
/// type-checker — without it a `.tsx` using the `Ctx` second parameter would
/// raise a spurious `Cannot find name 'Ctx'` diagnostic in the editor. `emit`
/// reuses the emits-as-tuple convention (`emit('change', 1)` checks the payload
/// against the declared tuple); `slots` is the second type argument. The type
/// is purely ambient and fully erased — no runtime is emitted.
const CTX_HELPER: &str = "type __EmitShape<T> = T extends (...args: any[]) => any ? T : T extends Record<string, any> ? { [K in keyof T]: T[K] extends (...args: infer A) => any ? A : T[K] extends any[] ? T[K] : any[]; } : Record<string, any[]>;\n\
type __EmitArgs<T, K extends keyof T> = T[K] extends any[] ? T[K] : any[];\n\
type __EmitFn<T> = __EmitShape<T> extends (...args: any[]) => any ? __EmitShape<T> : (<K extends keyof __EmitShape<T>>(event: K, ...args: __EmitArgs<__EmitShape<T>, K>) => void);\n\
type Ctx<Emits = {}, Slots = {}> = { emit: __EmitFn<Emits>; slots: Slots; attrs: Record<string, unknown>; };\n";

/// The generated plain-`.ts` virtual document for one `.jsx`/`.tsx` source.
pub(in crate::ide) struct JsxVirtualTs {
    /// Generated plain TypeScript.
    pub(in crate::ide) code: String,
    /// Byte-range mappings from generated TS back to the original source.
    pub(in crate::ide) mappings: Vec<VizeMapping>,
}

/// A dynamic JSX expression recovered from the lowered tree: its original
/// source text plus the byte range it occupied in the `.jsx`/`.tsx` source.
pub(in crate::ide) struct JsxExpr {
    pub(in crate::ide) content: String,
    pub(in crate::ide) start: u32,
    pub(in crate::ide) end: u32,
}

/// Collect every dynamic (non-static) JSX expression in `source` with its
/// original source byte range, in source order.
///
/// Shares the exact lowering + expression walk that builds the virtual TS, so
/// callers (e.g. semantic tokens) see the same set of expressions, at the same
/// spans, that the type-aware features re-emit. Returns the expressions across
/// all render roots flattened into one list.
pub(in crate::ide) fn collect_jsx_expressions(source: &str, lang: JsxLang) -> Vec<JsxExpr> {
    let bump = Bump::new();
    let lowered = lower_source(&bump, source, lang);
    let mut exprs = Vec::new();
    for root in &lowered.roots {
        collect_root_expressions(&root.root, &mut exprs);
    }
    exprs.sort_by_key(|expr| expr.start);
    exprs
}

/// Lower a `.jsx`/`.tsx` Vize component to plain virtual TypeScript.
///
/// Returns `None` only when lowering cannot proceed at all (it never does
/// today — `lower_source` always yields a tree even for empty input — but the
/// signature leaves room for that without forcing callers to handle a panic).
pub(in crate::ide) fn generate_jsx_virtual_ts(source: &str, lang: JsxLang) -> Option<JsxVirtualTs> {
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

    let (code, mappings) = render_plain_ts(source, &roots);
    Some(JsxVirtualTs { code, mappings })
}

/// Build the plain-`.ts` text and its source mappings.
///
/// Every byte outside a JSX render root is copied verbatim; each render root is
/// replaced by `__vize_jsx_expr__(<expr>, <expr>, …)`, with each re-emitted
/// expression mapped back to its original byte range.
fn render_plain_ts(source: &str, roots: &[(u32, u32, Vec<JsxExpr>)]) -> (String, Vec<VizeMapping>) {
    let mut out = String::new();
    let mut mappings: Vec<VizeMapping> = Vec::new();

    // Ambient helpers: declared once at module scope so the re-emitted JSX
    // expressions and the synthesized render returns both type-check.
    out.push_str("declare function ");
    out.push_str(JSX_EXPR_SINK);
    out.push_str("(...args: unknown[]): any;\n");
    // Ambient `Ctx<Emits, Slots>` so the typed second parameter resolves and the
    // `emit`/`slots` usages in the setup body and JSX expressions type-check.
    // Mirrors the canon batch path so the LSP virtual TS matches the checker.
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
        // Emit an identity mapping so a wrong `props.X` use in the setup body
        // maps back to its true source range despite the prepended
        // ambient-helper preamble.
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
    out: &mut String,
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
// (non-static) expression's source text and byte range. Mirrors the canon
// batch jsx_codegen walker.
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
        content: content.to_string(),
        start: loc.start.offset,
        end: loc.end.offset,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ide::jsx::position::source_offset_to_virtual_position;

    fn generate(source: &str) -> JsxVirtualTs {
        generate_jsx_virtual_ts(source, JsxLang::Tsx).unwrap()
    }

    #[test]
    fn keeps_typed_props_param_verbatim_and_reemits_jsx_expr() {
        let source =
            "const Comp = (props: { msg: string }) => <div class=\"a\">{props.msg}</div>;\n";
        let generated = generate(source);
        assert!(
            generated.code.contains("props: { msg: string }"),
            "typed props param dropped: {}",
            generated.code
        );
        assert!(
            generated.code.contains("__vize_jsx_expr__(props.msg)"),
            "jsx expr not re-emitted: {}",
            generated.code
        );
        // Plain TS: no JSX element syntax survives.
        assert!(
            !generated.code.contains("<div"),
            "JSX element leaked into virtual TS: {}",
            generated.code
        );
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

    /// Parity with the canon batch path: the ambient `Ctx<Emits, Slots>` type is
    /// declared at module scope so a `.tsx` using the typed second parameter
    /// resolves `Ctx`/`emit`/`slots` in the LSP virtual TS exactly as the
    /// type-checker does (no spurious `Cannot find name 'Ctx'`).
    #[test]
    fn injects_ambient_ctx_helper() {
        let source = "const Comp = (props: { n: number }, { emit }: Ctx<{ change: [value: number] }>) => {\n  emit('change', props.n);\n  return <span>{props.n}</span>;\n};\n";
        let generated = generate(source);
        assert!(
            generated.code.contains("type Ctx<Emits = {}, Slots = {}>"),
            "ambient Ctx helper missing: {}",
            generated.code
        );
        // The typed second parameter stays verbatim so `emit` type-checks.
        assert!(
            generated
                .code
                .contains("{ emit }: Ctx<{ change: [value: number] }>"),
            "typed Ctx param dropped: {}",
            generated.code
        );
        // The `emit(...)` call in the setup body is verbatim (checked in place).
        assert!(generated.code.contains("emit('change', props.n);"));
    }

    /// #1502 acceptance: hover/completion must reflect **props, emits, and
    /// slots** on a typed `.tsx` component. Hover/completion query the Corsa
    /// backend over this generated virtual TS, so the data they surface is
    /// exactly what this lowering preserves. Pin that all three typed surfaces of
    /// a fully-typed component — the props parameter type, the emits tuple, and
    /// the slots shape — survive into the virtual TS verbatim (so the type
    /// checker resolves each), and that a cursor on a `props.` / `slots.` access
    /// forward-maps into the virtual TS (so a hover/completion request lands on
    /// the typed member rather than falling off the mapping).
    #[test]
    fn typed_props_emits_and_slots_are_all_resolvable_in_virtual_ts() {
        let source = "const Comp = (\n  props: { msg: string },\n  { emit, slots }: Ctx<{ change: [value: number] }, { default: () => unknown }>,\n) => {\n  emit('change', 1);\n  return <div>{props.msg}{slots.default()}</div>;\n};\n";
        let generated = generate(source);

        // Props: the typed parameter is verbatim, so `props.msg` resolves to
        // `string` (what hover/completion would report).
        assert!(
            generated.code.contains("props: { msg: string }"),
            "typed props param dropped: {}",
            generated.code
        );
        // Emits + slots: the typed `Ctx<Emits, Slots>` second parameter is
        // verbatim and the ambient `Ctx` helper is injected, so `emit` and
        // `slots` both type-check against the declared shapes.
        assert!(
            generated.code.contains("type Ctx<Emits = {}, Slots = {}>"),
            "ambient Ctx helper missing: {}",
            generated.code
        );
        assert!(
            generated.code.contains(
                "{ emit, slots }: Ctx<{ change: [value: number] }, { default: () => unknown }>"
            ),
            "typed emits/slots param dropped: {}",
            generated.code
        );
        // The emits call and both the props and slots accesses are re-emitted as
        // plain TS expressions (the JSX render root collapses to the sink call),
        // so each is independently type-checked.
        assert!(generated.code.contains("emit('change', 1);"));
        assert!(
            generated.code.contains("props.msg"),
            "props access not re-emitted: {}",
            generated.code
        );
        assert!(
            generated.code.contains("slots.default()"),
            "slots access not re-emitted: {}",
            generated.code
        );

        // A hover/completion cursor on the `props.msg` access must forward-map
        // into the generated virtual TS (this is exactly what
        // `JsxService::prepare_request` does before querying Corsa). If the
        // mapping dropped the access, hover/completion would silently no-op.
        let props_access = source.find("props.msg").expect("props.msg present");
        assert!(
            source_offset_to_virtual_position(
                &generated.code,
                &generated.mappings,
                // Land the cursor on the member name (after the dot).
                props_access + "props.".len(),
            )
            .is_some(),
            "props member access did not forward-map into the virtual TS"
        );
        // Same for the `slots.default` access — proving slots access is reachable
        // by the type-aware features, not just present as text.
        let slots_access = source.find("slots.default").expect("slots.default present");
        assert!(
            source_offset_to_virtual_position(
                &generated.code,
                &generated.mappings,
                slots_access + "slots.".len(),
            )
            .is_some(),
            "slots member access did not forward-map into the virtual TS"
        );
    }
}
