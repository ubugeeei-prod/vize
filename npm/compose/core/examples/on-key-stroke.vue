<script setup lang="ts">
import { computed, ref, useId, useTemplateRef } from "vue";
import { onKeyStroke } from "@vizejs/composable/on-key-stroke";

const reviewId = useId();
const primary = useTemplateRef<HTMLButtonElement>("primary");
const secondary = useTemplateRef<HTMLButtonElement>("secondary");
const selectedTarget = ref("primary");
const target = computed(() =>
  selectedTarget.value === "primary" ? primary.value : secondary.value,
);
const tips = ["Name the props", "Label the controls", "Handle the empty state"] as const;
const index = ref(0);
const currentTip = computed(() => tips[index.value]);
const helpVisible = ref(false);
const stopped = ref(false);

function moveBy(offset: number): void {
  index.value = Math.max(0, Math.min(tips.length - 1, index.value + offset));
}

const stop = onKeyStroke(
  ["j", "k", "?"],
  (event) => {
    event.preventDefault();
    if (event.key === "?") helpVisible.value = !helpVisible.value;
    else moveBy(event.key.toLowerCase() === "j" ? 1 : -1);
  },
  { target, dedupe: true },
);

function stopShortcuts(): void {
  stop();
  stopped.value = true;
}
</script>

<template>
  <div class="composable-example">
    <label :for="reviewId">Shortcut target</label>
    <select :id="reviewId" v-model="selectedTarget" :disabled="stopped">
      <option value="primary">Primary review control</option>
      <option value="secondary">Secondary review control</option>
    </select>
    <div class="example-actions">
      <button ref="primary" type="button" :aria-describedby="`${reviewId}-help`">
        Primary review control
      </button>
      <button ref="secondary" type="button" :aria-describedby="`${reviewId}-help`">
        Secondary review control
      </button>
    </div>
    <p :id="`${reviewId}-help`">
      Focus the selected control. J moves forward, K moves back, and ? toggles help. Holding a key
      does not repeat the action. Other controls keep their normal keyboard behavior.
    </p>
    <output aria-live="polite">Tip {{ index + 1 }} of {{ tips.length }}: {{ currentTip }}</output>
    <p v-if="helpVisible">
      Review one component at a time. Name its props, label its controls, and test empty data.
    </p>
    <div class="example-actions">
      <button type="button" :disabled="index === 0" @click="() => moveBy(-1)">
        Previous review tip
      </button>
      <button type="button" :disabled="index === tips.length - 1" @click="() => moveBy(1)">
        Next review tip
      </button>
      <button type="button" :disabled="stopped" @click="stopShortcuts">
        Stop review shortcuts
      </button>
    </div>
    <p>
      {{
        stopped
          ? "Shortcuts stopped. The native Previous/Next buttons still work."
          : "Shortcuts follow only the selected element; native buttons also work on touch screens."
      }}
    </p>
  </div>
</template>
