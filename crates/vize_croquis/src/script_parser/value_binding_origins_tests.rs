//! Lexical identity admission and refusal, independently of component policy.
use super::parse_script_setup;
use super::value_binding_origins::ValueReference;
use crate::scope::ScopeKind;
use oxc_span::Span;

fn at(source: &str, name: &str) -> (super::ScriptParseResult, ValueReference) {
    let result = parse_script_setup(source);
    let root = result
        .scopes
        .iter()
        .find(|scope| scope.kind == ScopeKind::ScriptSetup)
        .expect("setup")
        .id;
    let start = source.rfind(name).expect("reference") as u32;
    let reference = ValueReference::new(root, name, Span::new(start, start + name.len() as u32));
    (result, reference)
}

#[test]
fn stable_opaque_functions_and_const_leaves_keep_identity() {
    let (result, reference) = at("function f() {} const alias = f; alias", "alias");
    let origins = result.value_binding_origins.as_ref().expect("capture");
    let resolved = origins.resolve(&reference).expect("binding");
    assert_ne!(resolved.binding, resolved.canonical.expect("stable target"));
    assert!(resolved.import.is_none());
    for source in [
        "const value = ref(''); value",
        "const { value } = opaque(); value",
        "const [value] = opaque(); value",
    ] {
        let (result, reference) = at(source, "value");
        let origins = result.value_binding_origins.as_ref().expect("capture");
        let resolved = origins.resolve(&reference).expect("binding");
        assert_eq!(resolved.canonical, Some(resolved.binding));
        assert!(resolved.import.is_none());
    }
}

#[test]
fn reassignment_tdz_and_mutable_bindings_refuse_value_authority() {
    for (source, name) in [
        ("function f() {} f = () => 1; f", "f"),
        ("const alias = later; const later = () => 1; alias", "alias"),
        (
            "let factory = () => 1; const alias = factory; alias",
            "alias",
        ),
        (
            "const f = () => 1; const alias = f; f = () => 2; alias",
            "alias",
        ),
    ] {
        let (result, reference) = at(source, name);
        let origins = result.value_binding_origins.as_ref().expect("capture");
        assert!(
            origins
                .resolve(&reference)
                .expect("known binding")
                .canonical
                .is_none(),
            "{source}"
        );
    }
}

#[test]
fn closure_parameters_and_hoisted_function_shadow_imports() {
    for source in [
        "import { ref } from 'vue'; function f(ref) { const value = ref; }",
        "import { ref } from 'vue'; function f() { const value = ref; function ref() {} }",
        "import { ref } from 'vue'; function f() { const value = ref; class ref {} }",
        "import { ref } from 'vue'; try {} catch (ref) { const value = ref; }",
    ] {
        let result = parse_script_setup(source);
        let origins = result.value_binding_origins.as_ref().expect("capture");
        let scope = result
            .scopes
            .iter()
            .find(|scope| scope.get_binding("value").is_some())
            .expect("local")
            .id;
        let start = source.rfind("value").expect("declaration") as u32;
        let reference = ValueReference::new(scope, "value", Span::new(start + 1000, start + 1005));
        assert!(
            origins
                .resolve(&reference)
                .expect("local binding")
                .import
                .is_none(),
            "{source}"
        );
    }
}

#[test]
fn incomplete_and_two_program_domains_refuse_but_absent_script_preserves() {
    let (result, reference) = at(
        "import { ref } from 'vue'; const value = ref; value",
        "value",
    );
    let mut setup = result.into_croquis();
    setup.merge_plain_script(crate::Croquis::new());
    assert!(
        setup
            .value_binding_origins
            .as_ref()
            .expect("sole store")
            .resolve(&reference)
            .expect("binding")
            .import
            .is_some()
    );

    let plain = super::parse_script("const value = 1").into_croquis();
    setup.merge_plain_script(plain);
    assert!(
        !setup
            .value_binding_origins
            .as_ref()
            .expect("two programs")
            .is_complete()
    );

    let result = parse_script_setup("import { ref } from 'vue'; const value = ref; missing.");
    assert!(
        !result
            .value_binding_origins
            .as_ref()
            .expect("recovered domain")
            .is_complete()
    );
}
