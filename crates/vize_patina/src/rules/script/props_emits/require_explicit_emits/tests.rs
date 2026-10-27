//! Script-side cases. Every expectation was checked against eslint-plugin-vue
//! 10.9.2's `vue/require-explicit-emits`.

use crate::linter::{LintResult, Linter};

mod other_instances;
mod template;

fn lint_sfc(sfc: &str) -> LintResult {
    Linter::new()
        .with_enabled_rules(Some(vec!["script/require-explicit-emits".into()]))
        .lint_sfc(sfc, "Probe.vue")
}

/// The source text of every finding, e.g. `'input'`.
fn reported(sfc: &str) -> Vec<&str> {
    lint_sfc(sfc)
        .diagnostics
        .iter()
        .map(|diagnostic| &sfc[diagnostic.start as usize..diagnostic.end as usize])
        .collect()
}

fn setup(body: &str) -> String {
    format!("<script setup lang=\"ts\">\n{body}\n</script>\n")
}

fn options(body: &str) -> String {
    format!("<script>\n{body}\n</script>\n")
}

// --- defineEmits -----------------------------------------------------------

#[test]
fn runtime_declarations() {
    let cases = [
        (
            "const emit = defineEmits(['change', 'input']); emit('change'); emit('input')",
            vec![],
        ),
        (
            "const emit = defineEmits(['change']); emit('change'); emit('input')",
            vec!["'input'"],
        ),
        (
            "const emit = defineEmits({ change: null }); emit('submit')",
            vec!["'submit'"],
        ),
        ("const emit = defineEmits([]); emit(`save`)", vec!["`save`"]),
        ("const emit = defineEmits({}); emit('save')", vec!["'save'"]),
        (
            "const names = ['a']; const emit = defineEmits(names); emit('a'); emit('b')",
            vec!["'b'"],
        ),
        (
            "const dispatch = defineEmits(['a']); dispatch('b')",
            vec!["'b'"],
        ),
        (
            "const emit = defineEmits(['a']); function f() { emit('b') }",
            vec!["'b'"],
        ),
        (
            "const emit = defineEmits(['a']); const name = 'b'; emit(name)",
            vec![],
        ),
    ];
    for (body, expected) in cases {
        assert_eq!(reported(&setup(body)), expected, "{body}");
    }
}

#[test]
fn type_declarations() {
    let cases = [
        (
            "const emit = defineEmits<{ change: [id: number] }>(); emit('change'); emit('x')",
            vec!["'x'"],
        ),
        (
            "const emit = defineEmits<{ (e: 'a' | 'b'): void }>(); emit('b'); emit('c')",
            vec!["'c'"],
        ),
        (
            "const emit = defineEmits<(e: 'a') => void>(); emit('a'); emit('c')",
            vec!["'c'"],
        ),
        (
            "const emit = defineEmits<{}>(); emit('save')",
            vec!["'save'"],
        ),
        // A non-literal event parameter declares nothing, unlike an unknown type.
        (
            "const emit = defineEmits<{ (e: string): void }>(); emit('a')",
            vec!["'a'"],
        ),
        (
            "interface Base { (e: 'base'): void }\n\
             interface Emits extends Base { (e: 'a'): void }\n\
             type All = Emits & { b: [] }\n\
             const emit = defineEmits<All>(); emit('a'); emit('b'); emit('base')",
            vec!["'base'"],
        ),
    ];
    for (body, expected) in cases {
        assert_eq!(reported(&setup(body)), expected, "{body}");
    }
}

#[test]
fn unenumerable_declarations_declare_every_event() {
    for body in [
        "const names = ['save']; const emit = defineEmits([...names]); emit('cancel')",
        "const emit = defineEmits({ ...others }); emit('cancel')",
        "const key = 'k'; const emit = defineEmits({ [key]: null }); emit('cancel')",
        "import type { Emits } from './types'; const emit = defineEmits<Emits>(); emit('cancel')",
        "const emit = defineEmits(); emit('cancel')",
        "let names = ['a']; const emit = defineEmits(names); emit('cancel')",
    ] {
        assert_eq!(reported(&setup(body)), Vec::<&str>::new(), "{body}");
    }
}

#[test]
fn script_setup_without_a_macro_is_not_checked() {
    for body in [
        "const emit = useEmitter(); emit('change')",
        "function f() { emit('early') }\nconst emit = defineEmits(['a'])",
    ] {
        assert_eq!(reported(&setup(body)), Vec::<&str>::new(), "{body}");
    }
}

#[test]
fn define_model_does_not_declare_update_events() {
    let body =
        "const model = defineModel(); const emit = defineEmits([]); emit('update:modelValue')";
    assert_eq!(reported(&setup(body)), vec!["'update:modelValue'"]);
}

// --- Options API -----------------------------------------------------------

#[test]
fn a_component_without_an_emits_option_declares_nothing() {
    let sfc = options("export default { methods: { onClick() { this.$emit('change') } } }");
    assert_eq!(reported(&sfc), vec!["'change'"]);
}

