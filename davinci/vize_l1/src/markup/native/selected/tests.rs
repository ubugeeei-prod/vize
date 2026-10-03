use super::{NativeTemplateComponent, NativeTemplateGrammar};
use crate::container::Vue;
use crate::container::vue::{DescriptorOptions, ScriptRole};
use crate::{SurfaceParseOptions, embed::Lang};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

#[test]
fn selected_template_does_not_promote_script_or_comment_markup() {
    let arena = Allocator::default();
    let source = "<!-- <template><i/></template> --><script lang=ts>const fake='<div/>'</script><template><p a='full value'/></template><script setup lang=ts>const msg=1</script>";
    let descriptor = Vue.observe_descriptor(&arena, source, options());
    let admitted = descriptor.admitted().unwrap();
    let selected = admitted.template().unwrap();
    let original = selected.block();
    let owner = NativeTemplateComponent::parse_in(&arena, admitted)
        .unwrap()
        .unwrap();
    assert_eq!(owner.template_index(), selected.container_index());
    assert_eq!(owner.component().block(), original);
    assert!(core::ptr::eq(
        owner.component().block().root_source(),
        source
    ));
    assert_eq!(owner.grammar(), NativeTemplateGrammar::TypeScriptModule);
    let p = owner.children().next().unwrap().into_element().unwrap();
    assert_eq!(p.surface().tag(), "p");
    assert_eq!(
        p.attributes()
            .next()
            .unwrap()
            .surface()
            .value
            .as_ref()
            .unwrap()
            .content
            .text,
        "full value"
    );
    let ordinary = owner.ordinary().unwrap();
    let setup = owner.setup().unwrap();
    assert!(core::ptr::eq(ordinary.owner(), &owner));
    assert!(core::ptr::eq(setup.owner(), &owner));
    assert_eq!(
        ordinary.container_index(),
        admitted.ordinary().unwrap().container_index()
    );
    assert_eq!(
        setup.container_index(),
        admitted.setup().unwrap().container_index()
    );
    assert_eq!(ordinary.block(), admitted.ordinary().unwrap().block());
    assert_eq!(setup.block(), admitted.setup().unwrap().block());
    assert_eq!(ordinary.role(), ScriptRole::Ordinary);
    assert_eq!(setup.role(), ScriptRole::Setup);
    assert_eq!(ordinary.lang(), Lang::Ts);
    assert_eq!(setup.lang(), Lang::Ts);
}

#[test]
fn scriptless_empty_template_survives_descriptor_drop_and_owner_move() {
    let arena = Allocator::default();
    let source = "<template></template>";
    let owner = {
        let descriptor = Vue.observe_descriptor(&arena, source, options());
        NativeTemplateComponent::parse_in(&arena, descriptor.admitted().unwrap())
            .unwrap()
            .unwrap()
    };
    let moved = core::hint::black_box(owner);
    assert_eq!(moved.grammar(), NativeTemplateGrammar::JavaScriptModule);
    assert!(moved.ordinary().is_none());
    assert!(moved.setup().is_none());
    assert_eq!(moved.children().len(), 0);
    assert_eq!(moved.component().block().source(), "");
    assert!(core::ptr::eq(
        moved.component().block().root_source(),
        source
    ));
    assert_eq!(moved.into_component().into_carrier().tree.children.len(), 0);
}

#[test]
fn absent_template_is_distinct_from_selected_empty_template() {
    let arena = Allocator::default();
    let source = "<script>const fake='<div/>'</script>";
    let descriptor = Vue.observe_descriptor(&arena, source, options());
    let admitted = descriptor.admitted().unwrap();
    assert!(admitted.template().is_none());
    assert!(
        NativeTemplateComponent::parse_in(&arena, admitted)
            .unwrap()
            .is_none()
    );
}

#[test]
fn setup_only_typescript_grammar_is_derived_from_original_role() {
    let arena = Allocator::default();
    let source = "<template><span/></template><script setup lang=ts>const msg: string='x'</script>";
    let descriptor = Vue.observe_descriptor(&arena, source, options());
    let owner = NativeTemplateComponent::parse_in(&arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(owner.grammar(), NativeTemplateGrammar::TypeScriptModule);
    assert!(owner.ordinary().is_none());
    let setup = owner.setup().unwrap();
    assert_eq!(setup.role(), ScriptRole::Setup);
    assert_eq!(setup.lang(), Lang::Ts);
    assert_eq!(setup.block().source(), "const msg: string='x'");
}

#[test]
fn selected_owner_retains_original_recovery_without_certifying_clean_grammar() {
    let arena = Allocator::default();
    let descriptor = Vue.observe_descriptor(&arena, "<template><div></template>", options());
    let owner = NativeTemplateComponent::parse_in(&arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap();
    let div = owner.children().next().unwrap().into_element().unwrap();
    assert_eq!(div.surface().tag(), "div");
    assert!(matches!(div.surface().close, crate::ElementClose::Missing));
    assert_eq!(owner.component().block().source(), "<div>");
}

#[test]
fn ambiguous_or_unsupported_descriptors_cannot_supply_selected_admission() {
    let arena = Allocator::default();
    for source in [
        "<template/><script lang=js></script><script setup lang=ts></script>",
        "<template/><template/>",
        "<template/><script lang='t&#115;'></script>",
        "<template lang=pug>div</template>",
    ] {
        let descriptor = Vue.observe_descriptor(&arena, source, options());
        assert!(descriptor.admitted().is_err(), "{source}");
    }
}
