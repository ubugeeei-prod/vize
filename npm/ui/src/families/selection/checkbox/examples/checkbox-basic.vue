<!-- Native checkbox bound with v-model and labelled through a matching label element. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { Checkbox } from "../checkbox.ts";

const accepted = ref(false);
const checkboxId = useId();
</script>

<template>
  <form>
    <Checkbox :id="checkboxId" v-model="accepted" name="terms" />
    <label :for="checkboxId">I agree to the terms of service</label>
    <button type="submit" :disabled="!accepted">Create account</button>
  </form>
</template>
