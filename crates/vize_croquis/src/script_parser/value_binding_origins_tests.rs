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

#[test]
fn loop_lvalues_and_var_hoisting_cannot_keep_stale_loader_identity() {
    for statement in [
        "for (var loader of [() => import('./Other.vue')]) {}",
        "for (loader of [() => import('./Other.vue')]) {}",
        "for (loader in { other: 1 }) {}",
        "for ([loader] of [[() => import('./Other.vue')]]) {}",
        "for ({ loader } of [{ loader: () => import('./Other.vue') }]) {}",
        "for (var loader = () => import('./Other.vue'); false;) {}",
    ] {
        let mut source = vize_carton::String::from(
            "import { defineAsyncComponent } from 'vue'; function scope() { function loader() { return import('./Child.vue'); } ",
        );
        source.push_str(statement);
        source.push_str(" const Comp = defineAsyncComponent(loader); }");
        let result = parse_script_setup(source.as_str());
        let origins = result.value_binding_origins.as_ref().expect("capture");
        let reference_start = source.rfind("loader").expect("loader use") as u32;
        let scope = result
            .scopes
            .iter()
            .find(|scope| scope.get_binding("Comp").is_some())
            .expect("component scope")
            .id;
        let reference = ValueReference::new(
            scope,
            "loader",
            Span::new(reference_start, reference_start + 6),
        );
        let resolved = origins
            .resolve(&reference)
            .expect("known function declaration");
        assert_eq!(resolved.canonical, None, "{statement}");
    }
}

#[test]
fn cached_identity_keeps_each_use_tdz_independent() {
    let source = "const before = value; const value = 1; const after = value;";
    let (result, reference) = at(source, "value");
    let origins = result.value_binding_origins.as_ref().expect("capture");
    let late = origins.resolve(&reference).expect("later value use");
    assert_eq!(late.canonical, Some(late.binding));
    let early_start = source.find("value").expect("early use") as u32;
    let early = ValueReference::new(
        reference.scope,
        "value",
        Span::new(early_start, early_start + 5),
    );
    assert_eq!(
        origins.resolve(&early).expect("same declaration").canonical,
        None
    );
    assert_eq!(
        origins
            .resolve(&reference)
            .expect("later use still admitted")
            .canonical,
        Some(late.binding)
    );
}

#[test]
fn long_aliases_and_many_functions_use_indexed_iterative_identity() {
    use std::fmt::Write;
    let mut source = vize_carton::String::from(
        "import { defineAsyncComponent as origin } from 'vue'; const alias0 = origin;\n",
    );
    for index in 1..20_000 {
        writeln!(source, "const alias{index} = alias{};", index - 1).expect("source");
    }
    for index in 0..10_000 {
        writeln!(source, "function function{index}() {{}}").expect("source");
    }
    source.push_str("alias19999;");
    let (result, reference) = at(source.as_str(), "alias19999");
    let origins = result.value_binding_origins.as_ref().expect("capture");
    let resolved = origins.resolve(&reference).expect("alias binding");
    assert!(resolved.canonical.is_some());
    assert_eq!(resolved.import.expect("Vue origin").source, "vue");
    assert_eq!(origins.inventory(), [30_001, 30_001, 10_001, 0]);
    println!(
        "VALUE_BINDING_ORIGINS_INVENTORY inventory={:?} aliases=20000 functions=10000 source_bytes={}",
        origins.inventory(),
        source.len()
    );
}
