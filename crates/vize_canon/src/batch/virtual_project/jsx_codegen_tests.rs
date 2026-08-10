use std::ops::Range;

use super::*;

fn generate(source: &str) -> GeneratedJsxFile {
    generate_jsx_virtual_ts(Path::new("Comp.tsx"), source, JsxLang::Tsx).unwrap()
}

fn assert_generated_snapshot(name: &str, source: &str) {
    let generated = generate(source);
    insta::assert_snapshot!(format!("{name}_code"), generated.code.as_str());
    insta::assert_debug_snapshot!(
        format!("{name}_mappings"),
        mapping_summary(source, &generated)
    );
    insta::assert_debug_snapshot!(format!("{name}_diagnostics"), generated.diagnostics);
}

fn mapping_summary<'a>(
    source: &'a str,
    generated: &'a GeneratedJsxFile,
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

#[test]
fn typed_component_with_jsx_control_flow_directives_and_styles_is_exact() {
    let source = "import { computed, ref } from 'vue';\n\nconst Comp = (\n  { items, ok, tone, gap }: { items: Array<{ id: string; label: string }>; ok: boolean; tone: string; gap: number },\n  { emit, slots }: Ctx<{ select: [id: string] }, { footer: () => unknown }>,\n) => {\n  const selected = ref(items[0]?.id);\n  const activeItem = computed(() => items.find((item) => item.id === selected.value));\n  return (\n    <>\n      <ul class={tone} v-show={ok}>\n        {items.map((item, index) => (\n          <li key={item.id} onClick={() => emit('select', item.id)} data-index={index}>\n            {item.label}{selected.value === item.id ? <strong>Selected</strong> : <em>{index}</em>}\n          </li>\n        ))}\n      </ul>\n      <input v-model={selected.value} v-focus:lazy={tone} />\n      <footer>{activeItem.value?.label}{slots.footer()}</footer>\n      <style scoped>{`.row { gap: ${gap}px; }`}</style>\n    </>\n  );\n};\n";

    assert_generated_snapshot(
        "typed_component_with_jsx_control_flow_directives_and_styles",
        source,
    );
}

#[test]
fn multiple_roots_and_static_style_are_exact() {
    let source = "const First = (props: { msg: string }) => <section>{props.msg}</section>;\nconst Second = () => (\n  <>\n    <div class=\"box\" />\n    <style scoped>{`.box { color: red; }`}</style>\n  </>\n);\n";

    assert_generated_snapshot("multiple_roots_and_static_style", source);
}

#[test]
fn jsx_file_mode_is_exact() {
    let source =
        "export const Plain = ({ msg }) => <button onClick={() => save(msg)}>{msg}</button>;\n";
    let generated = generate_jsx_virtual_ts(Path::new("Plain.jsx"), source, JsxLang::Jsx).unwrap();

    insta::assert_snapshot!("jsx_file_mode_code", generated.code.as_str());
    insta::assert_debug_snapshot!(
        "jsx_file_mode_mappings",
        mapping_summary(source, &generated)
    );
    insta::assert_debug_snapshot!("jsx_file_mode_diagnostics", generated.diagnostics);
}

#[test]
fn semantic_component_tags_props_and_spreads_survive_plain_ts_lowering() {
    let source = "import Counter from './Counter.vue';\nconst Library = { Counter };\nconst count = 'wrong';\nconst bag = { enabled: true };\nexport const first = <Counter count={count} is-opened />;\nexport const second = <Library.Counter {...bag} label=\"hello\" />;\n";
    let generated = generate(source);

    assert!(generated.code.contains(component::HELPER));
    assert!(
        generated
            .code
            .contains("__vize_jsx_component__(Counter, {\"count\": count, \"isOpened\": true})")
    );
    assert!(
        generated.code.contains(
            "__vize_jsx_component__(Library.Counter, {...__vize_jsx_component_spread__(bag), \"label\": \"hello\"})"
        )
    );

    for authored in [
        "Counter",
        "count",
        "is-opened",
        "Library.Counter",
        "bag",
        "label",
    ] {
        assert!(
            generated.mappings.iter().any(|mapping| {
                &source[mapping.src_range.clone()] == authored
                    || mapping
                        .sub_spans
                        .iter()
                        .any(|span| &source[span.src_range.clone()] == authored)
            }),
            "missing authored mapping for {authored}:\n{}",
            generated.code
        );
    }
}
