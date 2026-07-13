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

#[cfg(test)]
use vize_atelier_jsx::{JsxLang, snapshot_jsx};
use vize_atelier_jsx::{JsxSyntaxSnapshot, JsxTypecheckEmit, JsxTypecheckExpression};
use vize_canon::virtual_ts::VizeMapping;

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
#[cfg(test)]
pub(in crate::ide) fn collect_jsx_expressions(source: &str, lang: JsxLang) -> Vec<JsxExpr> {
    let snapshot = snapshot_jsx(source, lang);
    collect_snapshot_expressions(&snapshot)
}

pub(in crate::ide) fn collect_snapshot_expressions(snapshot: &JsxSyntaxSnapshot) -> Vec<JsxExpr> {
    let mut exprs = Vec::new();
    for root in snapshot.typecheck_roots() {
        let emits = root.emits.iter().map(project_emit).collect::<Vec<_>>();
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
#[cfg(test)]
pub(in crate::ide) fn generate_jsx_virtual_ts(source: &str, lang: JsxLang) -> Option<JsxVirtualTs> {
    let snapshot = snapshot_jsx(source, lang);
    generate_jsx_virtual_ts_from_snapshot(&snapshot)
}

pub(in crate::ide) fn generate_jsx_virtual_ts_from_snapshot(
    snapshot: &JsxSyntaxSnapshot,
) -> Option<JsxVirtualTs> {
    let mut roots = snapshot
        .typecheck_roots()
        .iter()
        .map(|root| {
            (
                root.span.start,
                root.span.end,
                root.emits.iter().map(project_emit).collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    // Outermost roots never overlap and are produced in source order, but guard
    // the rewrite against any accidental disorder.
    roots.sort_by_key(|(start, _, _)| *start);

    let (code, mappings) = render_plain_ts(snapshot.source.as_ref(), &roots);
    Some(JsxVirtualTs { code, mappings })
}

fn project_emit(emit: &JsxTypecheckEmit) -> JsxEmit {
    match emit {
        JsxTypecheckEmit::Expression(expression) => JsxEmit::Expr(project_expression(expression)),
        JsxTypecheckEmit::ModelTarget(expression) => {
            JsxEmit::ModelTarget(project_expression(expression))
        }
        JsxTypecheckEmit::ForScope {
            source,
            value,
            index,
            body,
        } => JsxEmit::ForScope {
            source: project_expression(source),
            value_alias: value.as_ref().map(project_expression),
            key_alias: index.as_ref().map(project_expression),
            body: body.iter().map(project_emit).collect(),
        },
    }
}

fn project_expression(expression: &JsxTypecheckExpression) -> JsxExpr {
    JsxExpr {
        content: expression.code.as_ref().to_string(),
        start: expression.span.start,
        end: expression.span.end,
    }
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range;

    use crate::ide::jsx::position::source_offset_to_virtual_position;

    fn generate(source: &str) -> JsxVirtualTs {
        generate_jsx_virtual_ts(source, JsxLang::Tsx).unwrap()
    }

    fn assert_virtual_ts_snapshot(name: &str, source: &str) {
        let generated = generate(source);
        insta::assert_snapshot!(format!("{name}_code"), generated.code.as_str());
        insta::assert_debug_snapshot!(
            format!("{name}_mappings"),
            mapping_summary(source, &generated)
        );
    }

    fn mapping_summary<'a>(
        source: &'a str,
        generated: &'a JsxVirtualTs,
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

    fn virtual_positions_for_markers(
        source: &str,
        generated: &JsxVirtualTs,
        markers: &[&str],
    ) -> Vec<VirtualPosition> {
        markers
            .iter()
            .map(|marker| {
                let source_offset = source
                    .match_indices(marker)
                    .map(|(offset, _)| offset)
                    .next()
                    .expect("marker present");
                let position = source_offset_to_virtual_position(
                    &generated.code,
                    &generated.mappings,
                    source_offset,
                )
                .expect("marker maps into virtual TS");

                VirtualPosition {
                    marker: (*marker).to_string(),
                    source_offset,
                    source: source[source_offset..source_offset + marker.len()].to_string(),
                    position,
                }
            })
            .collect()
    }

    #[allow(dead_code)]
    #[derive(Debug)]
    struct VirtualPosition {
        marker: String,
        source_offset: usize,
        source: String,
        position: (u32, u32),
    }

    #[test]
    fn typed_component_with_jsx_control_flow_directives_and_styles_is_exact() {
        let source = "import { computed, ref } from 'vue';\n\nconst Comp = (\n  { items, ok, tone, gap }: { items: Array<{ id: string; label: string }>; ok: boolean; tone: string; gap: number },\n  { emit, slots }: Ctx<{ select: [id: string] }, { footer: () => unknown }>,\n) => {\n  const selected = ref(items[0]?.id);\n  const activeItem = computed(() => items.find((item) => item.id === selected.value));\n  return (\n    <>\n      <ul class={tone} v-show={ok}>\n        {items.map((item, index) => (\n          <li key={item.id} onClick={() => emit('select', item.id)} data-index={index}>\n            {item.label}{selected.value === item.id ? <strong>Selected</strong> : <em>{index}</em>}\n          </li>\n        ))}\n      </ul>\n      <input v-model={selected.value} v-focus:lazy={tone} />\n      <footer>{activeItem.value?.label}{slots.footer()}</footer>\n      <style scoped>{`.row { gap: ${gap}px; }`}</style>\n    </>\n  );\n};\n";

        assert_virtual_ts_snapshot(
            "typed_component_with_jsx_control_flow_directives_and_styles",
            source,
        );
        let generated = generate(source);
        insta::assert_debug_snapshot!(
            "typed_component_with_jsx_control_flow_directives_and_styles_positions",
            virtual_positions_for_markers(
                source,
                &generated,
                &[
                    "items.map",
                    "tone} v-show",
                    "ok}>",
                    "item.id",
                    "emit('select', item.id)",
                    "item.label",
                    "selected.value === item.id",
                    "index",
                    "selected.value} v-focus",
                    "activeItem.value?.label",
                    "slots.footer()",
                    "gap}px",
                ],
            )
        );
    }

    #[test]
    fn multiple_roots_and_static_style_are_exact() {
        let source = "const First = (props: { msg: string }) => <section>{props.msg}</section>;\nconst Second = () => (\n  <>\n    <div class=\"box\" />\n    <style scoped>{`.box { color: red; }`}</style>\n  </>\n);\n";

        assert_virtual_ts_snapshot("multiple_roots_and_static_style", source);
    }

    #[test]
    fn jsx_file_mode_is_exact() {
        let source =
            "export const Plain = ({ msg }) => <button onClick={() => save(msg)}>{msg}</button>;\n";
        let generated = generate_jsx_virtual_ts(source, JsxLang::Jsx).unwrap();

        insta::assert_snapshot!("jsx_file_mode_code", generated.code.as_str());
        insta::assert_debug_snapshot!(
            "jsx_file_mode_mappings",
            mapping_summary(source, &generated)
        );
    }

    #[test]
    fn collect_jsx_expressions_includes_for_body_model_and_style_exprs() {
        let source = "const Comp = (props: { items: string[]; color: string }) => (\n  <>\n    {props.items.map((item) => <span>{item}</span>)}\n    <input v-model={props.color} />\n    <style scoped>{`.box { color: ${props.color}; }`}</style>\n  </>\n);\n";
        let exprs = collect_jsx_expressions(source, JsxLang::Tsx)
            .into_iter()
            .map(|expr| ExprSummary {
                content: expr.content,
                source: source[expr.start as usize..expr.end as usize].to_string(),
                start: expr.start,
                end: expr.end,
            })
            .collect::<Vec<_>>();

        insta::assert_debug_snapshot!(
            "collect_jsx_expressions_includes_for_body_model_and_style_exprs",
            exprs
        );
    }

    #[allow(dead_code)]
    #[derive(Debug)]
    struct ExprSummary {
        content: String,
        source: String,
        start: u32,
        end: u32,
    }
}
