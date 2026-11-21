use super::*;
use crate::container::{Vue, vue::DescriptorOptions};
use crate::markup::SurfaceParseOptions;
use vize_l0::config::{VueDialect, VueVersion};

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

#[test]
fn each_role_retains_its_own_original_block_and_can_be_observed_independently() {
    let arena = Allocator::default();
    let source = "<template>雪</template><script setup lang='ts'>/* α */ const x=1</script><script lang='ts'>/* β */ const x=1</script>";
    let owner = Vue.observe_descriptor(&arena, source, options());
    let descriptor = owner.admitted().unwrap();
    descriptor.with_script_programs(&arena, |programs| {
        let setup = programs.get(ScriptRole::Setup).unwrap();
        let ordinary = programs.get(ScriptRole::Ordinary).unwrap();
        assert!(!core::ptr::eq(ordinary, setup));
        for (role, selected, observed, comment) in [
            (
                ScriptRole::Setup,
                descriptor.setup().unwrap(),
                setup,
                "/* α */",
            ),
            (
                ScriptRole::Ordinary,
                descriptor.ordinary().unwrap(),
                ordinary,
                "/* β */",
            ),
        ] {
            let original = selected.block();
            let single = selected.observe_program(&arena);
            assert!(core::ptr::eq(
                observed.source().text(),
                single.source().text()
            ));
            assert_eq!(observed.hole(), single.hole());
            assert!(core::ptr::eq(observed, programs.get(role).unwrap()));
            assert!(core::ptr::eq(observed.source().text(), original.source()));
            assert!(core::ptr::eq(observed.source().authored_root(), source));
            assert_eq!(observed.source().span(), original.span());
            assert_eq!(
                observed.source_type(),
                ProgramOptions::module(selected.lang()).source_type()
            );
            let comment_view = observed.comments().next().unwrap();
            assert_eq!(comment_view.text().unwrap(), comment);
            assert_eq!(comment_view.authored_span().unwrap().slice(source), comment);
            assert!(observed.admitted_program().is_some());
        }
    });
}

#[test]
fn absence_recovery_and_equal_text_keep_their_original_identity() {
    let arena = Allocator::default();
    let source = "<script>const broken = (;</script>";
    let owner = Vue.observe_descriptor(&arena, source, options());
    owner
        .admitted()
        .unwrap()
        .with_script_programs(&arena, |programs| {
            assert!(programs.get(ScriptRole::Setup).is_none());
            let rejected = programs.get(ScriptRole::Ordinary).unwrap();
            assert!(rejected.hole().is_some() && rejected.admitted_program().is_none());
            assert!(rejected.diagnostics().next().is_some());
            assert!(core::ptr::eq(
                rejected,
                programs.get(ScriptRole::Ordinary).unwrap()
            ));
        });
    let first =
        alloc::string::String::from("<script>const x=1</script><script setup>const x=1</script>");
    let second = first.clone();
    let owners = [
        Vue.observe_descriptor(&arena, &first, options()),
        Vue.observe_descriptor(&arena, &second, options()),
    ];
    let [left, right] = owners.each_ref().map(|d| d.admitted().unwrap());
    left.with_script_programs(&arena, |a| {
        right.with_script_programs(&arena, |b| {
            let script = a.get(ScriptRole::Ordinary).unwrap();
            let twin_role = a.get(ScriptRole::Setup).unwrap();
            let twin_file = b.get(ScriptRole::Ordinary).unwrap();
            assert_eq!(script.source().text(), twin_role.source().text());
            assert_ne!(script.source().span(), twin_role.source().span());
            assert_eq!(script.source().span(), twin_file.source().span());
            assert!(!core::ptr::eq(
                script.source().text(),
                twin_file.source().text()
            ));
        })
    });
}
