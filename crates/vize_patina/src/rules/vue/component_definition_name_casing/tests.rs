use super::{
    ComponentDefinitionNameCasing, ComponentDefinitionNameCasingOption, may_name_component,
};
use crate::linter::Linter;
use crate::rule::RuleRegistry;
use vize_l0::{String, cstr};

const RULE: &str = "vue/component-definition-name-casing";

fn reports(
    casing: ComponentDefinitionNameCasingOption,
    source: &str,
) -> Vec<(String, String, String)> {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ComponentDefinitionNameCasing { casing }));
    let result = Linter::with_registry(registry).lint_sfc(source, "Component.vue");
    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, result.diagnostics.len());
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            assert_eq!(diagnostic.rule_name, RULE);
            (
                String::from(
                    source
                        .get(diagnostic.start as usize..diagnostic.end as usize)
                        .unwrap(),
                ),
                diagnostic.message.clone(),
                String::from(diagnostic.help.as_deref().unwrap_or_default()),
            )
        })
        .collect()
}

fn pascal(source: &str) -> Vec<String> {
    reports(ComponentDefinitionNameCasingOption::PascalCase, source)
        .into_iter()
        .map(|(range, ..)| range)
        .collect()
}

fn kebab(source: &str) -> Vec<String> {
    reports(ComponentDefinitionNameCasingOption::KebabCase, source)
        .into_iter()
        .map(|(range, ..)| range)
        .collect()
}

#[test]
fn reports_each_definition_source_with_its_literal_range() {
    let cases = [
        (
            "<script>\nexport default { name: 'my_component' }\n</script>\n",
            "'my_component'",
        ),
        (
            "<script lang=\"ts\">\nimport { defineComponent } from 'vue'\nexport default defineComponent({ name: \"definition_casing\" })\n</script>\n",
            "\"definition_casing\"",
        ),
        (
            "<script setup lang=\"ts\">\ndefineOptions({ name: 'definition_casing_setup' })\n</script>\n",
            "'definition_casing_setup'",
        ),
        (
            "<script setup lang=\"ts\">\nimport { createApp } from 'vue'\nimport HelperChild from './HelperChild.vue'\nconst app = createApp({})\napp.component('helper_child', HelperChild)\n</script>\n",
            "'helper_child'",
        ),
        (
            "<script>\nVue.component(`my-component`, {})\n</script>\n",
            "`my-component`",
        ),
        (
            "<script>\nnew Vue({ name: 'root_app' })\n</script>\n",
            "'root_app'",
        ),
        (
            "<script>\nexport default defineNuxtComponent({ 'name': 'nuxt_box' })\n</script>\n",
            "'nuxt_box'",
        ),
    ];
    for (source, range) in cases {
        assert_eq!(pascal(source), [range], "{source}");
    }
}

#[test]
fn names_the_converted_name_in_the_message_help() {
    let source = "<script setup>\ndefineOptions({ name: 'definition_casing_setup' })\n</script>\n";
    assert_eq!(
        reports(ComponentDefinitionNameCasingOption::PascalCase, source),
        [(
            String::from("'definition_casing_setup'"),
            String::from("Component name 'definition_casing_setup' is not PascalCase"),
            String::from("Rename the component to 'DefinitionCasingSetup'."),
        )]
    );
    let source = "<script>\nexport default { name: 'MyComponent' }\n</script>\n";
    assert_eq!(
        reports(ComponentDefinitionNameCasingOption::KebabCase, source),
        [(
            String::from("'MyComponent'"),
            String::from("Component name 'MyComponent' is not kebab-case"),
            String::from("Rename the component to 'my-component'."),
        )]
    );
    // Upstream's converter cannot make a valid name out of a dotted one.
    let source = "<script>\nexport default { name: 'my.component' }\n</script>\n";
    assert_eq!(
        reports(ComponentDefinitionNameCasingOption::PascalCase, source),
        [(
            String::from("'my.component'"),
            String::from("Component name 'my.component' is not PascalCase"),
            String::from("Rename the component to a PascalCase name."),
        )]
    );
}

