use super::*;
use vize_l3::decision::dom::DomUnsupported;
use vize_l4::{module::setup::SetupEmitErrorKind, targets::dom::DomErrorKind};

#[test]
fn unsupported_original_programs_and_whole_envelopes_remain_owned_refusals() {
    for source in [
        "<script setup>import value from 'pkg';const count=1</script><template>{{count}}</template>",
        "<script setup>const count=make()</script><template>{{count}}</template>",
        "<script setup>const count=[1]</script><template>{{count}}</template>",
        "<script setup>const count={value:1}</script><template>{{count}}</template>",
        "<script setup lang=ts>let count:1=1</script><template>{{count}}</template>",
        "<script setup lang=ts>declare let count:number</script><template>{{count}}</template>",
        "<script setup>export const count=1</script><template>{{count}}</template>",
        "<script setup>let count=1</script><script>export default{}</script><template>{{count}}</template>",
        "<script setup>let count=1</script><style></style><template>{{count}}</template>",
        "<script setup src='external.ts'/><template>{{count}}</template>",
    ] {
        let arena = Allocator::default();
        let compiled = compile_native_selected_setup_sfc_dom(
            &arena,
            source,
            NativeSelectedSfcDomOptions::default(),
        );
        assert_eq!(
            compiled.result().err(),
            Some(NativeSelectedSetupSfcDomError::Observation),
            "{source}"
        );
        let original = compiled.observation().original();
        assert!(core::ptr::eq(original.descriptor().source(), source));
        assert!(!original.issues().is_empty());
        assert!(compiled.observation().admitted().is_none());
        if let Some(template) = original.template() {
            assert!(
                template.retained_setup().is_some(),
                "pre-cursor program custody: {source}"
            );
            assert!(template.setup().is_err());
        }
    }
}

#[test]
fn compound_escaped_and_outer_handler_reads_never_gain_setup_target_authority() {
    for template in [
        "{{count+1}}",
        r"{{c\u006funt}}",
        "<button @click='let unused=$event;count++;'/>",
    ] {
        let arena = Allocator::default();
        let source = format!("<script setup>let count=1</script><template>{template}</template>");
        let compiled = compile_native_selected_setup_sfc_dom(
            &arena,
            &source,
            NativeSelectedSfcDomOptions::default(),
        );
        assert!(compiled.result().is_err(), "{template}");
        assert!(core::ptr::eq(
            compiled.observation().original().descriptor().source(),
            source.as_str()
        ));
        assert!(
            compiled
                .observation()
                .original()
                .template()
                .unwrap()
                .retained_setup()
                .is_some()
        );
    }
}

#[test]
fn genuine_original_for_custody_is_retained_while_loop_emission_refuses() {
    let arena = Allocator::default();
    let source =
        "<script setup>const items=1</script><template><button v-for='item in items'/></template>";
    let compiled = compile_native_selected_setup_sfc_dom(
        &arena,
        source,
        NativeSelectedSfcDomOptions::default(),
    );
    assert!(compiled.observation().admitted().is_some());
    assert!(matches!(
        compiled.result().err(),
        Some(NativeSelectedSetupSfcDomError::Dom(DomError {
            kind: DomErrorKind::Unsupported(DomUnsupported::Operation),
            ..
        }))
    ));
    let admitted = compiled.observation().admitted().unwrap();
    let file = admitted.setup().file();
    let vize_l2::op::Op::OriginalFor(original) = &file.artifact().root().ops[0] else {
        panic!("actual original For")
    };
    let head = file.for_head_for(original).unwrap();
    assert!(head.resolution().is_some());
    assert!(file.bindings().any(|b| b.template_declaration().is_some()));
}

#[test]
fn generated_collisions_refuse_before_partial_emission_with_whole_program_retained() {
    for name in ["Object", "__value", "__v_raw"] {
        let arena = Allocator::default();
        let source =
            format!("<script setup>const {name}=1</script><template>{{{{{name}}}}}</template>");
        let compiled = compile_native_selected_setup_sfc_dom(
            &arena,
            &source,
            NativeSelectedSfcDomOptions::default(),
        );
        assert!(
            matches!(
                compiled.result().err(),
                Some(NativeSelectedSetupSfcDomError::Setup(SetupEmitError {
                    kind: SetupEmitErrorKind::GeneratedBindingCollision,
                    ..
                }))
            ),
            "{name}"
        );
        assert!(compiled.observation().admitted().is_some());
        assert!(
            compiled
                .observation()
                .original()
                .template()
                .unwrap()
                .retained_setup()
                .is_some()
        );
    }
}

#[test]
fn unavailable_runtime_retains_complete_actual_setup_and_file_without_output() {
    let arena = Allocator::default();
    let source = "<script setup>const count=1</script><template>{{count}}</template>";
    let compiled = compile_native_selected_setup_sfc_dom(
        &arena,
        source,
        NativeSelectedSfcDomOptions {
            runtime_version: "3.6.0",
            ..NativeSelectedSfcDomOptions::default()
        },
    );
    assert!(matches!(
        compiled.result().err(),
        Some(NativeSelectedSetupSfcDomError::Assembly(_))
    ));
    let admitted = compiled.observation().admitted().unwrap();
    assert!(admitted.setup().file().is_complete());
    assert!(admitted.setup().syntax().diagnostics().count() == 0);
}
