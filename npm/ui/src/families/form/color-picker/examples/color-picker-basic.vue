<!-- Accent color picker with a saturation area, hue and alpha sliders, presets, and a hex field. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  ColorPicker,
  ColorPickerArea,
  ColorPickerChannelSlider,
  ColorPickerField,
  ColorPickerSwatch,
  ColorPickerSwatchGroup,
} from "../color-picker.ts";

const accent = ref("#3366cc");
// The 2D area needs a real size so pointer positions map onto saturation and brightness.
const areaSize = { inlineSize: "200px", blockSize: "140px" };
</script>

<template>
  <div>
    <ColorPicker v-model="accent" name="accent" aria-label="Accent color">
      <ColorPickerArea :style="areaSize" />
      <ColorPickerChannelSlider channel="hue" />
      <ColorPickerChannelSlider channel="alpha" />
      <ColorPickerSwatchGroup aria-label="Brand presets">
        <ColorPickerSwatch value="#3366cc" label="Ocean" />
        <ColorPickerSwatch value="#e4572e" label="Ember" />
        <ColorPickerSwatch value="#2e933c" label="Fern" />
      </ColorPickerSwatchGroup>
      <ColorPickerField aria-label="Hex value" />
    </ColorPicker>
    <output>{{ accent }}</output>
  </div>
</template>
