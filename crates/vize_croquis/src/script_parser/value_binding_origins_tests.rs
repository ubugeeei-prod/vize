//! Lexical identity admission and refusal, independently of component policy.
use super::value_binding_origins::ValueReference;
use crate::scope::ScopeKind;
use oxc_span::Span;

fn parse_script_setup(source: &str) -> super::ScriptParseResult {
    super::parse_script_setup_for_value_origins::<false>(
        source, None, false, false, false, false, None,
    )
}

#[test]
fn root_async_component_shapes_admit_value_origins() {
    for source in [
        "import { defineAsyncComponent } from 'vue'; const Child = defineAsyncComponent(() => import('./Child.vue'));",
        "import { defineAsyncComponent } from 'vue'; function loader() { return import('./Child.vue'); } const Child = defineAsyncComponent(loader);",
        "import { defineAsyncComponent } from 'vue'; const Child = defineAsyncComponent({ timeout: 100, 'loader': () => import('./Child.vue') });",
        "import * as Vue from 'vue'; const Child = Vue.defineAsyncComponent(() => import('./Child.vue'));",
    ] {
        let result = super::parse_script_setup(source);
        let origins = result
            .value_binding_origins
            .as_ref()
            .expect("root component call demands capture");
        assert!(origins.is_complete(), "{source}");
        assert_eq!(
            origins.async_components.module_for(origins, "Child"),
            Some("./Child.vue"),
            "{source}"
        );
    }
}

#[test]
fn nested_async_component_registry_remains_uncaptured() {
    let source = "import { defineAsyncComponent } from 'vue'; const builtInComponents = { Child: defineAsyncComponent(() => import('./Child.vue')) };";
    let result = super::parse_script_setup(source);
    assert!(result.value_binding_origins.is_none());
    assert!(result.bindings.bindings.contains_key("builtInComponents"));
}

#[test]
fn provenance_tests_can_force_capture_without_a_component_call() {
    let source = "import { ref } from 'vue'; const value = ref;";
    assert!(
        super::parse_script_setup(source)
            .value_binding_origins
            .is_none()
    );
    let result = parse_script_setup(source);
    let origins = result
        .value_binding_origins
        .as_ref()
        .expect("test-only forced capture");
    assert!(origins.is_complete());
}

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
        "import { ref } from 'vue'; function f() { try {} catch (ref) { const value = ref; } }",
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
fn constructor_and_template_reads_preserve_unrelated_import_authority() {
    for expression in ["new Date()", "String.raw`value`", "`value ${1}`"] {
        let mut source = vize_carton::String::from("import { defineAsyncComponent } from 'vue'; ");
        source.push_str(expression);
        source.push_str("; defineAsyncComponent");
        let (result, reference) = at(source.as_str(), "defineAsyncComponent");
        let origins = result.value_binding_origins.as_ref().expect("capture");
        assert!(origins.is_complete(), "{expression}");
        let resolved = origins.resolve(&reference).expect("Vue import");
        assert!(resolved.canonical.is_some(), "{expression}");
        assert_eq!(resolved.import.expect("import authority").source, "vue");
    }
}

#[test]
fn constructor_and_template_writes_invalidate_only_the_written_binding() {
    for expression in [
        "new Wrapper(loader = () => import('./Other.vue'))",
        "new Wrapper(...[loader = () => import('./Other.vue')])",
        "tag`${loader = () => import('./Other.vue')}`",
        "(loader = tag)``",
        "`value ${loader = () => import('./Other.vue')}`",
    ] {
        let mut source =
            vize_carton::String::from("function loader() { return import('./Child.vue'); } ");
        source.push_str(expression);
        source.push_str("; loader");
        let (result, reference) = at(source.as_str(), "loader");
        let origins = result.value_binding_origins.as_ref().expect("capture");
        assert!(origins.is_complete(), "{expression}");
        assert_eq!(
            origins
                .resolve(&reference)
                .expect("loader binding")
                .canonical,
            None,
            "{expression}"
        );
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

#[test]
fn unsupported_top_level_try_refuses_the_original_domain() {
    let result =
        parse_script_setup("import { ref } from 'vue'; try {} catch (ref) { const value = ref; }");
    let origins = result
        .value_binding_origins
        .as_ref()
        .expect("original program capture");
    assert!(!origins.is_complete());
}

#[test]
fn demanded_builtin_walk_retains_the_same_lexical_bindings() {
    let source = "import { ref } from 'vue'; const count = ref(0); const alias = count;";
    let regular = parse_script_setup(source);
    let demanded = super::parse_script_setup_for_value_origins::<true>(
        source, None, false, false, false, false, None,
    );
    let normal = regular
        .value_binding_origins
        .as_ref()
        .expect("regular domain");
    let builtin = demanded
        .value_binding_origins
        .as_ref()
        .expect("demanded domain");
    assert_eq!(normal.inventory(), builtin.inventory());
    let scope = regular
        .scopes
        .iter()
        .find(|scope| scope.kind == ScopeKind::ScriptSetup)
        .expect("setup")
        .id;
    let reference = ValueReference::new(scope, "alias", Span::new(u32::MAX, u32::MAX));
    assert_eq!(
        normal
            .resolve(&reference)
            .expect("normal binding")
            .canonical,
        builtin
            .resolve(&reference)
            .expect("demanded binding")
            .canonical
    );
}

#[test]
fn actual_uncaptured_refused_and_empty_programs_never_alias_absence() {
    fn captured() -> crate::Croquis {
        parse_script_setup("import { ref } from 'vue'; const value = ref;").into_croquis()
    }
    fn compiled() -> crate::Croquis {
        super::parse_script_setup_for_value_origins::<false>(
            "const value = 1;",
            None,
            false,
            false,
            true,
            false,
            None,
        )
        .into_croquis()
    }
    fn failed() -> crate::Croquis {
        parse_script_setup("const =").into_croquis()
    }
    fn empty_program() -> crate::Croquis {
        parse_script_setup("").into_croquis()
    }
    fn assert_refused(analysis: &crate::Croquis) {
        assert_eq!(analysis.source_program_count, 2);
        if let Some(origins) = analysis.value_binding_origins.as_ref() {
            assert!(!origins.is_complete());
        }
    }
    for other in [compiled, failed, empty_program] {
        let other_program = other();
        assert_eq!(other_program.source_program_count, 1);
        let mut left = captured();
        left.merge_plain_script(other_program);
        assert_refused(&left);

        let mut right = other();
        right.merge_plain_script(captured());
        assert_refused(&right);
    }
    let mut sole = captured();
    sole.merge_plain_script(crate::Croquis::new());
    assert_eq!(sole.source_program_count, 1);
    assert!(
        sole.value_binding_origins
            .as_ref()
            .expect("sole capture")
            .is_complete()
    );

    let mut empty_first = crate::Croquis::new();
    empty_first.merge_plain_script(captured());
    assert_eq!(empty_first.source_program_count, 1);
    assert!(
        empty_first
            .value_binding_origins
            .as_ref()
            .expect("preserved original store")
            .is_complete()
    );

    let mut triple = captured();
    triple.merge_plain_script(compiled());
    triple.merge_plain_script(empty_program());
    assert_refused(&triple);
    triple.merge_plain_script(crate::Croquis::new());
    assert_refused(&triple);

    let mut contradiction = captured();
    contradiction.source_program_count = 0;
    contradiction.merge_plain_script(crate::Croquis::new());
    assert_eq!(contradiction.source_program_count, 0);
    assert!(
        !contradiction
            .value_binding_origins
            .as_ref()
            .expect("contradiction refuses")
            .is_complete()
    );
}
