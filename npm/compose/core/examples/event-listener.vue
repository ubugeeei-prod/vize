<script setup lang="ts">
import { computed, ref, useId, useTemplateRef } from "vue";
import { useEventListener } from "@vizejs/composable/event-listener";

const draftsId = useId();
const firstInput = useTemplateRef<HTMLInputElement>("firstInput");
const secondInput = useTemplateRef<HTMLInputElement>("secondInput");
const firstDraft = ref("Compiler workshop");
const secondDraft = ref("Editor workshop");
const selectedTarget = ref("first");
const target = computed(() =>
  selectedTarget.value === "first" ? firstInput.value : secondInput.value,
);
const targetName = computed(() => (selectedTarget.value === "first" ? "first" : "second"));
const observedEvents = ref(0);
const observedText = ref("No input observed yet.");
const { isListening, start, stop } = useEventListener(target, "input", (event) => {
  if (!(event.currentTarget instanceof HTMLInputElement)) return;
  observedEvents.value += 1;
  observedText.value = event.currentTarget.value;
});
</script>

<template>
  <div class="composable-example">
    <label :for="`${draftsId}-first`">First workshop draft</label>
    <input :id="`${draftsId}-first`" ref="firstInput" v-model="firstDraft" />
    <label :for="`${draftsId}-second`">Second workshop draft</label>
    <input :id="`${draftsId}-second`" ref="secondInput" v-model="secondDraft" />
    <label :for="`${draftsId}-target`">Observed draft</label>
    <select :id="`${draftsId}-target`" v-model="selectedTarget">
      <option value="first">First workshop draft</option>
      <option value="second">Second workshop draft</option>
    </select>
    <output aria-live="polite"
      >Listening to {{ targetName }} draft: {{ isListening ? "yes" : "no" }} · Observed input
      events: {{ observedEvents }}</output
    >
    <p>Last observed text: {{ observedText }}</p>
    <div class="example-actions">
      <button type="button" :disabled="!isListening" @click="() => stop()">
        Pause input observation
      </button>
      <button type="button" :disabled="isListening" @click="() => start()">
        Resume input observation
      </button>
    </div>
    <p>
      Both drafts remain editable. Only input events from the selected draft update the observation;
      pausing removes that listener and resuming attaches it to the current draft. This local
      monitor does not save or send draft text.
    </p>
  </div>
</template>
