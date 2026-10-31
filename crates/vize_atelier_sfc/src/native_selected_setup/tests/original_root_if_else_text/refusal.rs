//! Genuine late lower refusals cannot become partial public modules.

use super::*;

pub(super) fn whole_public_output_rejects_late_second_body_failures() -> Result<(), String> {
    // The exact suffix source has an explicit current successor in
    // native_original_root_if_else_composite_inputs_vue_3_5_35.json.
    // Its whole predecessor source/oracle remains bound to c0dc33d29c1c:
    // source ba8e9be1ab3e224084fe38740714b1497d1aeb9e4ded3accd86f105624c7927a,
    // oracle 8ac066dc4efacaf34cc421b8216a66bbf5d98f5863208104d3731044a70f0f4d.
    for body in ["{{right+right}}", "{{missing}}"] {
        let source = cstr!(
            "<script setup>let ready=true;let left='text';let right=2</script><template><p v-if='ready'>{{{{left}}}}</p><b v-else>{body}</b></template>"
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
                    "late second body {body}/{mode:?}/{source_map}: no whole selected acquisition or public output"
                );
            }
        }
    }
    Ok(())
}
