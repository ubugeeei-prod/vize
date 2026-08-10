//! Class-component `@Prop` usage-site contracts (#3298).
//!
//! A `@Prop`-decorated member used to be a template binding only, so the
//! generated module exported `type Props = {}` and a parent template could pass
//! a wrongly-typed or misspelled attribute without a single diagnostic. These
//! tests pin the module-level contract a parent consumes: `export type Props`
//! carries the declared members with the runtime `required` flag, exactly as a
//! `defineProps` / Options API `props:` component does.

use super::generate_virtual_ts_with_offsets_options_api;

fn class_component_virtual_ts(script: &str, template: &str) -> vize_carton::String {
    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full())
        .with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &Default::default(),
    )
    .code
}

#[test]
fn test_required_prop_decorator_exports_a_required_props_member() {
    // The exact `HelloDecorator.vue` shape from
    // tests/_fixtures/_projects/class-component.
    let script = r#"import { Vue, Prop } from 'vue-property-decorator'

export default class HelloDecorator extends Vue {
  @Prop({ type: String, required: true }) readonly name!: string;

  count = 0;

  get greeting(): string {
    return `Hello, ${this.name}! (${this.count})`;
  }
}
"#;
    let code = class_component_virtual_ts(script, "<button>{{ greeting }}</button>");

    assert!(
        code.contains("export type Props = {\n  name: string;\n};"),
        "a required @Prop must be a required member of the exported Props contract:\n{code}",
    );
    assert!(
        !code.contains("export type Props = {};"),
        "the empty Props no-op must be gone for a class component with props:\n{code}",
    );
    // `required: true` must not slip through as an optional member. Matched on
    // the Props body only: `__VizeVueComponentOptions` legitimately carries an
    // unrelated optional `name?: string` component-option field.
    assert!(
        !code.contains("  name?: string;\n};"),
        "`required: true` must not be emitted as an optional member:\n{code}",
    );
}

#[test]
fn test_prop_decorator_without_required_exports_an_optional_member() {
    // Vue defaults a runtime prop to `required: false`, so the `!` definite
    // assignment assertion (an author-side claim) must not make the prop a
    // caller-side obligation.
    let script = r#"import { Vue, Prop } from 'vue-property-decorator'

export default class Widget extends Vue {
  @Prop() readonly label!: string;
  @Prop({ type: Number }) readonly size?: number;
}
"#;
    let code = class_component_virtual_ts(script, "<div>{{ label }}{{ size }}</div>");

    assert!(
        code.contains("label?: string;"),
        "a @Prop without `required: true` is optional at the usage site:\n{code}",
    );
    assert!(
        code.contains("size?: number;"),
        "a `?` member stays optional and keeps its declared type:\n{code}",
    );
}

#[test]
fn test_prop_decorator_type_prefers_the_declared_annotation() {
    // `@Prop({ type: Array })` erases to `unknown[]` at runtime; the member's
    // TS annotation is strictly more precise and is what the class instance
    // type already resolved template bindings to.
    let script = r#"import { Vue, Prop } from 'vue-property-decorator'

export default class List extends Vue {
  @Prop({ type: Array, required: true }) readonly items!: Array<{ id: number }>;
  @Prop({ type: Number, default: 0 }) readonly offset!: number;
}
"#;
    let code = class_component_virtual_ts(script, "<div>{{ items }}{{ offset }}</div>");

    assert!(
        code.contains("items: Array<{ id: number }>;"),
        "the declared annotation must win over the runtime ctor:\n{code}",
    );
    assert!(
        !code.contains("items: unknown[];"),
        "the runtime ctor must not overwrite a precise annotation:\n{code}",
    );
    // A `default:` still leaves the prop optional for callers, but the template
    // binding is unwrapped because the default always supplies a value.
    assert!(
        code.contains("offset?: number;"),
        "a defaulted prop is optional for callers:\n{code}",
    );
    assert!(
        code.contains(
            "const offset = props[\"offset\"] as Exclude<__DefineProps<Props>[\"offset\"], undefined>;"
        ) || code.contains("const offset = props[\"offset\"] as Exclude<Props[\"offset\"], undefined>;"),
        "a defaulted prop must stay non-undefined inside its own template:\n{code}",
    );
}

#[test]
fn test_renaming_prop_decorators_keep_the_class_instance_bridge() {
    // `@PropSync`/`@Model`/`@VModel` rename or pair their prop with a generated
    // computed member, so they declare no plain member-name contract and must
    // keep resolving through the class instance type.
    let script = r#"import { Vue, PropSync, Model, VModel } from 'vue-property-decorator'

export default class Renaming extends Vue {
  @PropSync('title') syncedTitle!: string;
  @Model('change') modelValue!: string;
  @VModel() selected!: string;
}
"#;
    let code = class_component_virtual_ts(
        script,
        "<div>{{ syncedTitle }}{{ modelValue }}{{ selected }}</div>",
    );

    assert!(
        code.contains("export type Props = {};"),
        "renaming decorators must not fabricate a member-named prop contract:\n{code}",
    );
    for name in ["syncedTitle", "modelValue", "selected"] {
        let binding = vize_carton::cstr!(
            "const {name}: __VizeOptionsBinding<typeof __default__, \"{name}\">"
        );
        assert!(
            code.contains(binding.as_str()),
            "{name} must keep resolving through the class instance type:\n{code}",
        );
    }
}

#[test]
fn test_class_component_props_reach_the_component_instance_type() {
    // The default export is what a parent template resolves `$props` from; an
    // empty `Props` there is exactly why usage sites had no contract.
    let script = r#"import { Vue, Prop } from 'vue-property-decorator'

export default class HelloDecorator extends Vue {
  @Prop({ type: String, required: true }) readonly name!: string;
}
"#;
    let code = class_component_virtual_ts(script, "<div>{{ name }}</div>");

    assert!(
        code.contains("$props: Props;"),
        "the component instance must expose the props contract:\n{code}",
    );
    assert!(
        code.contains("const name = props[\"name\"];"),
        "a declared prop is bound from the props contract, not the instance bridge:\n{code}",
    );
    assert!(
        !code.contains("const name: __VizeOptionsBinding<typeof __default__, \"name\">"),
        "a declared prop must not be declared twice (TS2451):\n{code}",
    );
}
