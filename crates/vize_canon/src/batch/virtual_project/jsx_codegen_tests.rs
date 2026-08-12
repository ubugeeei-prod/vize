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

/// The rewritten render statement, i.e. the generated line that replaced the
/// authored JSX root. Asserting this exactly (rather than a substring) keeps the
/// emitted shape pinned, helper preamble aside.
fn rendered_statement(source: &str) -> String {
    let code = generate(source).code;
    code.lines()
        .find(|line| line.contains(JSX_EXPR_SINK) && !line.starts_with("declare function"))
        .unwrap_or_else(|| panic!("no rendered statement in:\n{code}"))
        .to_string()
}

/// A scoped slot binds its parameter pattern over the slot body. Re-emitting the
/// pattern through the ordinary directive walk made it a bare read of an
/// undeclared name and evaluated the body outside that scope (#4042).
#[test]
fn scoped_slot_object_binds_its_pattern_over_the_slot_body() {
    let source = "import Widget from \"./Widget.vue\";\nexport const view = <Widget fooBar=\"ok\">{{ default: (props: { item: string }) => props.item }}</Widget>;\n";

    assert_eq!(
        rendered_statement(source),
        "export const view = __vize_jsx_expr__(__vize_jsx_component__(Widget, {\"fooBar\": \"ok\"}), __vize_jsx_component_slot__(Widget, \"default\", (props) => __vize_jsx_expr__(props.item)));"
    );
}

#[test]
fn scoped_slot_render_prop_child_binds_its_pattern_over_the_slot_body() {
    let source = "import Widget from \"./Widget.vue\";\nexport const view = <Widget fooBar=\"ok\">{(props: { item: string }) => props.item}</Widget>;\n";

    assert_eq!(
        rendered_statement(source),
        "export const view = __vize_jsx_expr__(__vize_jsx_component__(Widget, {\"fooBar\": \"ok\"}), __vize_jsx_component_slot__(Widget, \"default\", (props) => __vize_jsx_expr__(props.item)));"
    );
}

#[test]
fn scoped_slot_destructured_pattern_and_named_slots_each_get_their_own_scope() {
    let source = "import Widget from \"./Widget.vue\";\nexport const view = <Widget fooBar=\"ok\">{{ default: ({ item }: { item: string }) => item, footer: (b: { n: number }) => b.n }}</Widget>;\n";

    assert_eq!(
        rendered_statement(source),
        "export const view = __vize_jsx_expr__(__vize_jsx_component__(Widget, {\"fooBar\": \"ok\"}), __vize_jsx_component_slot__(Widget, \"default\", ({ item }) => __vize_jsx_expr__(item)), __vize_jsx_component_slot__(Widget, \"footer\", (b) => __vize_jsx_expr__(b.n)));"
    );
}

/// A component rendered *inside* a slot body keeps its props contract, so an
/// invalid prop bound from the slot payload is still checked (#4042): before the
/// scope existed the payload read resolved to an error type and masked it.
#[test]
fn component_inside_a_scoped_slot_body_keeps_its_props_call() {
    let source = "import Widget from \"./Widget.vue\";\nimport Counter from \"./Counter.vue\";\nexport const view = <Widget fooBar=\"ok\">{{ default: (props: { item: string }) => <Counter count={props.item} /> }}</Widget>;\n";

    assert_eq!(
        rendered_statement(source),
        "export const view = __vize_jsx_expr__(__vize_jsx_component__(Widget, {\"fooBar\": \"ok\"}), __vize_jsx_component_slot__(Widget, \"default\", (props) => __vize_jsx_expr__(__vize_jsx_component__(Counter, {\"count\": props.item}))));"
    );
}

/// A scoped slot nested in a structural scope keeps both scopes: the `v-for`
/// aliases bind over the loop body and the slot pattern binds over the slot body
/// inside it.
#[test]
fn scoped_slot_inside_a_v_for_body_binds_both_scopes() {
    let source = "import Widget from \"./Widget.vue\";\nconst items: string[] = [];\nexport const view = <ul>{items.map((item) => <Widget fooBar={item}>{{ default: (props: { item: string }) => props.item }}</Widget>)}</ul>;\n";

    assert_eq!(
        rendered_statement(source),
        "export const view = __vize_jsx_expr__((items).map((item) => __vize_jsx_expr__(__vize_jsx_component__(Widget, {\"fooBar\": item}), __vize_jsx_component_slot__(Widget, \"default\", (props) => __vize_jsx_expr__(props.item)))));"
    );
}

