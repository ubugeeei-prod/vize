//! Small type-checking oracles for patterns that surfaced while trialing Vize
//! in VOICEVOX.

#[path = "support/script_block_project.rs"]
mod project;

const SLIDER: &str = r#"<script setup lang="ts">
defineProps<{
  disable: boolean;
  modelValue: number | null;
}>();
defineEmits<{
  "update:modelValue": [value: number | null];
  click: [];
  change: [];
}>();
</script>
<template><div /></template>
"#;

const AUDIO_ACCENT: &str = r#"<script setup lang="ts">
import { ref } from "vue";
import Slider from "./Slider.vue";

function updateAccent(value: number | null): void {
  void value;
}

function stopPropagation(): void {}

const previewAccentSlider = {
  qSliderProps: {
    disable: ref(false),
    modelValue: ref<number | null>(0),
    "onUpdate:modelValue": updateAccent,
    onChange: () => {},
  },
};
</script>

<template>
  <Slider
    :disable="previewAccentSlider.qSliderProps.disable.value"
    :modelValue="previewAccentSlider.qSliderProps.modelValue.value"
    @update:modelValue="
      previewAccentSlider.qSliderProps['onUpdate:modelValue'] as (
        value: number | null,
      ) => void
    "
    @click.stop="stopPropagation"
    @change="previewAccentSlider.qSliderProps.onChange"
  />
</template>
"#;

const ENGINE_MANAGE_DIALOG: &str = r#"<script setup lang="ts">
type FeatureFlags = Record<string, boolean>;

const selectedId = "engine-a";
const engineManifests: Record<string, { supportedFeatures?: FeatureFlags }> = {
  "engine-a": {
    supportedFeatures: {
      morphing: true,
      singing: false,
    },
  },
};

function featureClass(enabled: boolean): string {
  return enabled ? "" : "text-warning";
}
</script>

<template>
  <ul>
    <li
	      v-for="(value, feature) in engineManifests[selectedId].supportedFeatures != null
	        ? engineManifests[selectedId].supportedFeatures
	        : null"
	      :key="feature"
	      :data-feature="feature.toUpperCase()"
	      :class="featureClass(value)"
	    >
      {{ feature }}
    </li>
  </ul>
</template>
"#;

#[test]
fn update_model_handler_type_assertions_accept_nullable_payloads() {
    let rows = project::check(&[
        ("src/AudioAccent.vue", AUDIO_ACCENT),
        ("src/Slider.vue", SLIDER),
    ]);
    assert!(
        rows.is_empty(),
        "nullable update:modelValue handler assertions should stay diagnostic-free: {rows:#?}"
    );
}

#[test]
fn nullable_object_iteration_preserves_key_and_value_aliases() {
    let rows = project::check(&[("src/EngineManageDialog.vue", ENGINE_MANAGE_DIALOG)]);
    assert!(
        rows.is_empty(),
        "nullable object v-for sources should keep feature/value aliases typed: {rows:#?}"
    );
}
