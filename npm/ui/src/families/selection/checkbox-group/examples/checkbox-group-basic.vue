<!-- Typed topping selection with a tri-state select-all parent, bound with v-model. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { CheckboxGroup, CheckboxGroupItem, CheckboxGroupSelectAll } from "../checkbox-group.ts";

const toppings = ["Mushrooms", "Olives", "Peppers", "Pineapple"];
const selected = ref<readonly string[]>(["Olives"]);
const baseId = useId();
</script>

<template>
  <div>
    <span :id="`${baseId}-legend`">Pizza toppings</span>
    <CheckboxGroup
      v-model="selected"
      :options="toppings"
      name="toppings"
      :aria-labelledby="`${baseId}-legend`"
      :is-option-disabled="(topping) => topping === 'Pineapple'"
    >
      <CheckboxGroupSelectAll :id="`${baseId}-all`" />
      <label :for="`${baseId}-all`">Select all</label>
      <template v-for="(topping, index) in toppings" :key="topping">
        <CheckboxGroupItem :id="`${baseId}-${index}`" :value="topping" />
        <label :for="`${baseId}-${index}`">{{ topping }}</label>
      </template>
    </CheckboxGroup>
    <output>{{ selected.length ? selected.join(", ") : "No toppings" }}</output>
  </div>
</template>
