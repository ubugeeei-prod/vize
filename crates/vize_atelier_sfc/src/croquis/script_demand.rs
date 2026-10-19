//! Result specialization for the same existing script walks.

use vize_croquis::binding_occurrences::BindingOccurrences;
use vize_croquis::script_parser::AuthoredModuleExport;
use vize_croquis::{Croquis, Drawer};

pub(super) trait ScriptDemand {
    type Output;

    fn prepare(drawer: Drawer) -> Drawer;
    fn disabled() -> Self::Output;
    fn empty() -> Self::Output;
    fn finish(drawer: Drawer) -> Self::Output;
    fn merge(plain: Self::Output, setup: Self::Output, offset: u32) -> Self::Output;
}

pub(super) struct OrdinaryScript;

impl ScriptDemand for OrdinaryScript {
    type Output = Croquis;

    #[inline]
    fn prepare(drawer: Drawer) -> Drawer {
        drawer
    }

    #[inline]
    fn disabled() -> Croquis {
        Croquis::new()
    }

    #[inline]
    fn empty() -> Croquis {
        Croquis::new()
    }

    #[inline]
    fn finish(drawer: Drawer) -> Croquis {
        drawer.finish()
    }

    #[inline]
    fn merge(plain: Croquis, mut setup: Croquis, offset: u32) -> Croquis {
        setup.shift_script_offsets(offset);
        setup.merge_plain_script(plain);
        setup
    }
}

pub(super) struct CapturedScript;

pub(super) struct ModuleExportsScript;

impl ScriptDemand for ModuleExportsScript {
    type Output = (Croquis, Option<Vec<AuthoredModuleExport>>);

    fn prepare(drawer: Drawer) -> Drawer {
        drawer.with_empty_module_exports()
    }

    fn disabled() -> Self::Output {
        (Croquis::new(), None)
    }

    fn empty() -> Self::Output {
        (Croquis::new(), Some(Vec::new()))
    }

    fn finish(drawer: Drawer) -> Self::Output {
        drawer.finish_with_module_exports()
    }

    fn merge(
        (plain, plain_exports): Self::Output,
        (setup, setup_exports): Self::Output,
        offset: u32,
    ) -> Self::Output {
        let exports = match (plain_exports, setup_exports) {
            (Some(mut plain), Some(setup)) => {
                for export in setup {
                    if !plain.iter().any(|known| known.name == export.name) {
                        plain.push(export);
                    }
                }
                Some(plain)
            }
            _ => None,
        };
        (OrdinaryScript::merge(plain, setup, offset), exports)
    }
}

impl ScriptDemand for CapturedScript {
    type Output = (Croquis, Option<BindingOccurrences>);

    fn prepare(drawer: Drawer) -> Drawer {
        drawer.with_binding_occurrences()
    }

    fn disabled() -> Self::Output {
        (Croquis::new(), None)
    }

    fn empty() -> Self::Output {
        (Croquis::new(), Some(BindingOccurrences::default()))
    }

    fn finish(drawer: Drawer) -> Self::Output {
        drawer.finish_script_occurrences()
    }

    fn merge(
        (plain, plain_packet): Self::Output,
        (mut summary, setup_packet): Self::Output,
        offset: u32,
    ) -> Self::Output {
        summary.shift_script_offsets(offset);
        summary.merge_plain_script(plain);
        let packet = setup_packet.and_then(|mut setup| {
            setup.shift_script_offsets(offset)?;
            setup.merge(plain_packet?);
            Some(setup)
        });
        (summary, packet)
    }
}

#[cfg(test)]
mod tests {
    use crate::croquis::{
        SfcCroquisOptions, analyze_sfc_descriptor_with_context,
        analyze_sfc_descriptor_with_occurrences,
    };
    use crate::{SfcParseOptions, parse_sfc};

    #[test]
    fn specialized_script_results_preserve_whole_outputs_and_empty_disabled_boundaries() {
        for source in [
            "<style>.a{color:red}</style>",
            "<script>export const plain=1</script>",
            "<script setup>const value=1; const read=value</script>",
            "<script>export const plain=1</script><script setup>const value=plain; const read=value</script>",
        ] {
            let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
            let mut disabled = SfcCroquisOptions::full();
            disabled.analyzer_options.analyze_script = false;
            for options in [
                SfcCroquisOptions::full(),
                SfcCroquisOptions::for_compile(),
                SfcCroquisOptions::lint_demand(),
                SfcCroquisOptions::full().without_script_merge(),
                disabled,
            ] {
                let ordinary = analyze_sfc_descriptor_with_context(&descriptor, None, options);
                let (captured, packet) =
                    analyze_sfc_descriptor_with_occurrences(&descriptor, None, options);
                assert_eq!(ordinary.croquis.to_vir(), captured.croquis.to_vir());
                assert_eq!(
                    serde_json::to_value(ordinary.croquis.semantic_snapshot()).unwrap(),
                    serde_json::to_value(captured.croquis.semantic_snapshot()).unwrap(),
                );
                assert_eq!(ordinary.script_content, captured.script_content);
                assert_eq!(ordinary.script_offset, captured.script_offset);
                if !options.analyzer_options.analyze_script {
                    assert!(packet.is_none());
                } else if descriptor.script.is_none() && descriptor.script_setup.is_none() {
                    let packet = packet.expect("no script retains the complete empty packet");
                    assert_eq!(packet.bindings().count(), 0);
                    assert!(packet.occurrences().is_empty());
                }
            }
        }
    }
}
