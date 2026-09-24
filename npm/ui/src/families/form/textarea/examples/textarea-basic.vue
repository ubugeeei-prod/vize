<!-- Labelled feedback textarea bound with v-model and a live character count. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { Textarea } from "../textarea.ts";

const message = ref("");
const fieldId = useId();
const hintId = useId();
</script>

<template>
  <div>
    <label :for="fieldId">Feedback</label>
    <Textarea
      :id="fieldId"
      v-model="message"
      name="feedback"
      :rows="4"
      :maxlength="280"
      placeholder="Tell us what you think"
      :aria-describedby="hintId"
    />
    <p :id="hintId">{{ message.length }} / 280 characters</p>
  </div>
</template>
