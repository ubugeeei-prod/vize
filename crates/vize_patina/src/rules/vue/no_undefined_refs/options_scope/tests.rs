use crate::linter::{LintResult, Linter};

const RULE: &str = "vue/no-undefined-refs";

fn lint(sfc: &str) -> LintResult {
    Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .lint_sfc(sfc, "Probe.vue")
}

/// Each report as `(start, end, name)`.
fn reported(sfc: &str) -> Vec<(u32, u32, std::string::String)> {
    lint(sfc)
        .diagnostics
        .iter()
        .map(|diagnostic| {
            assert_eq!(diagnostic.rule_name, RULE);
            let name = diagnostic
                .message
                .as_str()
                .strip_prefix("Variable '")
                .and_then(|rest| rest.strip_suffix("' is not defined"))
                .expect("undefined-ref message");
            (diagnostic.start, diagnostic.end, name.to_owned())
        })
        .collect()
}

fn at(sfc: &str, name: &str) -> (u32, u32, std::string::String) {
    let start = sfc.rfind(name).expect("reported name") as u32;
    (start, start + name.len() as u32, name.to_owned())
}

fn none() -> Vec<(u32, u32, std::string::String)> {
    Vec::new()
}

fn component(script: &str) -> std::string::String {
    format!(
        "<script>\n{script}\n</script>\n<template>{{{{ shared }}}} {{{{ missing }}}}</template>\n"
    )
}

#[test]
fn members_that_may_come_from_elsewhere_silence_the_component() {
    for script in [
        "export default { mixins: [shared] }",
        "export default defineComponent({ mixins: [shared], data: () => ({ own: 1 }) })",
        "export default { extends: Base }",
        "export default { ...base }",
        "export default { computed: { ...mapState(useStore, ['shared']) } }",
        "export default { computed: { ...mapGetters(['shared']) } }",
        "export default { methods: { ...mapActions(useStore, ['shared']) } }",
        "export default { methods: { ...helpers } }",
        "export default { computed: mapState(['shared']) }",
        "export default { methods: helpers }",
        "export default { props: sharedProps }",
        "export default { props: { ...sharedProps } }",
        "export default { setup() { return { ...toRefs(state) } } }",
        "export default { setup() { return useShared() } }",
        "export default { data: () => createState() }",
        "export default { computed: { [key]: () => 1 } }",
        "export default defineComponent(sharedOptions)",
        "export default defineNuxtComponent(sharedOptions)",
        "export default defineNuxtComponent({ mixins: [shared] })",
        "export default defineNuxtComponent({ computed: { ...mapState(useStore, ['shared']) } })",
        "export default defineNuxtComponent({ asyncData: () => load() })",
        "import Impl from './Impl'\nexport default Impl",
        "export default wrap({ data: () => ({ shared: 1 }) })",
    ] {
        assert_eq!(reported(&component(script)), none(), "{script}");
    }
}

#[test]
fn literal_members_still_report_the_rest() {
    for script in [
        "export default { mixins: [], data: () => ({ shared: 1 }) }",
        "export default { computed: { shared() { return 1 } } }",
        "export default { methods: { shared() {} } }",
        "export default { setup() { const shared = 1\nreturn { shared } } }",
        "export default { inheritAttrs: false, props: ['shared'] }",
        "export default defineComponent({ inject: { shared: {} } })",
        "export default defineNuxtComponent({ props: ['shared'] })",
        "export default defineNuxtComponent({ props: { shared: String } })",
        "export default defineNuxtComponent({ inject: ['shared'] })",
        "export default defineNuxtComponent({ computed: { shared() { return 1 } } })",
        "export default defineNuxtComponent({ methods: { shared() {} } })",
        "export default defineNuxtComponent({ data: () => ({ shared: 1 }) })",
        "export default defineNuxtComponent({ setup() { return { shared: 1 } } })",
        "export default defineNuxtComponent({ async asyncData() { return { shared: 1 } } })",
        "export default defineNuxtComponent({ asyncData: () => ({ shared: 1 }) })",
    ] {
        let sfc = component(script);
        assert_eq!(reported(&sfc), [at(&sfc, "missing")], "{script}");
    }

    // A `setup()` without a return defines nothing, so both reads stay
    // reported.
    let sfc = component("export default { setup() { onMounted(load) } }");
    let shared = sfc.find("shared").expect("shared read") as u32;
    assert_eq!(
        reported(&sfc),
        [
            (shared, shared + 6, "shared".to_owned()),
            at(&sfc, "missing")
        ]
    );
}

#[test]
fn script_setup_keeps_its_reports() {
    let setup = "<script setup>\nconst shared = 1\n</script>\n<template>{{ shared }} {{ missing }}</template>\n";
    assert_eq!(reported(setup), [at(setup, "missing")]);

    let beside = "<script>\nexport default { inheritAttrs: false }\n</script>\n<script setup>\nconst shared = 1\n</script>\n<template>{{ shared }} {{ missing }}</template>\n";
    assert_eq!(reported(beside), [at(beside, "missing")]);
}

#[test]
fn pinned_unresolved_member_components() {
    let mixins = include_str!(
        "../../../../../../../tests/_fixtures/differential/linter/no-undefined-refs-unresolved-members/Mixins.vue.txt"
    );
    let map_helpers = include_str!(
        "../../../../../../../tests/_fixtures/differential/linter/no-undefined-refs-unresolved-members/MapHelpers.vue.txt"
    );
    let nuxt = include_str!(
        "../../../../../../../tests/_fixtures/differential/linter/no-undefined-refs-unresolved-members/NuxtComponent.vue.txt"
    );
    assert_eq!(reported(mixins), none());
    assert_eq!(reported(map_helpers), none());
    assert_eq!(reported(nuxt), [at(nuxt, "titel")]);
}
