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
use vize_s0::Allocator;

mod collect;
mod component;
#[cfg(any(test, feature = "native"))]
mod generate;
mod slot;
#[cfg(any(test, feature = "native"))]
pub(in crate::ide) use generate::{JsxVirtualTs, generate_jsx_virtual_ts};

use collect::{collect_root_expressions, collect_style_expressions, expr_of};

/// A dynamic JSX expression recovered from the lowered tree: its original
/// source text plus the byte range it occupied in the `.jsx`/`.tsx` source.
#[derive(Clone)]
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
    /// Only the `native`/test generator renders this variant; the structural
    /// walk keeps it so both builds share one collector.
    #[cfg_attr(not(any(test, feature = "native")), allow(dead_code))]
    Component(component::JsxComponent),
    /// A scoped-slot scope: the slot's binding pattern plus the body units
    /// evaluated with that pattern in scope, typed from the host component's
    /// declared `$slots`.
    #[cfg_attr(not(any(test, feature = "native")), allow(dead_code))]
    SlotScope(slot::JsxSlotScope),
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
    let allocator = Allocator::new();
    let lowered = lower_source(&allocator, allocator.as_oxc(), source, lang);
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

/// Append `expr`'s source text to `out` and record the mapping back to the
/// byte range it occupied in the original `.jsx`/`.tsx` source.
///
/// Shared by the generator and the semantic component renderer so both emit
/// identical mappings.
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
            // A scoped slot contributes its binding pattern and its body, so
            // the structural walk sees the same expressions the generator
            // re-emits inside the slot callback.
            JsxEmit::SlotScope(scope) => {
                let params = scope.params();
                out.push(JsxExpr {
                    content: params.content.clone(),
                    start: params.start,
                    end: params.end,
                });
                flatten_emits(scope.body(), out);
            }
            JsxEmit::Component(_) => {}
        }
    }
}

// `insta`'s snapshot macros expand through the disallowed `std::format!`; the
// expansion is inside `insta`, so only an allow at the test module can silence
// it. See CONTRIBUTING.md, "Snapshot assertions in test targets".
#[allow(clippy::disallowed_macros)]
#[cfg(test)]
#[path = "virtual_ts_tests.rs"]
mod tests;
