<script lang="ts">
import { ref } from "vue";

const dragging = ref(false);
let dropCallback: null | (() => void) = null;

function resetDragging() {
  dragging.value = false;
}
</script>

<script setup lang="ts">
const items = [{ id: "alpha" }, { id: "beta" }];

function startDrag() {
  dragging.value = true;
  dropCallback = resetDragging;
}
</script>

<template>
  <div :class="{ dragging }">
    <button v-for="item in items" :key="item.id" @click="startDrag">
      {{ item.id }}
    </button>
    <slot name="state" :dragging="dragging" />
  </div>
</template>
