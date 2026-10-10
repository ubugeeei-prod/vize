<script setup lang="ts">
import { ref, useId, useTemplateRef } from "vue";
import { onClickOutside } from "@vizejs/composable/on-click-outside";

const preferencesId = useId();
const panel = useTemplateRef<HTMLElement>("panel");
const trigger = useTemplateRef<HTMLButtonElement>("trigger");
const ignored = useTemplateRef<HTMLButtonElement>("ignored");
const open = ref(false);
const note = ref("Keep examples copyable.");
const dismissals = ref(0);
const message = ref("No dismissal yet.");

onClickOutside(
  panel,
  () => {
    if (!open.value) return;
    open.value = false;
    dismissals.value += 1;
    message.value = "Dismissed by an outside click.";
  },
  { ignore: [trigger, ignored] },
);

function closeWithEscape(): void {
  open.value = false;
  message.value = "Closed with Escape inside the preferences.";
  trigger.value?.focus();
}
</script>

<template>
  <div class="composable-example">
    <button
      ref="trigger"
      type="button"
      :aria-expanded="open"
      :aria-controls="`${preferencesId}-panel`"
      @click="() => (open = !open)"
    >
      {{ open ? "Close guide preferences" : "Open guide preferences" }}
    </button>
    <section
      v-if="open"
      :id="`${preferencesId}-panel`"
      ref="panel"
      :aria-labelledby="`${preferencesId}-title`"
    >
      <h2 :id="`${preferencesId}-title`">Guide preferences</h2>
      <label :for="`${preferencesId}-note`">Review note</label>
      <input :id="`${preferencesId}-note`" v-model="note" @keydown.esc="closeWithEscape" />
      <button
        type="button"
        @click="() => (message = `Local note: ${note}`)"
        @keydown.esc="closeWithEscape"
      >
        Apply local preference
      </button>
      <p>
        This inline region keeps ordinary Tab navigation. Escape closes it and focuses its trigger.
      </p>
    </section>
    <div class="example-actions">
      <button ref="ignored" type="button">Keep preferences open</button>
      <button type="button">Continue reading outside</button>
    </div>
    <output aria-live="polite">Outside dismissals: {{ dismissals }} · {{ message }}</output>
    <p>The trigger and Keep preferences open are ignored. Clicking inside preserves the region.</p>
  </div>
</template>
