<script setup lang="ts">
import { ref, useId } from "vue";
import { useForm } from "@vizejs/composable/use-form";

const registrationId = useId();
const lastPreview = ref("");
const { field, dirty, submitCount, submitting, handleSubmit, reset } = useForm({
  initialValues: { name: "", email: "" },
  validateOn: "submit",
  revalidateOn: "change",
  validators: {
    name: (value) => (value.trim().length >= 3 ? undefined : "Use at least three characters."),
    email: (value) =>
      /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value)
        ? undefined
        : "Enter an email address with a domain.",
  },
});
const { value: name, error: nameError, onBlur: blurName } = field("name");
const { value: email, error: emailError, onBlur: blurEmail } = field("email");
const previewRegistration = handleSubmit((values) => {
  lastPreview.value = `Last preview: ${values.name.trim()} · ${values.email}`;
});

function resetRegistration(): void {
  reset();
  lastPreview.value = "";
}
</script>

<template>
  <form class="composable-example" novalidate @submit="previewRegistration">
    <p>Create a local workshop registration preview. No details are sent.</p>
    <label :for="`${registrationId}-name`">Attendee name</label>
    <input
      :id="`${registrationId}-name`"
      v-model="name"
      name="name"
      autocomplete="name"
      :aria-invalid="Boolean(nameError)"
      :aria-describedby="`${registrationId}-name-help ${registrationId}-name-error`"
      @blur="blurName"
    />
    <p :id="`${registrationId}-name-help`">Use at least three characters.</p>
    <p :id="`${registrationId}-name-error`" role="alert">{{ nameError || "" }}</p>
    <label :for="`${registrationId}-email`">Contact email</label>
    <input
      :id="`${registrationId}-email`"
      v-model="email"
      name="email"
      type="email"
      autocomplete="email"
      :aria-invalid="Boolean(emailError)"
      :aria-describedby="`${registrationId}-email-help ${registrationId}-email-error`"
      @blur="blurEmail"
    />
    <p :id="`${registrationId}-email-help`">Use an address such as ada@example.test.</p>
    <p :id="`${registrationId}-email-error`" role="alert">{{ emailError || "" }}</p>
    <output aria-live="polite"
      >Changed: {{ dirty ? "yes" : "no" }} · Submit attempts: {{ submitCount }}</output
    >
    <p role="status">{{ lastPreview || "No registration preview yet." }}</p>
    <div class="example-actions">
      <button type="submit" :disabled="submitting">Preview registration</button>
      <button type="button" @click="resetRegistration">Reset registration</button>
    </div>
    <p>
      Validation starts with Preview registration. After that attempt, edits revalidate each field.
    </p>
  </form>
</template>
