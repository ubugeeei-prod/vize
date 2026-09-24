<!-- Radio group in a native fieldset whose legend, description, and group error are wired together. -->
<script setup lang="ts">
import { computed, ref } from "vue";

import {
  Fieldset,
  FieldsetDescription,
  FieldsetErrorMessage,
  FieldsetLegend,
} from "../fieldset.ts";

const contact = ref("");
const errors = computed(() =>
  contact.value === ""
    ? [{ name: "contact", message: "Choose how we should contact you.", path: ["contact"] }]
    : [],
);
</script>

<template>
  <Fieldset name="contact" :errors has-description>
    <FieldsetLegend>Preferred contact method</FieldsetLegend>
    <FieldsetDescription>We only use this for order updates.</FieldsetDescription>
    <label><input v-model="contact" type="radio" name="contact" value="email" /> Email</label>
    <label><input v-model="contact" type="radio" name="contact" value="phone" /> Phone</label>
    <label><input v-model="contact" type="radio" name="contact" value="post" /> Post</label>
    <FieldsetErrorMessage />
  </Fieldset>
</template>
