export const colorPickerRendererFixtures = [
  {
    filename: "ColorPickerConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  ColorPickerArea,
  ColorPickerChannelSlider,
  ColorPickerEyeDropper,
  ColorPickerField,
  ColorPickerRoot,
  ColorPickerSwatch,
  ColorPickerSwatchGroup,
} from "./families/form/color-picker/color-picker.ts";

const color = ref("#3366cc");
const presets = ["#3366cc", "#ff0000", "#00aa55"];
</script>

<template>
  <ColorPickerRoot v-model="color" format="hex" name="accent">
    <template #default="{ value, state }">
      <ColorPickerArea aria-label="Tone">
        <template #default="{ xPercent, yPercent }">{{ xPercent }} {{ yPercent }}</template>
      </ColorPickerArea>
      <ColorPickerChannelSlider channel="hue" />
      <ColorPickerChannelSlider channel="alpha" orientation="vertical" />
      <ColorPickerSwatchGroup aria-label="Presets">
        <ColorPickerSwatch v-for="preset in presets" :key="preset" :value="preset">
          <template #default="{ checked }">{{ checked ? "selected" : "" }}</template>
        </ColorPickerSwatch>
      </ColorPickerSwatchGroup>
      <ColorPickerField aria-label="Hex" />
      <ColorPickerEyeDropper aria-label="Pick from screen" unsupported="hide" />
      <output :data-state="state">{{ value }}</output>
    </template>
  </ColorPickerRoot>
</template>
`,
  },
] as const;
