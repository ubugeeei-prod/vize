use super::super::validate_script_setup_semantics;
use super::script_setup_has_semantic_validator_candidates;

#[test]
fn semantic_validator_prefilter_is_a_strict_superset() {
    for source in [
        "const local = makeDefault()\nwithDefaults(defineProps(), { value: local })",
        "const\tlocal = makeDefault()\nwithDefaults(defineProps(), { value: local })",
        "function local() {}\ndefineOptions({ setup: local })",
        "const { value = 0 } = defineProps<{ value?: string }>()",
    ] {
        assert!(
            script_setup_has_semantic_validator_candidates(source),
            "validator candidate must not be skipped: {source}"
        );
    }
    for source in [
        "const value = compute()",
        "defineProps<{ value: string }>()",
        "import { fallback } from './defaults'\nwithDefaults(defineProps(), { value: fallback })",
    ] {
        assert!(
            !script_setup_has_semantic_validator_candidates(source),
            "obviously irrelevant source should keep the fast path: {source}"
        );
    }
}

#[test]
fn rejects_setup_local_values_in_every_hoisted_runtime_macro() {
    let cases = [
        (
            "withDefaults direct default",
            "const local = makeDefault()\nwithDefaults(defineProps<{ value?: string }>(), { value: local })",
            "withDefaults",
        ),
        (
            "withDefaults factory closure",
            "const local = makeDefault()\nwithDefaults(defineProps<{ value?: string }>(), { value: () => local })",
            "withDefaults",
        ),
        (
            "defineProps runtime validator",
            "const local = makeValidator()\ndefineProps({ value: { validator: local } })",
            "defineProps",
        ),
        (
            "defineProps runtime constructor",
            "class local {}\ndefineProps({ value: { type: local } })",
            "defineProps",
        ),
        (
            "defineEmits runtime validator",
            "const local = makeValidator()\ndefineEmits({ change: local })",
            "defineEmits",
        ),
        (
            "defineOptions component option",
            "const local = makeName()\ndefineOptions({ name: local })",
            "defineOptions",
        ),
        (
            "function declaration",
            "function local() {}\ndefineOptions({ setup: local })",
            "defineOptions",
        ),
        (
            "enum runtime value",
            "enum local { Ready }\ndefineOptions({ ready: local.Ready })",
            "defineOptions",
        ),
        (
            "namespace runtime value",
            "namespace local { export const value = 1 }\ndefineOptions({ value: local.value })",
            "defineOptions",
        ),
        (
            "defineModel runtime option",
            "const local = makeValidator()\ndefineModel<string>({ validator: local })",
            "defineModel",
        ),
        (
            "defineModel prop option beside runtime accessor",
            "const local = makeValidator()\ndefineModel<string>({ validator: local, get: value => value })",
            "defineModel",
        ),
    ];

    for (label, source, macro_name) in cases {
        let Err(error) = validate_script_setup_semantics(source) else {
            panic!("{label} should reject a setup-local reference");
        };
        assert_eq!(
            error.code.as_deref(),
            Some("SCRIPT_SETUP_MACRO_SCOPE"),
            "{label}"
        );
        assert!(error.message.contains(macro_name), "{label}: {error:?}");
        assert!(error.message.contains("`local`"), "{label}: {error:?}");
        let loc = error.loc.expect("invalid reference should have a location");
        assert_eq!(
            &source[loc.start..loc.end],
            "local",
            "{label} should highlight only the invalid identifier"
        );
    }
}

#[test]
fn allows_values_that_remain_valid_after_macro_hoisting() {
    let cases = [
        (
            "imported binding",
            "import { fallback } from './defaults'\nwithDefaults(defineProps<{ value?: string }>(), { value: fallback })",
        ),
        (
            "literal setup constant",
            "const fallback = 'ready'\nwithDefaults(defineProps<{ value?: string }>(), { value: fallback })",
        ),
        (
            "global binding",
            "defineProps({ value: { default: Math.random() } })",
        ),
        (
            "shadowed callback parameter",
            "const fallback = makeDefault()\nwithDefaults(defineProps<{ value?: string }>(), { value: (fallback) => fallback })",
        ),
        (
            "defineExpose setup local",
            "const exposed = makeValue()\ndefineExpose({ exposed })",
        ),
        (
            "local enum in a type argument",
            "enum LocalEnum { Ready }\ndefineProps<{ value?: LocalEnum }>()",
        ),
        (
            "local class in a type argument",
            "class LocalClass {}\ndefineProps<{ value?: LocalClass }>()",
        ),
        (
            "local value in a type query",
            "const runtime = { ready: true }\ndefineProps<{ value?: typeof runtime }>()",
        ),
        (
            "defineModel runtime accessors",
            "const local = computed(() => 1)\ndefineModel<number>({ get(value) { return value ?? local.value }, set(value) { return Math.max(local.value, value) } })",
        ),
        (
            "defineModel modifiers binding in setter",
            "const [model, modifiers] = defineModel<string>({ set(value) { return modifiers.trim ? value.trim() : value } })",
        ),
    ];

    for (label, source) in cases {
        validate_script_setup_semantics(source)
            .unwrap_or_else(|error| panic!("{label} should remain valid: {error:?}"));
    }
}

#[test]
fn reports_the_exact_authored_identifier_range() {
    let source =
        "const items = []\n\nwithDefaults(defineProps<{\n  items?: string[]\n}>(), { items })";
    let error = validate_script_setup_semantics(source).expect_err("local default must fail");
    let loc = error.loc.expect("scope error should be located");

    assert_eq!(&source[loc.start..loc.end], "items");
    assert_eq!((loc.start_line, loc.start_column), (5, 9));
    assert_eq!((loc.end_line, loc.end_column), (5, 14));
}
