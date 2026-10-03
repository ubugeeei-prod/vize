use super::{Allocator, NativeSelectedSfcDomOptions, compile_native_selected_sfc_dom};
use crate::native_selected::NativeSelectedSfcDomError;
use vize_l1_to_l2::native_file::NativeSelectedSfcIssueKind;

#[test]
fn whole_script_style_custom_external_and_profile_refusals_retain_the_original_envelope() {
    for source in [
        "<template><button @click='var x=$event;'/></template><script></script>",
        "<script>/*empty*/</script><template><button @click='var x=$event;'/></template>",
        "<script>export default {}</script><template><button @click='var x=$event;'/></template>",
        "<template><button @click='var x=$event;'/></template><script setup>const x=1;</script>",
        "<script setup lang=ts>const x:number=1;</script><template><button @click='var x=$event;'/></template>",
        "<template><button @click='var x=$event;'/></template><style>.go{color:red}</style>",
        "<style scoped>.go{}</style><template><button @click='var x=$event;'/></template>",
        "<template><button @click='var x=$event;'/></template><style module>.go{}</style>",
        "<template><button @click='var x=$event;'/></template><docs>custom</docs>",
        "<template src='external.vue'></template>",
        "<template><button/></template><script src='external.js'></script>",
        "<template><button/></template><style src='external.css'></style>",
        "<template lang=pug>button(@click='var x=$event;')</template>",
        "<template><button/></template><template><button/></template>",
    ] {
        let arena = Allocator::default();
        let compilation =
            compile_native_selected_sfc_dom(&arena, source, NativeSelectedSfcDomOptions::default());
        assert_eq!(
            compilation.result().err(),
            Some(NativeSelectedSfcDomError::Observation)
        );
        let observation = compilation.observation();
        assert!(observation.admitted().is_none());
        assert!(observation.template().is_none());
        assert!(!observation.issues().is_empty());
        assert!(core::ptr::eq(observation.descriptor().source(), source));
        assert!(!observation.descriptor().container().blocks.is_empty());
        for issue in observation.issues() {
            assert!(
                source
                    .get(issue.span.start as usize..issue.span.end as usize)
                    .is_some()
            );
        }
    }
}

#[test]
fn controls_aliases_nested_text_entities_and_other_handler_families_do_not_gain_product_admission()
{
    for template in [
        "<button v-for='item in [1]' @click='var x=$event;'/>",
        "<button @click='var x=$event;' v-for='(item,index) in [1]'/>",
        "<button v-if=true @click='var x=$event;'/>",
        "<template #default='{ item }'><button @click='var x=$event;'/></template>",
        "<slot @click='var x=$event;'/>",
        "<button @keyup='var x=$event;'/>",
        "<button @click.stop='var x=$event;'/>",
        "<button @click='$event.count++'/>",
        "<button @click='return $event;'/>",
        "<button @click='var x=$event; x.count++;'/>",
        "<button @click='let x:number=1; $event.count++;'/>",
        "<button @click='var x=$event;'> &amp; </button>",
        "<button @click='var x=$event;'>\n text\t\n</button>",
        "<pre><button @click='var x=$event;'/></pre>",
    ] {
        let arena = Allocator::default();
        let source = format!("<!--雪🌸--><template>{template}</template>");
        let compilation = compile_native_selected_sfc_dom(
            &arena,
            &source,
            NativeSelectedSfcDomOptions {
                source_map: true,
                ..NativeSelectedSfcDomOptions::default()
            },
        );
        assert!(compilation.result().is_err(), "{template}");
        let observation = compilation.observation();
        assert!(core::ptr::eq(
            observation.descriptor().source(),
            source.as_str()
        ));
        assert!(
            observation.template().is_some(),
            "original template remains: {template}"
        );
        if observation.issues().is_empty() {
            assert!(observation.admitted().is_some());
            assert!(matches!(
                compilation.result().err(),
                Some(NativeSelectedSfcDomError::Dom(_) | NativeSelectedSfcDomError::Analysis(_))
            ));
        } else {
            assert!(matches!(
                observation.issues()[0].kind,
                NativeSelectedSfcIssueKind::Template(_)
            ));
        }
    }
}

#[test]
fn unavailable_runtime_retains_completed_original_handler_without_partial_output() {
    let arena = Allocator::default();
    let source = "<template><button @click='var x=$event;'/></template>";
    let compilation = compile_native_selected_sfc_dom(
        &arena,
        source,
        NativeSelectedSfcDomOptions {
            runtime_version: "3.6.0",
            ..NativeSelectedSfcDomOptions::default()
        },
    );
    assert!(matches!(
        compilation.result().err(),
        Some(NativeSelectedSfcDomError::Assembly(_))
    ));
    assert!(compilation.observation().admitted().is_some());
    assert!(
        compilation
            .observation()
            .template()
            .unwrap()
            .file()
            .unwrap()
            .is_complete()
    );
}
