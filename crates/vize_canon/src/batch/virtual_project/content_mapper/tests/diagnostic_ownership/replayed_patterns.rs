//! Each authored slot and `v-for` pattern reaches the host once, whether or not
//! the component-props pass replays its scope (#12479, #12520).

use serde_json::Value;

const SCRIPT: &str = "<script setup lang=\"ts\">\nimport Child from \"./Child.vue\";\nimport Other from \"./Other.vue\";\nconst items = [{ id: 1, name: \"a\" }];\nconst pairs: [number, string][] = [[1, \"a\"]];\n</script>\n\n";

/// Authored offsets of the generated `needle` copies the host reports: those
/// inside a span and outside every ignore directive, mapped through the
/// smallest span. Unmapped copies are ignored like every unmapped range.
fn visible(source: &str, needle: &str) -> Vec<usize> {
    let packet = super::packet("src/P3.vue", source, true);
    let text = packet["text"].as_str().expect("virtual text");
    let rows = |value: &Value| -> Vec<Vec<usize>> {
        serde_json::from_value(value.clone()).unwrap_or_default()
    };
    let spans = rows(&packet["mappings"]);
    let ignores = rows(&packet["diagnosticDirectives"]["directives"]);
    text.match_indices(needle)
        .map(|(generated, _)| generated)
        .filter(|generated| {
            !ignores
                .iter()
                .any(|row| row[4] == 0 && (row[2]..row[3]).contains(generated))
        })
        .filter_map(|generated| {
            spans
                .iter()
                .filter(|row| (row[0]..row[0] + row[1]).contains(&generated))
                .min_by_key(|row| row[1])
                .map(|row| row[2] + generated - row[0])
        })
        .collect()
}

/// Authored offsets of the visible `needle` copies in `template`, and the
/// authored offset of the `nth` `needle` in it.
fn copies(template: &str, needle: &str) -> (Vec<usize>, impl Fn(usize) -> usize) {
    let source = format!("{SCRIPT}<template>\n  {template}\n</template>\n");
    let visible = visible(&source, needle);
    let authored = source
        .match_indices(needle)
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    (visible, move |nth| authored[nth])
}

#[test]
fn slot_pattern_bindings_and_body_reads_are_visible_once() {
    for (template, needle) in [
        (
            "<Child><template #default=\"{ missing }\">{{ missing }}</template></Child>",
            "missing",
        ),
        (
            "<Child><template #default=\"{ value }\">{{ value.toFixed() }}</template></Child>",
            "value",
        ),
        (
            "<Child><template #default=\"{ value = 'd', item: { id, missing = 1 } }\">{{ missing }}</template></Child>",
            "missing",
        ),
        (
            "<Child v-slot=\"{ missing }\">{{ missing }}</Child>",
            "missing",
        ),
    ] {
        let (visible, authored) = copies(template, needle);
        assert_eq!(visible, [authored(0), authored(1)], "{template}");
    }
}

#[test]
fn a_slot_replayed_for_component_checks_keeps_one_visible_pattern() {
    let (visible, authored) = copies(
        "<Child v-slot=\"{ value, missing }\"><Other :num=\"value\" :text=\"missing\" /></Child>",
        "missing",
    );
    let pattern = authored(0);
    assert_eq!(
        visible.iter().filter(|offset| **offset == pattern).count(),
        1,
        "{visible:?}"
    );
}

#[test]
fn replayed_slots_keep_unique_helper_names() {
    let source = format!(
        "{SCRIPT}<template>\n  <Child #default=\"{{ value }}\"><Other :num=\"value\" /></Child>\n  <Child #default=\"{{ item }}\"><Other :num=\"item.id\" /></Child>\n</template>\n"
    );
    let packet = super::packet("src/P3.vue", &source, true);
    let text = packet["text"].as_str().expect("virtual text");
    let mut helpers = text
        .match_indices("void function _slot_props_default_")
        .map(|(start, needle)| {
            let rest = &text[start + needle.len()..];
            &rest[..rest.find('(').expect("helper parameters")]
        })
        .collect::<Vec<_>>();
    assert_eq!(helpers.len(), 2, "{text}");
    helpers.dedup();
    assert_eq!(helpers.len(), 2, "{text}");
}

#[test]
fn v_for_pattern_bindings_are_visible_once_with_or_without_a_replay() {
    for template in [
        // No component: the props pass emits nothing.
        "<div v-for=\"{ nme } in items\">{{ nme }}</div>",
        // A component elsewhere: the loop is replayed with unmapped aliases.
        "<div v-for=\"{ nme } in items\">{{ nme }}</div><Other :num=\"1\" />",
        // A slot around the loop, neither checked.
        "<Child v-slot=\"{ value }\"><div v-for=\"{ nme } in items\">{{ value }}{{ nme }}</div></Child>",
    ] {
        let (visible, authored) = copies(template, "nme");
        assert_eq!(visible, [authored(0), authored(1)], "{template}");
    }
    let (visible, authored) = copies(
        "<div v-for=\"[first, extra] in pairs\">{{ extra }}</div><Other :num=\"1\" />",
        "extra",
    );
    assert_eq!(visible, [authored(0), authored(1)]);
}

#[test]
fn replayed_v_for_aliases_keep_one_visible_declaration() {
    for template in [
        "<template v-for=\"{ nme } in items\"><Other :text=\"nme\" :num=\"1\" /></template>",
        "<Child v-slot=\"{ value }\"><template v-for=\"{ nme } in items\"><Other :num=\"value\" :text=\"nme\" /></template></Child>",
    ] {
        let (visible, authored) = copies(template, "nme");
        let pattern = authored(0);
        assert_eq!(
            visible.iter().filter(|offset| **offset == pattern).count(),
            1,
            "{template}: {visible:?}"
        );
    }
}
