//! Canon owns JSX/TSX virtual TypeScript and source mappings.
//!
//! This module retains the structural expression walk for semantic tokens.
//! Native editor requests consume the same Canon project as the CLI; stateless
//! projection tests also call the shared Canon generator.

use vize_atelier_jsx::{JsxLang, lower_source};
use vize_l0::Allocator;

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
mod tests {
    use std::ops::Range;

    use vize_l0::cstr;

    use super::*;
    use crate::ide::jsx::position::source_offset_to_virtual_position;

    fn generate(source: &str) -> JsxVirtualTs {
        generate_jsx_virtual_ts(source, JsxLang::Tsx).unwrap()
    }

    fn assert_virtual_ts_snapshot(name: &str, source: &str) {
        let generated = generate(source);
        insta::assert_snapshot!(cstr!("{name}_code").as_str(), generated.code.as_str());
        insta::assert_debug_snapshot!(
            cstr!("{name}_mappings").as_str(),
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

    #[expect(
        dead_code,
        reason = "fields are only read through the `Debug` output that the assertions compare"
    )]
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
                let position = source_offset_to_virtual_position(generated, source_offset)
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

    #[expect(
        dead_code,
        reason = "fields are only read through the `Debug` output that the assertions compare"
    )]
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
    fn component_tags_props_and_spreads_survive_editor_lowering() {
        let source = "import Counter from './Counter.vue';\nconst props = { count: 'wrong' };\nexport const view = <Counter {...props} is-opened />;\n";
        let generated = generate(source);

        assert!(
            generated
                .code
                .contains(vize_canon::virtual_ts::JSX_COMPONENT_HELPER)
        );
        assert!(generated.code.contains(
        "__vize_jsx_component__(Counter)({...__vize_jsx_component_spread__(props), \"isOpened\": true})"
    ));
        for authored in ["Counter", "props", "is-opened"] {
            assert!(generated.mappings.iter().any(|mapping| {
                &source[mapping.src_range.clone()] == authored
                    || mapping
                        .sub_spans
                        .iter()
                        .any(|span| &source[span.src_range.clone()] == authored)
            }));
        }
    }

    /// The editor generator must produce the same scoped-slot scope the Canon batch
    /// generator does, byte for byte, or a diagnostic would land at a different
    /// range in the editor than on the CLI (#4042). The expected text below is the
    /// exact string asserted by
    /// `vize_canon::batch::virtual_project::jsx_codegen::tests`.
    #[test]
    fn scoped_slot_lowering_matches_the_batch_generator() {
        let source = "import Widget from './Widget.vue';\nexport const view = <Widget fooBar=\"ok\">{{ default: (props: { item: string }) => props.item }}</Widget>;\n";
        let generated = generate(source);

        let rendered = generated
            .code
            .lines()
            .find(|line| line.starts_with("export const view = "))
            .expect("the render root must be rewritten");
        assert_eq!(
            rendered,
            "export const view = __vize_jsx_expr__(__vize_jsx_component__(Widget)({\"fooBar\": \"ok\"}), __vize_jsx_component_slot__(Widget, \"default\", (props: { item: string }) => __vize_jsx_expr__(props.item)));"
        );

        let mapped: Vec<&str> = generated
            .mappings
            .iter()
            .map(|mapping| &source[mapping.src_range.clone()])
            .collect();
        assert_eq!(
            mapped,
            vec![
                "import Widget from './Widget.vue';\nexport const view = ",
                "Widget",
                "\"ok\"",
                "fooBar=\"ok\"",
                "Widget",
                "props: { item: string }",
                "(",
                "props.item",
                ";\n",
            ]
        );
    }

    /// The structural walk (semantic tokens, hover) sees the imported component,
    /// slot pattern and body expression once, at their authored source spans.
    #[test]
    fn collect_jsx_expressions_includes_scoped_slot_pattern_and_body() {
        let source = "import Widget from './Widget.vue';\nexport const view = <Widget fooBar=\"ok\">{{ default: (props: { item: string }) => props.item }}</Widget>;\n";
        let expressions: Vec<(String, usize, usize)> =
            collect_jsx_expressions(source, JsxLang::Tsx)
                .into_iter()
                .map(|expr| (expr.content, expr.start as usize, expr.end as usize))
                .collect();
        let tag = source.find("<Widget").expect("opening component tag") + 1;
        let pattern = source.find("(props:").expect("slot parameter") + 1;
        let body = source.find("=> props.item").expect("slot body") + 3;

        assert_eq!(
            expressions,
            vec![
                ("Widget".to_string(), tag, tag + "Widget".len()),
                ("props".to_string(), pattern, pattern + "props".len()),
                ("props.item".to_string(), body, body + "props.item".len()),
            ]
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

    #[expect(
        dead_code,
        reason = "fields are only read through the `Debug` output that the assertions compare"
    )]
    #[derive(Debug)]
    struct ExprSummary {
        content: String,
        source: String,
        start: u32,
        end: u32,
    }
}
