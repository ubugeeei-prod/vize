<!-- Required textarea that reports an invalid state with a linked error message. -->
<script setup lang="ts">
import { computed, ref, useId } from "vue";

import { Textarea } from "../textarea.ts";

const reason = ref("");
const fieldId = useId();
const errorId = useId();
const invalid = computed(() => reason.value.trim().length < 10);
</script>

<template>
  <div>
    <label :for="fieldId">Reason for cancellation</label>
    <Textarea
      :id="fieldId"
      v-model="reason"
      name="reason"
      required
      :aria-invalid="invalid"
      :aria-errormessage="errorId"
    />
    <p :id="errorId">{{ invalid ? "Please write at least 10 characters." : "" }}</p>
  </div>
</template>
