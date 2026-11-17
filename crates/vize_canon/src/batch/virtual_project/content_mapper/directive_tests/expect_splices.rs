//! #6258: an expectation's interval holds the listeners of its native owner
//! and the replayed checks of its slot-loop component, and no sibling's.

use super::super::protocol::DIRECTIVE_POLICY_EXPECT;
use super::transform;

const SCRIPT: &str = "<script setup lang=\"ts\">\nimport { ref } from 'vue'\nimport Frame from './Frame.vue'\nimport Item from './Item.vue'\nconst items = ref([])\nconst memo = 'x'\nconst onUpdate = (value: string) => console.log(value)\n</script>\n";

/// Which generated `needles` start inside the expectation's interval.
fn covered(template: &str, needles: &[&str]) -> Vec<bool> {
    let source = format!("{SCRIPT}<template>\n{template}\n</template>\n");
    let result = transform(&source);
    let directives = result.diagnostic_directives.unwrap();
    let [_, _, start, end, _, _] = directives
        .directives
        .iter()
        .find(|row| row.0[4] == DIRECTIVE_POLICY_EXPECT)
        .unwrap()
        .0;
    let text = result.text.as_str();
    needles
        .iter()
        .map(|needle| {
            let at = text.find(needle).unwrap();
            start <= at && at < end
        })
        .collect()
}

#[test]
fn native_listener_follows_its_bindings_before_a_bound_sibling() {
    let owner = "<input :value=\"memo\" @change=\"(event) => onUpdate(event?.target?.value)\" />";
    let needles = [
        "$ElementView<\"input\">, \"value\"> = (",
        "(event) => onUpdate(event?.target?.value)",
        "$ElementView<\"p\">, \"title\"> = (",
    ];
    for template in [
        format!(
            "  <div>\n    <!-- @vue-expect-error -->{owner}\n    <p :title=\"memo\" />\n  </div>"
        ),
        format!(
            "  <ul><li v-for=\"row in items\" :key=\"row\">\n    <!-- @vue-expect-error -->{owner}\n    <p :title=\"row\" />\n  </li></ul>"
        ),
    ] {
        assert_eq!(
            covered(&template, &needles),
            [true, true, false],
            "{template}"
        );
    }
}

#[test]
fn slot_loop_component_replay_follows_its_bindings() {
    let template = "  <Frame><template #menu>\n    <!-- @vue-expect-error --><Item v-for=\"item in items\" :key=\"item.key\" name=\"a\" />\n    <Item :name=\"memo\" />\n  </template></Frame>\n  <p :title=\"memo\" />";
    let needles = [
        "items);\n    for (const [item] of",
        "void (item.key)",
        "\"name\": \"a\"",
        "\"name\": memo,",
        "as unknown as __Frame_Check_2)(",
        "$ElementView<\"p\">, \"title\"> = (",
    ];
    assert_eq!(
        covered(template, &needles),
        [true, true, true, false, false, false]
    );
}

#[test]
fn a_render_time_call_keeps_the_listener_after_the_sibling() {
    // A call could be an assertion that narrows what the listener captures.
    let template = "  <div>\n    <!-- @vue-expect-error --><input :value=\"memo\" @change=\"(event) => onUpdate(event?.target?.value)\" />\n    <p :title=\"String(memo)\" />\n  </div>";
    let source = format!("{SCRIPT}<template>\n{template}\n</template>\n");
    let text = transform(&source).text;
    let sibling = text.find("$ElementView<\"p\">, \"title\">").unwrap();
    let listener = text.find("// @change handler").unwrap();
    assert_eq!(sibling < listener, true);
}
