use super::support::options;
use vize_l0::Allocator;
use vize_l1::container::vue::ScriptRole;
use vize_l1_to_l2::native_file::lower_sfc_native;
use vize_l1_to_l2::vue_file::VueScriptRole;
use vize_l2::file::Namespace;

#[test]
fn setup_only_and_script_only_need_no_synthetic_ordinary_or_template() {
    let arena = Allocator::default();
    let source = "<script setup>const value = 1;</script>";
    let observed = lower_sfc_native(&arena, source, options());
    let native = observed.admitted().unwrap();
    assert!(native.observation().template().is_none());
    assert!(native.file().ordinary().is_none());
    assert_eq!(observed.scripts().len(), 1);
    let script = observed.scripts().first().unwrap();
    assert_eq!(script.role(), ScriptRole::Setup);
    assert_eq!(script.container_index(), 0);
    let binding = native.file().file().bindings().next().unwrap();
    assert_eq!(
        native.file().exposure(binding).unwrap().role(),
        VueScriptRole::Setup
    );
}

#[test]
fn ordinary_only_is_real_script_file_without_setup_exposure() {
    let arena = Allocator::default();
    let source = "<script>const value = 1;</script>";
    let observed = lower_sfc_native(&arena, source, options());
    let native = observed.admitted().unwrap();
    assert!(observed.template().is_none());
    assert!(native.file().setup().is_none());
    let binding = native.file().file().bindings().next().unwrap();
    assert_eq!(binding.declaration().unwrap().name.as_str(), "value");
    assert!(native.file().exposure(binding).is_none());
}

#[test]
fn reversed_authored_order_preserves_indices_and_constructs_ordinary_first() {
    let arena = Allocator::default();
    let source = "<script setup>const value = ordinary;</script><script>const ordinary = 1;</script><template>{{value}}</template>";
    let observed = lower_sfc_native(&arena, source, options());
    let native = observed.admitted().unwrap();
    let mut scripts = observed.scripts().iter();
    let ordinary = scripts.next().unwrap();
    let setup = scripts.next().unwrap();
    assert_eq!(ordinary.role(), ScriptRole::Ordinary);
    assert_eq!(ordinary.container_index(), 1);
    assert_eq!(setup.role(), ScriptRole::Setup);
    assert_eq!(setup.container_index(), 0);
    assert_eq!(
        ordinary.unit(),
        Some(native.file().ordinary().unwrap().unit())
    );
    assert_eq!(setup.unit(), Some(native.file().setup().unwrap().unit()));
    assert_ne!(
        native.file().ordinary().unwrap().scope(),
        native.file().setup().unwrap().scope()
    );
    let bindings: Vec<_> = native.file().file().bindings().collect();
    assert_eq!(bindings.len(), 2);
    for binding in bindings {
        assert!(native.file().exposure(binding).is_some());
    }
    assert!(!native.file().file().references().is_empty());
}

#[test]
fn ordinary_only_template_local_refuses_without_minting_a_fake_binding_node() {
    let arena = Allocator::default();
    let source = "<script>const value = 1;</script><template>{{value}}</template>";
    let observed = lower_sfc_native(&arena, source, options());
    assert!(observed.admitted().is_none());
    assert!(
        observed
            .scripts()
            .first()
            .unwrap()
            .syntax()
            .unwrap()
            .admitted_program()
            .is_some()
    );
    let produced = observed.template().unwrap().produced().unwrap();
    assert_eq!(produced.embeds.len(), 1);
    let embed = produced.embeds.first().unwrap();
    assert!(embed.node.is_none());
    assert!(embed.syntax.expression().is_some());
    let partial = observed.rejected_file().unwrap().file().unwrap();
    assert!(!partial.template_issues().is_empty());
    assert_eq!(partial.units().len(), 1);
}

#[test]
fn type_only_setup_identity_never_becomes_runtime_exposure() {
    let arena = Allocator::default();
    let source = "<script setup lang=ts>import type { T } from 'types'; const value = 1;</script><template>{{value}}</template>";
    let observed = lower_sfc_native(&arena, source, options());
    let native = observed.admitted().unwrap();
    let mut found_type = false;
    let mut found_value = false;
    for binding in native.file().file().bindings() {
        let declaration = binding.declaration().unwrap();
        if declaration.namespace == Namespace::Type {
            found_type = true;
            assert_eq!(declaration.name.as_str(), "T");
            assert!(native.file().exposure(binding).is_none());
        } else {
            found_value = true;
            assert_eq!(declaration.name.as_str(), "value");
            assert!(native.file().exposure(binding).is_some());
        }
    }
    assert!(found_type && found_value);
}

#[test]
fn empty_template_and_static_template_without_scripts_are_real_files() {
    let arena = Allocator::default();
    for source in [
        "<template></template>",
        "<template><div>日本語</div></template>",
    ] {
        let observed = lower_sfc_native(&arena, source, options());
        let native = observed.admitted().unwrap();
        assert!(observed.scripts().is_empty());
        assert!(native.file().file().units().is_empty());
        let produced = observed.template().unwrap().produced().unwrap();
        assert!(produced.is_supported());
        assert!(core::ptr::eq(
            produced.component.block().root_source(),
            source
        ));
    }
}

#[test]
fn template_call_retains_genuine_exposed_reference_facts_without_script_call_admission() {
    let arena = Allocator::default();
    let source = "<script setup>import { run } from 'dep'; const value = 1;</script><template>{{run(value)}}</template>";
    let observed = lower_sfc_native(&arena, source, options());
    let native = observed.admitted().unwrap();
    let produced = observed.template().unwrap().produced().unwrap();
    let embed = produced.embeds.first().unwrap();
    let resolution = native
        .file()
        .file()
        .expression(embed.node.unwrap())
        .unwrap();
    let table = resolution.table().unwrap();
    assert!(core::ptr::eq(
        table.expression().ast,
        embed.syntax.expression().unwrap()
    ));
    let mut names = Vec::new();
    for occurrence in table.occurrences() {
        let binding = native.file().file().binding(occurrence.binding).unwrap();
        names.push(binding.declaration().unwrap().name.as_str());
        assert!(native.file().exposure(binding).is_some());
        assert!(resolution.accepts(binding));
    }
    assert_eq!(names, ["run", "value"]);
}
