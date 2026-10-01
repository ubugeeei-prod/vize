//! Existing prop and slot values agree with the previous Croquis projection.

mod corpus;
mod edits;
mod legacy;

use super::{component_metadata_from_interface, export_component_interface};
use vize_l2::summary::SfcSummary;

#[test]
fn alpha_metadata_preserves_full_prop_and_slot_contracts() {
    let samples = [
        "<script setup lang='ts'>withDefaults(defineProps<{ label: 'a b'; count?: number; active?: boolean }>(), { count: 1, active: true })</script><template><slot name='header' /></template>",
        "<script setup lang='ts'>const label = defineModel<string>('title', { required: true }); defineSlots<{ default(props: { item: string }): any; footer(props: { id: number }): any }>()</script><template><slot /><slot name='header'/></template>",
        "<script>export default { props: { size: { type: String, required: true }, disabled: Boolean } }</script><template><slot name='body'/></template>",
        "<script setup lang='ts' generic='T extends string'>defineProps<{ value: T; name?: string }>(); const privateRef = ref(0)</script><template><slot :name='name'/><slot name='body'/></template>",
        "<script setup lang='ts'>interface Props { a: string; z?: number }; defineProps<Props>()</script>",
    ];
    for source in samples {
        for (options_api, legacy_vue2) in [(false, false), (true, false), (true, true)] {
            let descriptor = vize_resident::parse_descriptor("Widget.vue", source).unwrap();
            let pages =
                export_component_interface(&descriptor, "Widget.vue", options_api, legacy_vue2)
                    .unwrap();
            let summary = SfcSummary::from_alpha(pages).unwrap();
            let projected = component_metadata_from_interface(
                &vize_resident::ComponentSurface::from_summary(&summary),
            )
            .unwrap();
            let expected =
                legacy::legacy_metadata(Some(&descriptor), "Widget.vue", options_api, legacy_vue2);
            assert_eq!(projected.props, expected.props, "{source}; props");
            assert_eq!(projected.slots, expected.slots, "{source}; slots");
        }
    }
}
