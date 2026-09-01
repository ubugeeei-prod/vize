//! P2-11 residual formatter/helper-order witnesses reduced from the S2 DOM
//! corpus tail.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
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
    ),
    (
        "keyed_v_for_with_v_show_keeps_multiline_key_props",
        r#"
<div v-for="f in fields" v-show="visible(f)" :key="f.fieldname">
  <div v-if="f.fieldtype !== 'Check'">{{ f.label }}</div>
  <FormControl v-model="model[f.fieldname]" />
</div>
"#,
    ),
    (
        "keyed_v_for_with_conditional_native_models_has_no_trailing_fragment_comma",
        r#"
<section>
  <div v-for="colorName in Object.keys(customColors)" :key="colorName">
    <input
      v-if="isColor(colorName)"
      :id="`color-input-${colorName}`"
      type="color"
      :value="customColors[colorName]"
      @input="customColors[colorName] = $event.target.value"
    />
    <input
      v-else
      :id="`color-input-${colorName}`"
      v-model="customColors[colorName]"
      @input="setVariable(colorName, customColors[colorName])"
    />
  </div> <!-- End of color list -->
</section>
"#,
    ),
    (
        "filler_component_before_looped_component_slots_keeps_walk_in_sync",
        r#"
<TransitionGroup tag="ul">
  <Item v-if="empty" key="empty">&nbsp;</Item>
  <Item v-for="msg in messages" :key="msg.id" class="py-2">
    <Badge :variant="msg.kind">{{ msg.kind }}</Badge>
    <span :class="[`text-${msg.kind}`]">{{ msg.text }}</span>
  </Item>
</TransitionGroup>
"#,
    ),
];

#[test]
fn s2_format_residuals_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
