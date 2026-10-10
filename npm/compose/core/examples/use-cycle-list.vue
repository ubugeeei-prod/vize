<script setup lang="ts">
import { useId } from "vue";
import { useCycleList } from "@vizejs/composable/use-cycle-list";

const tipId = useId();
const {
  state: tip,
  index,
  list,
  next,
  prev,
} = useCycleList([
  { title: "Name the props", text: "Use clear prop names and document the values they accept." },
  {
    title: "Label the controls",
    text: "Give each form control a label that explains its purpose.",
  },
  { title: "Handle the empty state", text: "Explain what users can do when a list has no items." },
] as const);
</script>

<template>
  <div class="composable-example">
    <p>Explore a short component review checklist. Previous and Next wrap around the list.</p>
    <label :for="tipId">Review tip</label>
    <select :id="tipId" v-model.number="index">
      <option v-for="(item, position) in list" :key="item.title" :value="position">
        {{ item.title }}
      </option>
    </select>
    <output aria-live="polite">Tip {{ index + 1 }} of {{ list.length }}: {{ tip.title }}</output>
    <h3>{{ tip.title }}</h3>
    <p>{{ tip.text }}</p>
    <div class="example-actions">
      <button type="button" @click="() => prev()">Previous tip</button>
      <button type="button" @click="() => next()">Next tip</button>
    </div>
    <p>You can also choose a tip directly. Navigation continues from that choice.</p>
  </div>
</template>
