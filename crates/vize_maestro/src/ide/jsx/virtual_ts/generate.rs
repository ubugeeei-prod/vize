//! Native/test-only JSX virtual TypeScript generation.

use vize_atelier_jsx::{JsxLang, lower_source};
use vize_canon::virtual_ts::VizeMapping;
use vize_s0::Allocator;

use super::{
    JsxEmit, collect_root_expressions, collect_style_expressions, component, push_mapped_expr, slot,
};

const JSX_EXPR_SINK: &str = "__vize_jsx_expr__";

/// Ambient `Ctx<Emits, Slots>` injected at module scope. This stays byte-for-byte
/// aligned with the Canon batch generator so CLI and editor diagnostics agree.
const CTX_HELPER: &str = "type __EmitShape<T> = T extends (...args: any[]) => any ? T : T extends Record<string, any> ? { [K in keyof T]: T[K] extends (...args: infer A) => any ? A : T[K] extends any[] ? T[K] : any[]; } : Record<string, any[]>;\n\
type __EmitArgs<T, K extends keyof T> = T[K] extends any[] ? T[K] : any[];\n\
type __EmitFn<T> = __EmitShape<T> extends (...args: any[]) => any ? __EmitShape<T> : (<K extends keyof __EmitShape<T>>(event: K, ...args: __EmitArgs<__EmitShape<T>, K>) => void);\n\
type Ctx<Emits = {}, Slots = {}> = { emit: __EmitFn<Emits>; slots: Slots; attrs: Record<string, unknown>; };\n";

/// The generated plain-`.ts` virtual document for one `.jsx`/`.tsx` source.
pub(in crate::ide) struct JsxVirtualTs {
    pub(in crate::ide) code: String,
    pub(in crate::ide) mappings: Vec<VizeMapping>,
}

/// Lower a `.jsx`/`.tsx` Vize component to plain virtual TypeScript.
pub(in crate::ide) fn generate_jsx_virtual_ts(source: &str, lang: JsxLang) -> Option<JsxVirtualTs> {
    let allocator = Allocator::new();
    let lowered = lower_source(&allocator, allocator.as_oxc(), source, lang);

    let mut roots: Vec<(u32, u32, Vec<JsxEmit>)> = Vec::with_capacity(lowered.roots.len());
    for root in &lowered.roots {
        let mut emits = Vec::new();
        collect_root_expressions(&root.root, &mut emits, true);
        collect_style_expressions(&root.scoped_style_exprs, &mut emits);
        roots.push((root.root.loc.span.start, root.root.loc.span.end, emits));
    }
    roots.sort_by_key(|(start, _, _)| *start);

    let (code, mappings) = render_plain_ts(source, &roots);
    Some(JsxVirtualTs { code, mappings })
}

fn render_plain_ts(source: &str, roots: &[(u32, u32, Vec<JsxEmit>)]) -> (String, Vec<VizeMapping>) {
    let mut out = String::new();
    let mut mappings = Vec::new();

    out.push_str("declare function ");
    out.push_str(JSX_EXPR_SINK);
    out.push_str("(...args: unknown[]): any;\n");
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
            continue;
        }
        push_verbatim(&mut out, &mut mappings, source, cursor, start);
        render_sink_call(&mut out, &mut mappings, emits);
        cursor = end.max(start);
    }
    push_verbatim(&mut out, &mut mappings, source, cursor, source.len());

    (out, mappings)
}

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
        JsxEmit::SlotScope(scope) => {
            // `render_open` leaves the helper call open; the trailing `)` below
            // closes it around the body sink call.
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
