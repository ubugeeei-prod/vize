//! The synthesized `update:` emit payload for `defineModel` (#3904): an
//! optional model without a default carries `T | undefined` — its `ModelRef`
//! type, and what vue-tsc's synthesized listener accepts — while required
//! models and models with defaults keep the bare payload.

use crate::virtual_ts::generate_virtual_ts;

fn emits_of(script: &str) -> std::string::String {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, "<div>{{ model }}</div>");
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    let code = output.code.as_str();
    let start = code.find("export type Emits").expect("Emits alias");
    let end = code[start..].find(";\n").map_or(code.len(), |e| start + e);
    code[start..end].into()
}

fn code_of(script: &str) -> std::string::String {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, "<div>{{ model }}</div>");
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    generate_virtual_ts(&summary, Some(script), Some(&root), 0)
        .code
        .into()
}

fn props_type_of(script: &str) -> std::string::String {
    let code = code_of(script);
    let start = code
        .find("export type Props")
        .or_else(|| code.find("type __VizeResolvedProps"))
        .expect("Props alias");
    let end = code[start..]
        .find("\n\n")
        .map_or(code.len(), |end| start + end);
    code[start..end].into()
}

#[test]
fn an_optional_model_update_payload_carries_undefined() {
    let emits = emits_of("const model = defineModel<string>()\nvoid model;\n");
    assert!(
        emits.contains("\"update:modelValue\": [value: (string) | undefined]"),
        "optional model must accept undefined in its update payload:\n{emits}"
    );
}

#[test]
fn required_and_defaulted_models_keep_the_bare_payload() {
    let required = emits_of("const model = defineModel<string>({ required: true })\nvoid model;\n");
    assert!(
        required.contains("\"update:modelValue\": [value: string]"),
        "required model keeps the bare payload:\n{required}"
    );
    let defaulted =
        emits_of("const model = defineModel<string>({ default: \"x\" })\nvoid model;\n");
    assert!(
        defaulted.contains("\"update:modelValue\": [value: string]"),
        "defaulted model keeps the bare payload:\n{defaulted}"
    );
}

#[test]
fn an_untyped_model_keeps_the_bare_unknown_payload() {
    let emits = emits_of("const model = defineModel()\nvoid model;\n");
    assert!(
        emits.contains("\"update:modelValue\": [value: unknown]"),
        "an untyped model keeps the bare `unknown` payload:\n{emits}"
    );
}

#[test]
fn runtime_constructor_model_flows_into_public_props_and_emits() {
    let code = code_of("const model = defineModel({ type: String, default: '' })\nvoid model;\n");

    assert!(
        code.contains("\"modelValue\"?: string;"),
        "runtime constructor model should expose a string public prop:\n{code}"
    );
    assert!(
        code.contains("\"update:modelValue\": [value: string]"),
        "runtime constructor model should expose a string update payload:\n{code}"
    );
}

#[test]
fn runtime_constructor_model_array_type_can_include_null() {
    let code = code_of("const model = defineModel({ type: [String, null] })\nvoid model;\n");

    assert!(
        code.contains("\"modelValue\"?: string | null;"),
        "runtime constructor model should preserve null public prop type:\n{code}"
    );
    assert!(
        code.contains("\"update:modelValue\": [value: (string | null) | undefined]"),
        "runtime constructor model should preserve null update payload:\n{code}"
    );
}

#[test]
fn optional_runtime_constructor_model_update_payload_carries_undefined() {
    let emits = emits_of("const model = defineModel({ type: String })\nvoid model;\n");
    assert!(
        emits.contains("\"update:modelValue\": [value: (string) | undefined]"),
        "optional runtime constructor model should accept undefined:\n{emits}"
    );
}

#[test]
fn a_function_typed_model_parenthesizes_the_base() {
    let emits = emits_of("const model = defineModel<() => string>()\nvoid model;\n");
    assert!(
        emits.contains("\"update:modelValue\": [value: (() => string) | undefined]"),
        "a function-typed model must not absorb the union into its return type:\n{emits}"
    );
}

