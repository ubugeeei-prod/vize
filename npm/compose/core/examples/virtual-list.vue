<script setup lang="ts">
import { computed, ref, useId, watch } from "vue";
import { useVirtualList } from "@vizejs/composable/virtual-list";

const query = ref("");
const searchId = useId();
const scrollHelpId = useId();
const guides = Array.from({ length: 1_000 }, (_, index) => ({
  id: index + 1,
  title: `Guide ${String(index + 1).padStart(4, "0")}`,
}));
const matches = computed(() =>
  guides.filter((guide) => guide.title.toLowerCase().includes(query.value.trim().toLowerCase())),
);
const { list, containerProps, wrapperProps, scrollTo } = useVirtualList(matches, {
  itemSize: 40,
  overscan: 2,
  initialItemCount: 7,
});
// A new search starts at its first result, without moving keyboard focus.
watch(query, () => scrollTo(0), { flush: "post" });
</script>

<template>
  <div class="composable-example">
    <label :for="searchId">Find a guide</label>
    <input :id="searchId" v-model="query" type="search" placeholder="Try 099 or missing" />
    <div class="example-actions">
      <button type="button" :disabled="matches.length === 0" @click="() => scrollTo(0)">
        First result
      </button>
      <button
        type="button"
        :disabled="matches.length === 0"
        @click="() => scrollTo(matches.length - 1)"
      >
        Last result
      </button>
    </div>
    <output aria-live="polite">{{ matches.length }} matching guides</output>
    <p :id="scrollHelpId">Focus the list and use Arrow keys, Page Down, Home or End.</p>
    <div
      v-bind="containerProps"
      role="region"
      aria-label="Guide catalogue"
      :aria-describedby="scrollHelpId"
      tabindex="0"
      class="guide-catalogue"
    >
      <div v-bind="wrapperProps" role="list" aria-label="Matching guides">
        <div
          v-for="row in list"
          :key="row.data.id"
          role="listitem"
          :aria-posinset="row.index + 1"
          :aria-setsize="matches.length"
          class="guide-row"
        >
          {{ row.data.title }}
        </div>
      </div>
    </div>
    <p v-if="matches.length === 0">No guides match. Try a different search.</p>
    <p v-else>
      {{ list.length }} rows rendered · positions {{ (list[0]?.index ?? 0) + 1 }}–{{
        (list.at(-1)?.index ?? 0) + 1
      }}
      of {{ matches.length }}
    </p>
  </div>
</template>

<style scoped>
.guide-catalogue {
  height: 200px;
  border: 1px solid #888;
  border-radius: 6px;
}
.guide-row {
  height: 40px;
  box-sizing: border-box;
  padding: 8px 12px;
  border-bottom: 1px solid #8884;
}
</style>
