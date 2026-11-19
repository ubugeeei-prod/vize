use super::support::options;
use vize_l0::Allocator;
use vize_l1_to_l2::native_file::{NativeSfcIssueKind, NativeTemplateOutcome, lower_sfc_native};
use vize_l1_to_l2::vue_file::VueFileIssueKind;

#[test]
fn actual_uv_recovery_retains_diagnostics_and_does_not_gain_admission() {
    let arena = Allocator::default();
    let source = "<script>const invalid = /uv;</script><template>kept</template>";
    let observed = lower_sfc_native(&arena, source, options());
    assert!(observed.admitted().is_none());
    let script = observed.scripts().first().unwrap();
    let syntax = script.syntax().unwrap();
    assert!(syntax.program().is_none());
    assert!(syntax.admitted_program().is_none());
    assert!(syntax.diagnostics().next().is_some());
    assert_eq!(syntax.source().text(), "const invalid = /uv;");
    assert!(script.unit().is_none());
    assert!(observed.template().unwrap().produced().is_some());
    assert!(
        observed
            .issues()
            .iter()
            .any(|issue| matches!(issue.kind, NativeSfcIssueKind::ScriptSyntax(_)))
    );
}

#[test]
fn invalid_ordinary_does_not_discard_later_valid_setup_or_template() {
    let arena = Allocator::default();
    let source = "<script>const invalid = /uv;</script><script setup>const value = 1;</script><template>{{value}}</template>";
    let observed = lower_sfc_native(&arena, source, options());
    assert!(observed.admitted().is_none());
    let mut scripts = observed.scripts().iter();
    assert!(
        scripts
            .next()
            .unwrap()
            .syntax()
            .unwrap()
            .admitted_program()
            .is_none()
    );
    assert!(
        scripts
            .next()
            .unwrap()
            .syntax()
            .unwrap()
            .admitted_program()
            .is_some()
    );
    assert_eq!(observed.descriptor().container().blocks.len(), 3);
    let partial = observed.file().unwrap();
    assert!(partial.ordinary().is_none());
    assert!(partial.setup().is_some());
    assert_eq!(partial.file().units().len(), 1);
    assert!(observed.template().unwrap().produced().is_some());
}

#[test]
fn setup_export_preserves_file_facts_and_original_unconstructed_template() {
    let arena = Allocator::default();
    let source = "<script setup>export const value = 1;</script><template>{{value}}</template>";
    let observed = lower_sfc_native(&arena, source, options());
    assert!(observed.admitted().is_none());
    let rejected = observed.rejected_file().unwrap();
    assert!(
        rejected
            .issues()
            .iter()
            .any(|issue| issue.kind == VueFileIssueKind::SetupExport)
    );
    assert_eq!(rejected.file().unwrap().units().len(), 1);
    let NativeTemplateOutcome::FactoryRefused { component, .. } =
        observed.template().unwrap().outcome()
    else {
        panic!("expected retained original template owner");
    };
    assert_eq!(component.block().source(), "{{value}}");
    assert_eq!(component.carrier().tree.children.len(), 1);
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
}

#[test]
fn unadmitted_macro_remains_typed_and_keeps_original_parser_owner() {
    let arena = Allocator::default();
    let source = "<script setup>const props = defineProps({value: Number});</script><template>kept</template>";
    let observed = lower_sfc_native(&arena, source, options());
    assert!(observed.admitted().is_none());
    assert!(
        observed
            .rejected_file()
            .unwrap()
            .issues()
            .iter()
            .any(|issue| matches!(issue.kind, VueFileIssueKind::UnsupportedCall { .. }))
    );
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
    assert!(matches!(
        observed.template().unwrap().outcome(),
        NativeTemplateOutcome::FactoryRefused { .. }
    ));
}

#[test]
fn descriptor_refusal_preserves_blocks_without_parsing_untrusted_children() {
    let arena = Allocator::default();
    for source in [
        "<script>const value = 1;</script><template>kept</template><custom>original</custom>",
        "<script>const value = 1;",
    ] {
        let observed = lower_sfc_native(&arena, source, options());
        assert!(observed.admitted().is_none());
        assert!(observed.descriptor().admitted().is_err());
        assert!(!observed.descriptor().container().blocks.is_empty());
        assert!(observed.scripts().is_empty());
        assert!(observed.template().is_none());
        assert!(observed.file().is_none());
        assert!(observed.rejected_file().is_none());
    }
}

#[test]
fn opaque_style_custody_does_not_discard_original_script_and_template_owners() {
    let arena = Allocator::default();
    let source = "<style lang=scss scoped module=theme>p{color:v-bind(color)}</style><script>const value=1</script><template><p/></template><style>raw &amp;</style>";
    let observed = lower_sfc_native(&arena, source, options());
    let descriptor = observed.descriptor().admitted().unwrap();
    assert_eq!(descriptor.styles().len(), 2);
    assert!(observed.admitted().is_some());
    assert_eq!(observed.scripts().len(), 1);
    let script = &observed.scripts()[0];
    assert_eq!(script.container_index(), 1);
    assert!(script.syntax().unwrap().admitted_program().is_some());
    assert!(core::ptr::eq(script.block().root_source(), source));
    let template = observed.template().unwrap();
    assert!(core::ptr::eq(
        template.component().unwrap().block().root_source(),
        source
    ));
    let file = observed.file().unwrap().file();
    assert!(core::ptr::eq(file.artifact().source(), source));
    for index in [0, 3] {
        let style = descriptor
            .styles()
            .find(|style| style.container_index() == index)
            .unwrap();
        assert!(core::ptr::eq(
            style.original_block(),
            &observed.descriptor().container().blocks[index]
        ));
    }
}

#[test]
fn component_and_control_refusals_retain_real_template_and_partial_file() {
    let arena = Allocator::default();
    for source in [
        "<template><Card>kept</Card></template>",
        "<template><div v-if=\"true\">kept</div></template>",
    ] {
        let observed = lower_sfc_native(&arena, source, options());
        assert!(observed.admitted().is_none());
        assert!(observed.descriptor().admitted().is_ok());
        let produced = observed.template().unwrap().produced().unwrap();
        assert!(!produced.component.carrier().tree.children.is_empty());
        assert!(observed.rejected_file().is_some() || !produced.holes.is_empty());
    }
}
