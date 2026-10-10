//! Plain TypeScript rendering and authored JSX source mappings.

use super::{CTX_HELPER, JSX_EXPR_SINK, JsxEmit, JsxExpr, component, slot};
use crate::virtual_ts::VizeMapping;
use vize_carton::String as CompactString;

/// Build the plain-`.ts` text and its source mappings.
///
/// Every byte outside a JSX render root is copied verbatim; each render root is
/// replaced by `__vize_jsx_expr__(<unit>, <unit>, …)`, with each re-emitted
/// expression mapped back to its original byte range.
pub(super) fn render_plain_ts(
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
        JsxEmit::Component(component) => component::render(out, mappings, component),
        JsxEmit::SlotScope(scope) => {
            // `__vize_jsx_component_slot__(<Host>, "<name>", (<pattern>) =>
            //  __vize_jsx_expr__(<body…>))`: the body is re-emitted inside the
            // callback so the slot pattern binds with the payload type declared
            // by the host component's `$slots`. `render_open` leaves the helper
            // call open; the trailing `)` below closes it.
            slot::render_open(out, mappings, scope);
            render_sink_call(out, mappings, scope.body());
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

fn emit_contains_component(emit: &JsxEmit) -> bool {
    match emit {
        // A slot scope is only ever produced under a component host, and its
        // opening call needs the same helper block.
        JsxEmit::Component(_) | JsxEmit::SlotScope(_) => true,
        JsxEmit::ForScope { body, .. } => body.iter().any(emit_contains_component),
        JsxEmit::Expr(_) | JsxEmit::ModelTarget(_) => false,
    }
}

/// Copy a re-emitted expression's text into `out` and record the mapping from
/// its generated range back to its original `.jsx`/`.tsx` byte range.
pub(super) fn push_mapped_expr(
    out: &mut CompactString,
    mappings: &mut Vec<VizeMapping>,
    expr: &JsxExpr,
) {
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
    out.push_str(source.get(src_start..src_end).unwrap_or_default());
    let gen_end = out.len();
    mappings.push(VizeMapping {
        gen_range: gen_start..gen_end,
        src_range: src_start..src_end,
        sub_spans: Vec::new(),
    });
}
