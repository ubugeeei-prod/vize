<script setup lang="ts">
import ChildPanel from "./ChildPanel.vue";

const { tone = "neutral" } = defineProps<{ tone?: "neutral" | "accent" }>();
const emit = defineEmits<{ dismiss: [] }>();
</script>

<template>
  <div>
    <ChildPanel :title="`tone-${tone}`" @close="() => emit('dismiss')" />
    <component :is="ChildPanel" :title="`tone-${tone}`" />

    <!-- not reported: the same expressions on a native element -->
    <section :title="`tone-${tone}`" @click="() => emit('dismiss')"></section>

    <!-- not reported: plain bindings on the component -->
    <ChildPanel :title="tone" @close="emit('dismiss')" />
  </div>
</template>