#[test]
fn the_option_selects_the_accepted_casing() {
    let source = "<script>\nexport default { name: 'MyComponent' }\n</script>\n<script setup>\ndefineOptions({ name: 'my-component' })\n</script>\n";
    assert_eq!(pascal(source), ["'my-component'"]);
    assert_eq!(kebab(source), ["'MyComponent'"]);

    let source = "<script>\napp.component('MyComponent', {})\napp.component('my-component', {})\napp.component('myComponent', {})\n</script>\n";
    assert_eq!(pascal(source), ["'my-component'", "'myComponent'"]);
    assert_eq!(kebab(source), ["'MyComponent'", "'myComponent'"]);
}

#[test]
fn follows_upstream_checkers() {
    for valid in ["MyComponent", "UI", "A", "Foo2", "", "Ünicode"] {
        let source = cstr!("<script>\nexport default {{ name: '{valid}' }}\n</script>\n");
        assert_eq!(pascal(&source), Vec::<String>::new(), "{valid}");
    }
    for invalid in [
        "myComponent",
        "my-component",
        "My_Component",
        "My Component",
        "My.Comp",
    ] {
        let source = cstr!("<script>\nexport default {{ name: '{invalid}' }}\n</script>\n");
        assert_eq!(pascal(&source).len(), 1, "{invalid}");
    }
    for valid in [
        "my-component",
        "component",
        "grid-2-col",
        "",
        "my-component-",
    ] {
        let source = cstr!("<script>\nexport default {{ name: '{valid}' }}\n</script>\n");
        assert_eq!(kebab(&source), Vec::<String>::new(), "{valid}");
    }
    for invalid in [
        "MyComponent",
        "-my-component",
        "my--component",
        "my_component",
        "my.component",
    ] {
        let source = cstr!("<script>\nexport default {{ name: '{invalid}' }}\n</script>\n");
        assert_eq!(kebab(&source).len(), 1, "{invalid}");
    }
}

#[test]
fn ignores_names_that_are_not_static_literals_or_definitions() {
    let silent = [
        // Computed or non-string names.
        "<script>\nconst name = 'my_component'\nexport default { name }\n</script>\n",
        "<script>\nexport default { name: NAME }\n</script>\n",
        "<script>\nexport default { name: `my_${suffix}` }\n</script>\n",
        "<script>\nexport default { name: 1 }\n</script>\n",
        "<script>\nexport default { [key]: 'my_component' }\n</script>\n",
        "<script>\napp.component(name, {})\n</script>\n",
        // Not a definition.
        "<script>\nconst options = { name: 'my_component' }\n</script>\n",
        "<script>\nfoo({ name: 'my_component' })\n</script>\n",
        "<script>\napp.component('my_component')\n</script>\n",
        "<script>\napp.component('my_component', A, B)\n</script>\n",
        "<script>\nthis.$options.components.foo('my_component', {})\n</script>\n",
        // `export default` and `defineOptions` belong to their own block kind.
        "<script setup>\nexport default { name: 'my_component' }\n</script>\n",
        "<script>\ndefineOptions({ name: 'my_component' })\n</script>\n",
        // Invalid syntax is the parser's report.
        "<script>\nexport default { name: 'my_component', }}\n</script>\n",
    ];
    for source in silent {
        assert_eq!(pascal(source), Vec::<String>::new(), "{source}");
    }
}

#[test]
fn reports_only_the_first_name_property() {
    let source =
        "<script>\nexport default { name: 'First_Name', name: 'second_name' }\n</script>\n";
    assert_eq!(pascal(source), ["'First_Name'"]);
    let source = "<script>\nexport default { ['name']: 'computed_literal' }\n</script>\n";
    assert_eq!(pascal(source), ["'computed_literal'"]);
}

#[test]
fn the_prefilter_admits_every_checked_shape() {
    for admitted in [
        "export default { name: 'x' }",
        "export default { 'name' : 'x' }",
        "export default { \"name\"\n: 'x' }",
        "export default { ['name']: 'x' }",
        "export default { name /* c */ : 'x' }",
        "export default { name // c\n : 'x' }",
        "app.component('x', y)",
        "app?.component('x', y)",
        "app\n  .component('x', y)",
        "app./* c */component('x', y)",
    ] {
        assert!(may_name_component(admitted.as_bytes()), "{admitted}");
    }
    for skipped in [
        "",
        "import Foo from './components/Foo.vue'",
        "const props = defineProps<{ title: string }>()",
        "const name = ref('x')",
        "console.log(name)",
        "const component = shallowRef()",
    ] {
        assert!(!may_name_component(skipped.as_bytes()), "{skipped}");
    }
}

