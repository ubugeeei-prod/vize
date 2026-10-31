//! Unsupported composite families retain whole public lower refusals.
use super::*;
pub(super) fn whole_public_output_rejects_late_second_body_failures() -> Result<(), String> {
    for body in [
        "{{right+right}}",
        "{{missing}}",
        "x{{right}}y",
        "{{right}}{{right}}",
        "x<!--kept-->{{right}}",
    ] {
        let source = cstr!(
            "<script setup>let ready=true;let left='text';let right=2</script><template><p v-if='ready'>pre{{{{left}}}}</p><b v-else>{body}</b></template>"
        );
        for mode in [DomMode::Development, DomMode::Production] {
            for source_map in [false, true] {
                let arena = Allocator::default();
                let compiled = compile_native_selected_setup_sfc_dom_with_mode(
                    &arena,
                    &source,
                    NativeSelectedSfcDomOptions {
                        source_map,
                        ..NativeSelectedSfcDomOptions::default()
                    },
                    mode,
                );
                require!(
                    matches!(
                        compiled.result(),
                        Err(NativeSelectedSetupSfcDomError::Observation)
                    ) && compiled.observation().admitted().is_none(),
                    "late composite {body}/{mode:?}/{source_map}: no selected whole output"
                );
            }
        }
    }
    Ok(())
}
