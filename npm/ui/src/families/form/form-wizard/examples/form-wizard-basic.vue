<!-- Three-step sign-up wizard with a controlled step, a validation gate, and native progress. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import {
  FormWizard,
  FormWizardBack,
  FormWizardNext,
  FormWizardProgress,
  FormWizardStep,
} from "../form-wizard.ts";
import type { FormWizardValidationContext } from "../form-wizard.ts";

type SignupStep = "account" | "profile" | "review";

const steps: readonly SignupStep[] = ["account", "profile", "review"];
const current = ref<SignupStep>("account");
const email = ref("");
const displayName = ref("");
const completed = ref(false);
const emailId = useId();
const nameId = useId();

function validate({ step }: FormWizardValidationContext<SignupStep>): boolean {
  return step !== "account" || email.value.includes("@");
}

function finish(): void {
  completed.value = true;
}
</script>

<template>
  <FormWizard v-model="current" :steps :validate aria-label="Create account" @complete="finish">
    <FormWizardProgress />
    <FormWizardStep v-slot="{ index }" step="account" label="Account">
      <h3>Step {{ index + 1 }}: Account</h3>
      <label :for="emailId">Email</label>
      <input :id="emailId" v-model="email" type="email" autocomplete="email" required />
    </FormWizardStep>
    <FormWizardStep v-slot="{ index }" step="profile" label="Profile">
      <h3>Step {{ index + 1 }}: Profile</h3>
      <label :for="nameId">Display name</label>
      <input :id="nameId" v-model="displayName" autocomplete="nickname" />
    </FormWizardStep>
    <FormWizardStep v-slot="{ index }" step="review" label="Review">
      <h3>Step {{ index + 1 }}: Review</h3>
      <p>{{ displayName || "Anonymous" }} &lt;{{ email }}&gt;</p>
    </FormWizardStep>
    <FormWizardBack>Back</FormWizardBack>
    <FormWizardNext v-slot="{ isLast }">{{ isLast ? "Create account" : "Next" }}</FormWizardNext>
    <output>{{ completed ? "Account created" : `On step: ${current}` }}</output>
  </FormWizard>
</template>
