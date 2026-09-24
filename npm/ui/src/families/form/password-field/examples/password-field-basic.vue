<!-- New-password field with a visibility toggle, Caps Lock warning, and strength meter. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import {
  PasswordField,
  PasswordFieldInput,
  PasswordFieldToggle,
  estimatePasswordStrength,
} from "../password-field.ts";

const password = ref("");
const inputId = useId();
const strengthId = useId();
</script>

<template>
  <div>
    <label :for="inputId">New password</label>
    <PasswordField
      :id="inputId"
      v-slot="{ capsLock, strength }"
      v-model="password"
      name="password"
      autocomplete="new-password"
      :evaluate-strength="estimatePasswordStrength"
      :aria-describedby="strengthId"
    >
      <PasswordFieldInput />
      <PasswordFieldToggle v-slot="{ visible }">{{
        visible ? "Hide" : "Show"
      }}</PasswordFieldToggle>
      <meter aria-label="Password strength" :min="0" :max="4" :value="strength?.score ?? 0" />
      <p :id="strengthId">Strength: {{ strength?.label ?? "none" }}</p>
      <p v-if="capsLock" role="status">Caps Lock is on.</p>
    </PasswordField>
  </div>
</template>
