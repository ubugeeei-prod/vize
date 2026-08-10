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

use vize_atelier_jsx::{JsxLang, StyleExprSpan, lower_source};
use vize_canon::virtual_ts::VizeMapping;
use vize_carton::Bump;
use vize_relief::{
    ExpressionNode, RootNode, TemplateChildNode,
    elements::PropNode,
    expressions::{CompoundExpressionChild, CompoundExpressionNode},
};

mod component;

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

/// One re-emitted unit recovered from a lowered JSX root, in source order.
///
/// This mirrors `vize_canon`'s batch JSX virtual-TS generator: plain expression
/// reads are emitted as-is, `v-model` targets become self-assignments, and
/// `items.map(...)` bodies are emitted inside the callback so aliases stay in
/// scope for hover, completion, and diagnostics.
enum JsxEmit {
    Expr(JsxExpr),
    ModelTarget(JsxExpr),
    Component(component::JsxComponent),
    ForScope {
        source: JsxExpr,
        value_alias: Option<JsxExpr>,
        key_alias: Option<JsxExpr>,
        body: Vec<JsxEmit>,
    },
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
        let mut emits = Vec::new();
        collect_root_expressions(&root.root, &mut emits, false);
        collect_style_expressions(&root.scoped_style_exprs, &mut emits);
        flatten_emits(&emits, &mut exprs);
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
    let mut roots: Vec<(u32, u32, Vec<JsxEmit>)> = Vec::with_capacity(lowered.roots.len());
    for root in &lowered.roots {
        let mut emits = Vec::new();
        collect_root_expressions(&root.root, &mut emits, true);
        collect_style_expressions(&root.scoped_style_exprs, &mut emits);
        roots.push((root.root.loc.start.offset, root.root.loc.end.offset, emits));
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
fn render_plain_ts(source: &str, roots: &[(u32, u32, Vec<JsxEmit>)]) -> (String, Vec<VizeMapping>) {
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
    if roots
        .iter()
        .any(|(_, _, emits)| emits.iter().any(emit_contains_component))
    {
        out.push_str(component::HELPER);
    }

    let mut cursor = 0usize;
    for (start, end, emits) in roots {
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

        render_sink_call(&mut out, &mut mappings, emits);
        cursor = end.max(start);
    }
    // Trailing verbatim suffix (e.g. `export default Comp;`).
    push_verbatim(&mut out, &mut mappings, source, cursor, source.len());

    (out, mappings)
}

/// Emit `__vize_jsx_expr__(<unit>, <unit>, …)` for one render scope, recursing
/// into `v-for` bodies so their loop aliases stay in scope.
fn render_sink_call(out: &mut String, mappings: &mut Vec<VizeMapping>, emits: &[JsxEmit]) {
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

fn render_emit(out: &mut String, mappings: &mut Vec<VizeMapping>, emit: &JsxEmit) {
    match emit {
        JsxEmit::Expr(expr) => push_mapped_expr(out, mappings, expr),
        JsxEmit::ModelTarget(expr) => {
            out.push('(');
            push_mapped_expr(out, mappings, expr);
            out.push_str(" = ");
            out.push_str(&expr.content);
            out.push(')');
        }
        JsxEmit::Component(component) => component::render(out, mappings, component),
        JsxEmit::ForScope {
            source,
            value_alias,
            key_alias,
            body,
        } => {
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

fn emit_contains_component(emit: &JsxEmit) -> bool {
    match emit {
        JsxEmit::Component(_) => true,
        JsxEmit::ForScope { body, .. } => body.iter().any(emit_contains_component),
        JsxEmit::Expr(_) | JsxEmit::ModelTarget(_) => false,
    }
}

fn push_mapped_expr(out: &mut String, mappings: &mut Vec<VizeMapping>, expr: &JsxExpr) {
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

fn collect_root_expressions(
    root: &RootNode<'_>,
    out: &mut Vec<JsxEmit>,
    preserve_components: bool,
) {
    for child in &root.children {
        collect_child(child, out, preserve_components);
    }
}

fn collect_style_expressions(style_exprs: &[StyleExprSpan], out: &mut Vec<JsxEmit>) {
    for style_expr in style_exprs {
        if let Some(expr) = jsx_expr(&style_expr.content, style_expr.start, style_expr.end) {
            out.push(JsxEmit::Expr(expr));
        }
    }
}

fn collect_child(child: &TemplateChildNode<'_>, out: &mut Vec<JsxEmit>, preserve_components: bool) {
    match child {
        TemplateChildNode::Element(element) => {
            let semantic_component = preserve_components
                .then(|| component::collect(element))
                .flatten();
            let has_semantic_component = semantic_component.is_some();
            if let Some(component) = semantic_component {
                out.push(JsxEmit::Component(component));
            }
            for prop in &element.props {
                if !has_semantic_component || !component::captures_prop(element, prop) {
                    collect_prop(prop, out);
                }
            }
            for child in &element.children {
                collect_child(child, out, preserve_components);
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
                    collect_child(child, out, preserve_components);
                }
            }
        }
        TemplateChildNode::IfBranch(branch) => {
            if let Some(condition) = &branch.condition {
                collect_expression(condition, out);
            }
            for child in &branch.children {
                collect_child(child, out, preserve_components);
            }
        }
        TemplateChildNode::For(node) => {
            let Some(source) = expr_of(&node.source) else {
                for child in &node.children {
                    collect_child(child, out, preserve_components);
                }
                return;
            };
            let mut body = Vec::new();
            for child in &node.children {
                collect_child(child, &mut body, preserve_components);
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

fn alias_expr(alias: &ExpressionNode<'_>) -> Option<JsxExpr> {
    match alias {
        ExpressionNode::Simple(simple) => {
            let content = simple.content.trim();
            (!content.is_empty()).then(|| JsxExpr {
                content: content.to_string(),
                start: simple.loc.start.offset,
                end: simple.loc.end.offset,
            })
        }
        ExpressionNode::Compound(_) => None,
    }
}

fn jsx_expr(content: &str, start: u32, end: u32) -> Option<JsxExpr> {
    let content = content.trim();
    (!content.is_empty()).then(|| JsxExpr {
        content: content.to_string(),
        start,
        end,
    })
}

fn flatten_emits(emits: &[JsxEmit], out: &mut Vec<JsxExpr>) {
    for emit in emits {
        match emit {
            JsxEmit::Expr(expr) | JsxEmit::ModelTarget(expr) => out.push(JsxExpr {
                content: expr.content.clone(),
                start: expr.start,
                end: expr.end,
            }),
            JsxEmit::ForScope {
                source,
                value_alias,
                key_alias,
                body,
            } => {
                out.push(JsxExpr {
                    content: source.content.clone(),
                    start: source.start,
                    end: source.end,
                });
                if let Some(value_alias) = value_alias {
                    out.push(JsxExpr {
                        content: value_alias.content.clone(),
                        start: value_alias.start,
                        end: value_alias.end,
                    });
                }
                if let Some(key_alias) = key_alias {
                    out.push(JsxExpr {
                        content: key_alias.content.clone(),
                        start: key_alias.start,
                        end: key_alias.end,
                    });
                }
                flatten_emits(body, out);
            }
            JsxEmit::Component(_) => {}
        }
    }
}

#[cfg(test)]
#[path = "virtual_ts_tests.rs"]
mod tests;
