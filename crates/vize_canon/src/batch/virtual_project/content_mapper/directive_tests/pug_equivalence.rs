//! A Pug template is projected through its derived Vue template, so it must
//! get the same echo table as the HTML template it lowers to. Without it the
//! per-prop assignment and the props literal each reported one authored
//! expression error (#13284).

use std::path::Path;

use vize_l0::String as CompactString;

use super::super::protocol::DIRECTIVE_POLICY_IGNORE;
use super::super::{
    ContentMapperTransform, ContentMapperTransformOptions,
    generate_vue_content_mapper_transform_with_options,
};
use crate::virtual_ts::VirtualTsCheckOptions;

const SCRIPT: &str = r#"<script setup lang="ts">
import Child from "./Child.vue";
import Gen from "./Gen.vue";
const model = { a: 1 };
const items = [{ id: 1 }];
const flag = Math.random() > 0.5;
const key = "title";
</script>
"#;

/// `(name, HTML template body, Pug template body)`.
const TEMPLATES: [(&str, &str, &str); 17] = [
    (
        "issue",
        "<Child :title=\"model.inComponent\" />\n<div :title=\"model.inElement\" />",
        "Child(:title=\"model.inComponent\")\ndiv(:title=\"model.inElement\")",
    ),
    (
        "v-bind",
        "<Child v-bind:title=\"model.missing\" />",
        "Child(v-bind:title=\"model.missing\")",
    ),
    ("same-name", "<Child :title />", "Child(:title)"),
    (
        "object",
        "<Child v-bind=\"model.missing\" />",
        "Child(v-bind=\"model.missing\")",
    ),
    (
        "kebab",
        "<Child title=\"t\" :long-title=\"model.missing\" />",
        "Child(title=\"t\" :long-title=\"model.missing\")",
    ),
    (
        "dynamic",
        "<Child title=\"t\" :[key]=\"model.missing\" />",
        "Child(title=\"t\" :[key]=\"model.missing\")",
    ),
    (
        "v-for",
        "<Child v-for=\"item in items\" :key=\"item.id\" :title=\"item.missing\" />",
        "Child(v-for=\"item in items\" :key=\"item.id\" :title=\"item.missing\")",
    ),
    (
        "v-if",
        "<div v-if=\"flag\" />\n<Child v-else :title=\"model.missing\" />",
        "div(v-if=\"flag\")\nChild(v-else :title=\"model.missing\")",
    ),
    (
        "v-model",
        "<Child title=\"t\" v-model:model=\"model.missing\" />",
        "Child(title=\"t\" v-model:model=\"model.missing\")",
    ),
    (
        "listener",
        "<Child title=\"t\" @pick=\"model.missing\" />",
        "Child(title=\"t\" @pick=\"model.missing\")",
    ),
    (
        "several",
        "<Child :title=\"model.x\" :other=\"model.y\" />",
        "Child(:title=\"model.x\" :other=\"model.y\")",
    ),
    (
        "slot-props",
        "<Child title=\"t\" v-slot=\"{ item }\">\n  <Child :title=\"item.missing\" />\n</Child>",
        "Child(title=\"t\" v-slot=\"{ item }\")\n  Child(:title=\"item.missing\")",
    ),
    (
        "slot-outlet",
        "<slot :item=\"model.missing\" />",
        "slot(:item=\"model.missing\")",
    ),
    (
        "generic-child",
        "<Gen :items=\"[{ id: 1 }]\" :selected=\"model.missing\" />",
        "Gen(:items=\"[{ id: 1 }]\" :selected=\"model.missing\")",
    ),
    (
        "class-style",
        "<Child title=\"t\" :class=\"model.missing\" :style=\"model.other\" />",
        "Child(title=\"t\" :class=\"model.missing\" :style=\"model.other\")",
    ),
    (
        "loose-library-keys",
        "<Child :items=\"[1, 2]\" :buffer=\"200\" />",
        "Child(:items=\"[1, 2]\" :buffer=\"200\")",
    ),
    (
        "nested-expression",
        "<Child :title=\"flag ? String(model.missing) : 'x'\" />",
        "Child(:title=\"flag ? String(model.missing) : 'x'\")",
    ),
];

