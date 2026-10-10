<script setup lang="ts">
import { ref, useId } from "vue";
import { useStepper } from "@vizejs/composable/use-stepper";

const wizardId = useId();
const attendee = ref("Ada");
const session = ref("Compiler clinic");
const {
  stepNames,
  index,
  current,
  isFirst,
  isLast,
  isCurrent,
  isBefore,
  goToNext,
  goToPrevious,
  goBackTo,
} = useStepper(["attendee", "session", "review"]);
const titles = { attendee: "Attendee", session: "Session", review: "Review" } as const;
</script>

<template>
  <div class="composable-example">
    <p>Prepare a workshop registration in three steps. Edits stay available when you go back.</p>
    <ol aria-label="Registration steps">
      <li v-for="step in stepNames" :key="step">
        <button
          type="button"
          :disabled="!isBefore(step) && !isCurrent(step)"
          :aria-current="isCurrent(step) ? 'step' : undefined"
          @click="() => goBackTo(step)"
        >
          {{ titles[step] }}
        </button>
      </li>
    </ol>
    <output aria-live="polite"
      >Step {{ index + 1 }} of {{ stepNames.length }}: {{ titles[current] }}</output
    >
    <div v-if="current === 'attendee'">
      <label :for="`${wizardId}-attendee`">Workshop attendee</label>
      <input :id="`${wizardId}-attendee`" v-model="attendee" autocomplete="name" />
    </div>
    <div v-else-if="current === 'session'">
      <label :for="`${wizardId}-session`">Workshop session</label>
      <select :id="`${wizardId}-session`" v-model="session">
        <option>Compiler clinic</option>
        <option>Type checking clinic</option>
        <option>Editor tools clinic</option>
      </select>
    </div>
    <div v-else>
      <h3>Review registration</h3>
      <p>Attendee: {{ attendee || "Not entered" }}</p>
      <p>Session: {{ session }}</p>
      <p>This is a local review. No registration has been sent.</p>
    </div>
    <div class="example-actions">
      <button type="button" :disabled="isFirst" @click="goToPrevious">Previous step</button>
      <button type="button" :disabled="isLast" @click="goToNext">Next step</button>
    </div>
    <p>Use an earlier step button to edit it. Later steps open with Next step.</p>
  </div>
</template>