#[test]
fn unicode_whitespace_does_not_hide_component_names() {
    for whitespace in ['\u{00a0}', '\u{feff}', '\u{2028}', '\u{2029}'] {
        for script in [
            format!("export default {{ name{whitespace}: 'bad_name' }}"),
            format!("app.{whitespace}component('bad_name', {{}})"),
        ] {
            let source = format!("<script>{script}</script>");
            assert_eq!(pascal(&source), ["'bad_name'"], "{script}");
        }
    }
}

#[test]
fn escaped_keys_computed_key_trivia_and_line_comments_reach_the_scanner() {
    for script in [
        r"export default { n\u0061me: 'bad_name' }",
        r"export default { 'na\u006de': 'bad_name' }",
        r"export default { ['na\x6de']: 'bad_name' }",
        "export default { ['name' ]: 'bad_name' }",
        "export default { ['name' /* key */ ]: 'bad_name' }",
        r"app.\u0063omponent('bad_name', {})",
        "app. // registration\ncomponent('bad_name', {})",
        "app. // registration\r\ncomponent('bad_name', {})",
        "app. // registration\u{2028}component('bad_name', {})",
    ] {
        assert!(may_name_component(script.as_bytes()), "{script}");
        let source = format!("<script>{script}</script>");
        assert_eq!(pascal(&source), ["'bad_name'"], "{script}");
    }
}

#[test]
fn computed_member_registrations_follow_the_upstream_identifier_selector() {
    assert_eq!(
        pascal("<script>app[component]('bad_name', {})</script>"),
        ["'bad_name'"]
    );
    for member in ["['component']", "[\"component\"]", "[`component`]"] {
        // Upstream's registration visitor selects an Identifier property.
        // Its independent definition visitor still checks the object's name.
        let source = format!(
            "<script>app{member}('bad_registration', {{ name: 'bad_definition' }})</script>"
        );
        assert_eq!(pascal(&source), ["'bad_definition'"], "{member}");
    }
}

#[test]
fn wrapped_typescript_default_objects_keep_definition_names() {
    for expression in [
        "{ name: 'bad_name' } as ComponentOptions",
        "{ name: 'bad_name' } satisfies ComponentOptions",
        "({ name: 'bad_name' })",
        "({ name: 'bad_name' } as ComponentOptions)!",
    ] {
        let source = format!("<script lang=\"ts\">export default {expression}</script>");
        assert_eq!(pascal(&source), ["'bad_name'"], "{expression}");
    }
}

#[test]
fn is_in_the_presets_that_ran_the_old_file_name_check() {
    for registry in [
        RuleRegistry::with_happy_path(),
        RuleRegistry::with_opinionated(),
        RuleRegistry::with_nuxt(),
        RuleRegistry::with_ecosystem(),
    ] {
        assert!(registry.has_rule(RULE));
        assert!(registry.has_rule("vue/component-file-name-casing"));
    }
    assert!(!RuleRegistry::with_essential().has_rule(RULE));
}

macro_rules! issue_source {
    ($file:literal) => {
        include_str!(concat!(
            "../../../../../../tests/_fixtures/differential/linter/component-definition-name-casing-sources/",
            $file
        ))
    };
}

/// The reported repro: each of the three definition-name sources is found in
/// PascalCase and kebab-case alike, and the empty helper stays silent.
#[test]
fn pinned_issue_sources_report_each_definition_name() {
    let sources = [
        (
            issue_source!("DefinitionCasing.vue.txt"),
            "\"definition_casing\"",
        ),
        (
            issue_source!("DefinitionCasingSetup.vue.txt"),
            "\"definition_casing_setup\"",
        ),
        (
            issue_source!("DefinitionCasingApp.vue.txt"),
            "\"helper_child\"",
        ),
    ];
    for (source, name) in sources {
        assert_eq!(pascal(source), [name], "{source}");
        assert_eq!(kebab(source), [name], "{source}");
    }
    let helper = issue_source!("HelperChild.vue.txt");
    assert_eq!(pascal(helper), Vec::<String>::new());
    assert_eq!(
        issue_source!("lint.config.mjs.txt"),
        "export default { vue: { linter: { rules: { \"vue/component-definition-name-casing\": \"error\" } } } };\n"
    );
}
