export const formInputRendererFixtures = [
  {
    filename: "MaskedInputConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  MaskedInput,
  defineInputMaskTokens,
  useInputMask,
} from "./families/form/input-mask/input-mask.ts";

const phone = ref("");
const hex = defineInputMaskTokens({
  H: { pattern: /[0-9a-f]/i, transform: (character: string) => character.toUpperCase() },
});
const color = useInputMask({ mask: "#HHHHHH", tokens: hex });
</script>

<template>
  <MaskedInput v-model="phone" mask="(999) 999-9999" aria-label="Phone" eager />
  <input
    aria-label="Color"
    :value="color.masked.value"
    :inputmode="color.inputMode.value"
    @input="color.handleInput"
  />
  <output>{{ phone }}</output>
</template>
`,
  },
  {
    filename: "NumberFieldConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  NumberField,
  NumberFieldDecrement,
  NumberFieldIncrement,
  NumberFieldInput,
} from "./families/form/number-field/number-field.ts";

const price = ref<number | null>(12.5);
</script>

<template>
  <NumberField
    v-model="price"
    aria-label="Price"
    name="price"
    :min="0"
    :step="0.5"
    locale="de-DE"
    :format-options="{ style: 'currency', currency: 'EUR' }"
  >
    <template #default="{ formattedValue, canIncrement }">
      <NumberFieldDecrement />
      <NumberFieldInput placeholder="0,00 €" />
      <NumberFieldIncrement v-slot="{ holding }">{{ holding ? "+++" : "+" }}</NumberFieldIncrement>
      <output :data-can-increment="canIncrement">{{ formattedValue }}</output>
    </template>
  </NumberField>
</template>
`,
  },
  {
    filename: "RangeSliderConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  RangeSlider,
  RangeSliderRange,
  RangeSliderThumb,
  RangeSliderTrack,
} from "./families/form/range-slider/range-slider.ts";

const price = ref<readonly number[]>([20, 80]);
const labels = ["Minimum price", "Maximum price"];
</script>

<template>
  <RangeSlider v-model="price" aria-label="Price" name="price" :min-steps-between-thumbs="2">
    <template #default="{ values }">
      <RangeSliderTrack>
        <RangeSliderRange />
        <RangeSliderThumb
          v-for="(value, index) in values"
          :key="index"
          v-slot="{ active }"
          :index="index"
          :aria-label="labels[index]"
        >
          <output :data-active="active">{{ value }}</output>
        </RangeSliderThumb>
      </RangeSliderTrack>
    </template>
  </RangeSlider>
</template>
`,
  },
] as const;
