<script setup lang="ts">
import { computed, ref } from "vue";
import { useBreakpoints } from "@vizejs/composable/breakpoints";

const { compact, expanded, current, active, greater, smaller, smallerOrEqual, between } =
  useBreakpoints({ compact: 480, expanded: 768 });
const aboveExpanded = greater("expanded");
const belowExpanded = smaller("expanded");
const atOrBelowExpanded = smallerOrEqual("expanded");
const middle = between("compact", "expanded");
const matchedNames = computed(() => current.value.join(", ") || "none");
const preference = ref<"automatic" | "summary" | "full">("automatic");
const showFull = computed(
  () => preference.value === "full" || (preference.value === "automatic" && expanded.value),
);
const draft = ref("");
</script>

<template>
  <div class="composable-example">
    <label>
      Guide detail
      <select v-model="preference">
        <option value="automatic">Automatic</option>
        <option value="summary">Summary</option>
        <option value="full">Full detail</option>
      </select>
    </label>
    <section aria-label="Responsive compiler guide">
      <strong>Review compiler output</strong>
      <p>Check the generated template and its diagnostics.</p>
      <p v-if="showFull">
        Compare the source locations and generated render calls before publishing.
      </p>
    </section>
    <label>
      Review draft
      <input v-model="draft" type="text" placeholder="Keep a local note while resizing" />
    </label>
    <output aria-live="polite">
      Matched: {{ matchedNames }} · Active: {{ active ?? "none" }} · Detail:
      {{ showFull ? "full" : "summary" }}
    </output>
    <dl class="breakpoint-readings">
      <div>
        <dt>At least 480 px</dt>
        <dd>{{ compact ? "yes" : "no" }}</dd>
      </div>
      <div>
        <dt>At least 768 px</dt>
        <dd>{{ expanded ? "yes" : "no" }}</dd>
      </div>
      <div>
        <dt>Above 768 px</dt>
        <dd>{{ aboveExpanded ? "yes" : "no" }}</dd>
      </div>
      <div>
        <dt>Below 768 px</dt>
        <dd>{{ belowExpanded ? "yes" : "no" }}</dd>
      </div>
      <div>
        <dt>At most 768 px</dt>
        <dd>{{ atOrBelowExpanded ? "yes" : "no" }}</dd>
      </div>
      <div>
        <dt>480 to below 768 px</dt>
        <dd>{{ middle ? "yes" : "no" }}</dd>
      </div>
    </dl>
    <p>
      These readonly results describe this preview's viewport, not the surrounding page. Resize it
      to cross 480 or 768 CSS pixels. Automatic shows full detail from 768 px; your explicit choice
      and draft remain unchanged.
    </p>
    <p>
      SSR starts with every query false, no matched names, and summary detail. The browser applies
      real media queries after the first render; the server does not know its width.
    </p>
  </div>
</template>

<style scoped>
.breakpoint-readings {
  display: grid;
  gap: 6px;
  margin: 0;

  & div {
    display: flex;
    justify-content: space-between;
    gap: 12px;
  }

  & dd {
    margin: 0;
    font-weight: 600;
  }
}
</style>
