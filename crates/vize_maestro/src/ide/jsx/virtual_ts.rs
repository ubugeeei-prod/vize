//! Canon owns JSX/TSX virtual TypeScript and source mappings.
//!
//! This module retains the structural expression walk for semantic tokens.
//! Native editor requests consume the same Canon project as the CLI; stateless
//! projection tests also call the shared Canon generator.

use vize_atelier_jsx::{JsxLang, lower_source};
use vize_s0::Allocator;

mod collect;
#[cfg(any(test, feature = "native"))]
mod generate;
#[cfg(any(test, feature = "native"))]
pub(in crate::ide) use generate::JsxVirtualTs;
#[cfg(test)]
pub(in crate::ide) use generate::generate_jsx_virtual_ts;

use collect::{collect_root_expressions, collect_style_expressions};

/// A dynamic JSX expression recovered from the lowered tree: its original
/// source text plus the byte range it occupied in the `.jsx`/`.tsx` source.
#[derive(Clone)]
pub(in crate::ide) struct JsxExpr {
    pub(in crate::ide) content: String,
    pub(in crate::ide) start: u32,
    pub(in crate::ide) end: u32,
}

/// Structural expression ranges grouped by authored loop scope.
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
/// Uses the JSX compiler's lowered syntax tree and keeps binding-pattern
/// ranges for semantic tokens. Canon separately owns type-check projection.
pub(in crate::ide) fn collect_jsx_expressions(source: &str, lang: JsxLang) -> Vec<JsxExpr> {
    let allocator = Allocator::new();
    let lowered = lower_source(&allocator, allocator.as_oxc(), source, lang);
    let mut exprs = Vec::new();
    for root in &lowered.roots {
        let mut emits = Vec::new();
        collect_root_expressions(&root.root, &mut emits);
        collect_style_expressions(&root.scoped_style_exprs, &mut emits);
        flatten_emits(&emits, &mut exprs);
    }
    exprs.sort_by_key(|expr| expr.start);
    exprs
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

// `insta`'s snapshot macros expand through the disallowed `std::format!`; the
// expansion is inside `insta`, so only an allow at the test module can silence
// it. See CONTRIBUTING.md, "Snapshot assertions in test targets".
#[expect(
    clippy::disallowed_macros,
    reason = "insta snapshot assertions expand through std::format!; see CONTRIBUTING.md"
)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
#[cfg(test)]
#[path = "virtual_ts_tests.rs"]
mod tests;
