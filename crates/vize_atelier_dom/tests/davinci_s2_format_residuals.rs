//! P2-11 residual formatter/helper-order witnesses reduced from the S2 DOM
//! corpus tail.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[(
    "directus_image_editor_slots_before_show_helper",
    r#"
<VDrawer v-model="internalActive" class="modal">
  <template #activator="activatorBinding">
    <slot name="activator" v-bind="activatorBinding" />
  </template>
  <template #title-outer:append>
    <VIcon v-tooltip.bottom="$t('changes_are_permanent')" name="error" />
  </template>
  <VMenu>
    <template #activator="{ toggle }">
      <VIcon :name="aspectRatioIcon" clickable @click="toggle" />
    </template>
    <VList>
      <template v-if="customAspectRatios">
        <VListItem
          v-for="customAspectRatio in customAspectRatios"
          :key="customAspectRatio.text"
          clickable
          :active="aspectRatio === customAspectRatio.value"
        >
          {{ customAspectRatio.text }}
        </VListItem>
      </template>
    </VList>
  </VMenu>
  <button v-show="cropping" type="button" @click="cropping = false">
    {{ localDragMode === 'focal_point' ? $t('cancel_selection') : $t('cancel_crop') }}
  </button>
  <template #actions:primary>
    <PrivateViewHeaderBarActionButton :loading="saving">
      <template v-if="props.createAllowed" #split-menu>
        <VList>
          <VListItem clickable @click="saveAsNew" />
        </VList>
      </template>
    </PrivateViewHeaderBarActionButton>
  </template>
</VDrawer>
"#,
)];

#[test]
fn s2_format_residuals_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
