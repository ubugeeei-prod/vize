use std::ops::Range;

use vize_s0::cstr;

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

fn mapping_summary<'a>(source: &'a str, generated: &'a JsxVirtualTs) -> Vec<MappingSummary<'a>> {
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
fn component_tags_props_and_spreads_survive_editor_lowering() {
    let source = "import Counter from './Counter.vue';\nconst props = { count: 'wrong' };\nexport const view = <Counter {...props} is-opened />;\n";
    let generated = generate(source);

    assert!(generated.code.contains(component::HELPER));
    assert!(generated.code.contains(
        "__vize_jsx_component__(Counter, {...__vize_jsx_component_spread__(props), \"isOpened\": true})"
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
        "export const view = __vize_jsx_expr__(__vize_jsx_component__(Widget, {\"fooBar\": \"ok\"}), __vize_jsx_component_slot__(Widget, \"default\", (props) => __vize_jsx_expr__(props.item)));"
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
            "props",
            "props.item",
            ";\n",
        ]
    );
}

/// The structural walk (semantic tokens, hover) must still see the slot pattern
/// and every body expression once the generator wraps them in a scope.
#[test]
fn collect_jsx_expressions_includes_scoped_slot_pattern_and_body() {
    let source = "import Widget from './Widget.vue';\nexport const view = <Widget fooBar=\"ok\">{{ default: (props: { item: string }) => props.item }}</Widget>;\n";
    let contents: Vec<String> = collect_jsx_expressions(source, JsxLang::Tsx)
        .into_iter()
        .map(|expr| expr.content)
        .collect();

    assert_eq!(
        contents,
        vec!["props".to_string(), "props.item".to_string()]
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
