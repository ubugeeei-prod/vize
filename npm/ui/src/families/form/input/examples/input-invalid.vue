<!-- Required email input that exposes an error message while its value is invalid. -->
<script setup lang="ts">
import { computed, ref, useId } from "vue";

import { Input } from "../input.ts";

const email = ref("ada@");
const inputId = useId();
const errorId = useId();
const invalid = computed(() => !/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(email.value));
</script>

<template>
  <div>
    <label :for="inputId">Work email</label>
    <Input
      :id="inputId"
      v-model="email"
      type="email"
      name="email"
      required
      :aria-invalid="invalid"
      :aria-errormessage="invalid ? errorId : undefined"
    />
    <p v-if="invalid" :id="errorId">Enter an address like ada@example.com.</p>
  </div>
</template>
