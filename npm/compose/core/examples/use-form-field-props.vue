<script setup lang="ts">
import { useForm } from "@vizejs/composable/use-form";
import { useFormFieldProps } from "@vizejs/composable/use-form-field-props";

const form = useForm({
  initialValues: { email: "" },
  validateOn: "submit",
  revalidateOn: "change",
  validators: {
    email: (value) =>
      /^[^\s@]+@[^\s@]+$/.test(value) ? undefined : "Enter a complete email address.",
  },
});
const email = useFormFieldProps(form, "email", { idPrefix: "newsletter" });
const { value } = email.field;
const subscribe = form.handleSubmit(() => undefined);
</script>

<template>
  <form class="composable-example" novalidate @submit="subscribe">
    <p>Wire a label, input and error message together with the right ARIA attributes.</p>
    <label v-bind="email.labelProps.value">
      Newsletter email
      <input v-model="value" type="email" v-bind="email.fieldProps.value" />
    </label>
    <p v-bind="email.errorMessageProps.value">{{ email.errorMessage.value }}</p>
    <button type="submit">Subscribe</button>
  </form>
</template>