#[test]
fn define_model_prop_names_are_escaped_as_typescript_string_literals() {
    let script = "const model = defineModel<string>(\"quote\\\"line\\u2028\")\nvoid model;\n";
    let props = props_type_of(script);
    let emits = emits_of(script);

    assert!(
        props.contains(r#""quote\"line\u2028"?: string;"#),
        "model prop names must be escaped in public props:\n{props}"
    );
    assert!(
        props.contains(r#""quote\"line\u2028Modifiers"?: Partial<Record<string, true>>;"#),
        "model modifier prop names must be escaped in public props:\n{props}"
    );
    assert!(
        emits.contains(r#""update:quote\"line\u2028": [value: (string) | undefined]"#),
        "model update event names must be escaped in public emits:\n{emits}"
    );
}

#[test]
fn define_props_prop_names_are_escaped_as_typescript_string_literals() {
    let props =
        props_type_of("defineProps({ \"foo-bar\": String, \"quote\\\"line\\u2028\": Number })\n");

    assert!(
        props.contains(r#""foo-bar"?: string;"#),
        "hyphenated macro prop names must be quoted in public props:\n{props}"
    );
    assert!(
        props.contains(r#""quote\"line\u2028"?: number;"#),
        "escaped macro prop names must be valid TypeScript literals:\n{props}"
    );
}

#[test]
fn macro_props_only_suppress_colliding_model_members() {
    let props = props_type_of(
        r#"defineProps({ titleModifiers: Boolean })
const title = defineModel<string>("title")
void title;
"#,
    );

    assert!(
        props.contains(r#""title"?: string;"#),
        "a macro prop colliding with modifiers must not suppress the model prop:\n{props}"
    );
    assert!(
        !props.contains(r#""titleModifiers"?: Partial<Record<string, true>>;"#),
        "the generated modifier prop should be skipped when a macro prop owns the alias:\n{props}"
    );
}

#[test]
fn later_model_names_cannot_duplicate_modifier_aliases() {
    let props = props_type_of(
        r#"const title = defineModel<string>("title")
const modifierAlias = defineModel<number>("titleModifiers")
void title;
void modifierAlias;
"#,
    );

    assert!(
        props.contains(r#""title"?: string;"#),
        "the first model prop should be emitted:\n{props}"
    );
    assert!(
        props.contains(r#""titleModifiers"?: Partial<Record<string, true>>;"#),
        "the first model modifier alias should be emitted:\n{props}"
    );
    assert!(
        !props.contains(r#""titleModifiers"?: number;"#),
        "a later model prop must not duplicate an earlier modifier alias:\n{props}"
    );
    assert!(
        props.contains(r#""titleModifiersModifiers"?: Partial<Record<string, true>>;"#),
        "only the colliding generated member should be skipped:\n{props}"
    );
}

#[test]
fn a_typed_define_emits_intersection_carries_the_same_payload() {
    let optional = emits_of(
        "const emit = defineEmits<{ change: [] }>()\nconst model = defineModel<string>()\nvoid emit;\nvoid model;\n",
    );
    assert!(
        optional.contains("\"update:modelValue\": [value: (string) | undefined]"),
        "the typed `defineEmits` intersection accepts undefined for an optional model:\n{optional}"
    );
    let required = emits_of(
        "const emit = defineEmits<{ change: [] }>()\nconst model = defineModel<string>({ required: true })\nvoid emit;\nvoid model;\n",
    );
    assert!(
        required.contains("\"update:modelValue\": [value: string]"),
        "the typed `defineEmits` intersection keeps the bare payload for a required model:\n{required}"
    );
}

#[test]
fn runtime_emits_carry_the_same_payload() {
    let optional = emits_of(
        "const emit = defineEmits([\"change\"])\nconst model = defineModel<string>()\nvoid emit;\nvoid model;\n",
    );
    assert!(
        optional.contains("(event: \"update:modelValue\", value: (string) | undefined) => void"),
        "runtime emits accept undefined for an optional model:\n{optional}"
    );
    let required = emits_of(
        "const emit = defineEmits([\"change\"])\nconst model = defineModel<string>({ required: true })\nvoid emit;\nvoid model;\n",
    );
    assert!(
        required.contains("(event: \"update:modelValue\", value: string) => void"),
        "runtime emits keep the bare payload for a required model:\n{required}"
    );
}