#[test]
fn options_declarations() {
    let cases = [
        (
            "export default { emits: ['a'], methods: { f() { this.$emit('a'); this.$emit('b') } } }",
            vec!["'b'"],
        ),
        (
            "export default { emits: { a: null, 'b-c': (x) => true }, created() { this.$emit('b-c'); this.$emit('d') } }",
            vec!["'d'"],
        ),
        (
            "export default { emits: [...base], created() { this.$emit('d') } }",
            vec![],
        ),
        (
            "const emits = ['a']; export default { emits, created() { this.$emit('d') } }",
            vec![],
        ),
        (
            "export default { emits: ['a'] as const, created() { this.$emit('d') } }",
            vec![],
        ),
    ];
    for (body, expected) in cases {
        assert_eq!(reported(&options(body)), expected, "{body}");
    }
}

#[test]
fn this_dispatches_anywhere_inside_the_component() {
    let body = "export default { methods: { f() {
        const vm = this; vm.$emit('alias')
        let other = this; other.$emit('let-alias')
        function inner() { this.$emit('inner') }
        const arrow = () => this?.$emit('arrow')
        this.$emit?.('optional'); this['$emit']('computed')
        ;(this as any).$emit('cast'); picker.$emit('picker')
    } } }";
    assert_eq!(
        reported(&options(body)),
        vec!["'alias'", "'inner'", "'arrow'", "'optional'", "'computed'"]
    );
}

#[test]
fn setup_context_dispatches() {
    let cases = [
        (
            "export default { setup(props, { emit }) { emit('a') } }",
            vec!["'a'"],
        ),
        (
            "export default { setup(props, { emit: send }) { send('a') } }",
            vec!["'a'"],
        ),
        (
            "export default { setup: (props, ctx) => { ctx.emit('a'); ctx.$emit('b') } }",
            vec!["'a'"],
        ),
        (
            "export default { setup(props, { emit = noop }) { emit('a') } }",
            vec![],
        ),
        (
            "export default { setup(props, ctx = {}) { ctx.emit('a') } }",
            vec![],
        ),
    ];
    for (body, expected) in cases {
        assert_eq!(reported(&options(body)), expected, "{body}");
    }
}

#[test]
fn the_innermost_vue_object_owns_a_call() {
    let body = "import { defineComponent } from 'vue'\n\
        const helper = { methods: { f() { this.$emit('helper') } } }\n\
        export default { components: { Inner: defineComponent({ emits: ['a'], methods: {\n\
        g() { this.$emit('a'); this.$emit('inner') } } }) },\n\
        created() { this.$emit('a') } }";
    assert_eq!(reported(&options(body)), vec!["'inner'", "'a'"]);
}

#[test]
fn vue_objects_are_recognized_as_upstream_does() {
    let cases = [
        (
            "new Vue({ created() { this.$emit('instance') } })",
            vec!["'instance'"],
        ),
        (
            "Vue.component('x', { created() { this.$emit('global') } })",
            vec!["'global'"],
        ),
        (
            "app.mixin({ created() { this.$emit('mixin') } })",
            vec!["'mixin'"],
        ),
        (
            "Other.extend({ created() { this.$emit('extend') } })",
            vec![],
        ),
        (
            "// @vue/component\nconst c = {\n created() { this.$emit('mark') } }",
            vec!["'mark'"],
        ),
        (
            "import { defineComponent } from 'lib'\ndefineComponent({ created() { this.$emit('x') } })",
            vec![],
        ),
        (
            "const c = { created() { this.$emit('plain') } }; export default c",
            vec![],
        ),
    ];
    for (body, expected) in cases {
        assert_eq!(reported(&options(body)), expected, "{body}");
    }
}

#[test]
fn a_standalone_export_default_is_not_a_component() {
    let mut linter = crate::rules::script::ScriptLinter::new();
    linter.add_rule(Box::new(crate::rules::script::RequireExplicitEmits));
    let source = "export default { created() { this.$emit('x') } }";
    assert_eq!(linter.lint(source, 0).warning_count, 0);
    let defined = "defineComponent({ created() { this.$emit('x') } })";
    assert_eq!(linter.lint(defined, 0).warning_count, 1);
}

#[test]
fn setup_names_resolve_into_the_sibling_script() {
    // Upstream lints both blocks as one program.
    let typed = "<script lang=\"ts\">\nexport interface Emits extends Base {}\n\
        type Alias = Emits | { (e: 'a'): void }\n</script>\n\
        <script setup lang=\"ts\">\nconst emit = defineEmits<Alias>(); emit('a'); emit('x')\n</script>\n\
        <template><i @click=\"emit('y')\" /></template>\n";
    assert_eq!(reported(typed), vec!["'x'", "'y'"]);
    let runtime = "<script>\nconst list = ['b']\n</script>\n\
        <script setup>\nconst emit = defineEmits(list); emit('b'); emit('c')\n</script>\n";
    assert_eq!(reported(runtime), vec!["'c'"]);
}