/// Nested scoped slots each resolve *their own* host, so the inner slot's
/// parameter is typed from the inner component's `$slots`, not the outer one's.
#[test]
fn nested_scoped_slots_each_resolve_their_own_host() {
    let source = "import Widget from \"./Widget.vue\";\nimport Panel from \"./Panel.vue\";\nexport const view = <Widget fooBar=\"ok\">{{ default: (outer: { item: string }) => <Panel title={outer.item}>{{ default: (inner: { row: number }) => inner.row }}</Panel> }}</Widget>;\n";

    assert_eq!(
        rendered_statement(source),
        "export const view = __vize_jsx_expr__(__vize_jsx_component__(Widget, {\"fooBar\": \"ok\"}), __vize_jsx_component_slot__(Widget, \"default\", (outer) => __vize_jsx_expr__(__vize_jsx_component__(Panel, {\"title\": outer.item}), __vize_jsx_component_slot__(Panel, \"default\", (inner) => __vize_jsx_expr__(inner.row)))));"
    );
}

/// A slot with no binding pattern introduces no scope, so its body stays in the
/// enclosing sink exactly as before.
#[test]
fn non_scoped_slot_body_stays_in_the_enclosing_scope() {
    let source = "import Widget from \"./Widget.vue\";\nconst label = 1;\nexport const view = <Widget fooBar=\"ok\">{{ default: () => label }}</Widget>;\n";

    assert_eq!(
        rendered_statement(source),
        "export const view = __vize_jsx_expr__(__vize_jsx_component__(Widget, {\"fooBar\": \"ok\"}), label);"
    );
}

/// The scoped-slot pattern and every body expression stay mapped to their
/// authored ranges, so diagnostics land on the JSX the user wrote.
#[test]
fn scoped_slot_maps_its_pattern_and_body_to_authored_ranges() {
    let source = "import Widget from \"./Widget.vue\";\nexport const view = <Widget fooBar=\"ok\">{{ default: (props: { item: string }) => props.item }}</Widget>;\n";
    let generated = generate(source);

    let mapped: Vec<&str> = generated
        .mappings
        .iter()
        .map(|mapping| &source[mapping.src_range.clone()])
        .collect();
    assert_eq!(
        mapped,
        vec![
            // Verbatim prefix, then the component call: tag, prop value, whole
            // attribute, and the props object literal (mapped to the tag).
            "import Widget from \"./Widget.vue\";\nexport const view = ",
            "Widget",
            "\"ok\"",
            "fooBar=\"ok\"",
            "Widget",
            // The slot scope maps only the authored binding pattern; the
            // re-emitted host tag and the slot-name literal are scaffolding and
            // stay unmapped so a diagnostic on the tag cannot double-report.
            "props",
            "props.item",
            ";\n",
        ]
    );
}

/// The slot helper must be declared alongside the component helper, since a
/// scoped slot is only ever emitted under a component host.
#[test]
fn component_helper_declares_the_slot_payload_contract() {
    let helper = crate::virtual_ts::JSX_COMPONENT_HELPER;
    let declarations: Vec<&str> = helper
        .lines()
        .filter(|line| line.starts_with("declare function"))
        .collect();

    assert_eq!(
        declarations,
        vec![
            "declare function __vize_jsx_component_spread__<O>(value: O): __VizeJsxCanonicalRawProps<Omit<O, 'key' | 'ref'>>;",
            "declare function __vize_jsx_component__<C>(component: C, props: __VizeJsxComponentProps<C>): any;",
            "declare function __vize_jsx_component_slot__<C, N extends string>(component: C, name: N, render: (payload: __VizeJsxSlotPayload<C, N>) => unknown): any;",
        ]
    );
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