fn html(body: &str) -> CompactString {
    vize_l0::cstr!("{SCRIPT}<template>\n{body}\n</template>\n")
}

fn pug(body: &str) -> CompactString {
    vize_l0::cstr!("{SCRIPT}<template lang=\"pug\">\n{body}\n</template>\n")
}

/// Generated names carry analyzed offsets; only their shape is compared.
fn without_digits(text: &str) -> CompactString {
    text.chars()
        .map(|character| {
            if character.is_ascii_digit() {
                '#'
            } else {
                character
            }
        })
        .collect()
}

fn transform(source: &str, check_unknown_props: bool) -> ContentMapperTransform {
    generate_vue_content_mapper_transform_with_options(
        Path::new("/project/App.vue"),
        source,
        ContentMapperTransformOptions {
            checks: VirtualTsCheckOptions {
                check_unknown_props,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap()
}

/// Each directive as authored owner text, generated text, policy and value.
fn table(
    source: &str,
    check_unknown_props: bool,
) -> Vec<(CompactString, CompactString, usize, usize)> {
    let result = transform(source, check_unknown_props);
    result
        .diagnostic_directives
        .iter()
        .flat_map(|table| &table.directives)
        .map(|directive| {
            let [original, length, start, end, policy, value] = directive.0;
            (
                source[original..original + length].into(),
                without_digits(&result.text[start..end]),
                policy,
                value,
            )
        })
        .collect()
}

#[test]
fn pug_templates_get_the_html_echo_table() {
    let mut selectors = 0;
    for check_unknown_props in [true, false] {
        for (name, html_body, pug_body) in TEMPLATES {
            let expected = table(&html(html_body), check_unknown_props);
            selectors += expected.iter().filter(|row| row.3 == 2353).count();
            assert_eq!(
                table(&pug(pug_body), check_unknown_props),
                expected,
                "{name}, check_unknown_props: {check_unknown_props}"
            );
        }
    }
    assert!(selectors > 0, "the loose-library selector is compared");
}

#[test]
fn the_reported_prop_assignment_is_an_echo_of_the_props_literal() {
    let (_, _, body) = TEMPLATES[0];
    let result = transform(&pug(body), true);
    let text = result.text.as_str();
    let after = |anchor: &str| {
        let anchor = text
            .find(anchor)
            .unwrap_or_else(|| panic!("missing {anchor}:\n{text}"));
        anchor + text[anchor..].find("model.inComponent").unwrap()
    };
    let ignored = |at: usize| {
        result
            .diagnostic_directives
            .iter()
            .flat_map(|table| &table.directives)
            .any(|directive| {
                directive.0[4] == DIRECTIVE_POLICY_IGNORE
                    && directive.0[2] <= at
                    && at < directive.0[3]
            })
    };
    assert!(ignored(after("const __vize_prop_check_0_title")), "{text}");
    assert!(!ignored(after("\"title\": model.inComponent")), "{text}");
}

#[test]
fn script_after_a_pug_template_keeps_authored_generic_owners() {
    let generic = "<script setup lang=\"ts\" generic=\"T extends string, U extends T = T\">\nimport Child from \"./Child.vue\";\ndefineProps<{ value: T; other: U }>();\n</script>\n";
    let html_source = vize_l0::cstr!(
        "<template>\n<Child :title=\"value\" :other=\"other\" />\n</template>\n{generic}"
    );
    let pug_source = vize_l0::cstr!(
        "<template lang=\"pug\">\nChild(:title=\"value\" :other=\"other\")\n</template>\n{generic}"
    );
    for check_unknown_props in [true, false] {
        let expected = table(&html_source, check_unknown_props);
        assert!(
            expected
                .iter()
                .any(|(owner, _, _, _)| owner == "T extends string, U extends T = T"),
            "{expected:?}"
        );
        assert_eq!(table(&pug_source, check_unknown_props), expected);
    }
}
