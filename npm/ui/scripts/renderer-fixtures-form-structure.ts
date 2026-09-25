export const formStructureRendererFixtures = [
  {
    filename: "FieldsetConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  Fieldset,
  FieldsetDescription,
  FieldsetErrorMessage,
  FieldsetLegend,
} from "./families/form/fieldset/fieldset.ts";

const errors = [{ name: "address", message: "Enter a full address", path: ["address"] }];
</script>

<template>
  <Fieldset name="address" :errors="errors" has-description>
    <template #default="{ state }">
      <FieldsetLegend>Shipping address ({{ state }})</FieldsetLegend>
      <FieldsetDescription>Where we deliver</FieldsetDescription>
      <input aria-label="City" name="city" />
      <FieldsetErrorMessage v-slot="{ message }">{{ message }}</FieldsetErrorMessage>
    </template>
  </Fieldset>
</template>
`,
  },
  {
    filename: "FormWizardConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  FormWizard,
  FormWizardBack,
  FormWizardNext,
  FormWizardProgress,
  FormWizardStep,
} from "./families/form/form-wizard/form-wizard.ts";

const step = ref<"account" | "profile" | "review">("account");
const steps = ["account", "profile", "review"] as const;
const email = ref("");
</script>

<template>
  <FormWizard
    v-model="step"
    :steps="steps"
    aria-label="Sign up"
    :validate="({ step: current }) => current !== 'account' || email.length > 0"
  >
    <template #default="{ current, isLast }">
      <FormWizardStep step="account" label="Account"><input v-model="email" aria-label="Email" /></FormWizardStep>
      <FormWizardStep step="profile" label="Profile">Profile</FormWizardStep>
      <FormWizardStep step="review" label="Review">{{ current }}</FormWizardStep>
      <FormWizardProgress />
      <FormWizardBack>Back</FormWizardBack>
      <FormWizardNext>{{ isLast ? "Finish" : "Next" }}</FormWizardNext>
    </template>
  </FormWizard>
</template>
`,
  },
  {
    filename: "KnobConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { AnglePicker } from "./families/form/angle-picker/angle-picker.ts";
import { Knob } from "./families/form/knob/knob.ts";

const gain = ref(-6);
const hue = ref(210);
</script>

<template>
  <Knob v-model="gain" :min="-60" :max="12" :step="0.5" aria-label="Gain" :get-value-text="(value) => value + ' dB'">
    <template #default="{ angle }">
      <span :style="{ transform: 'rotate(' + angle + 'deg)' }"></span>
    </template>
  </Knob>
  <AnglePicker v-model="hue" aria-label="Hue" :step="15" v-slot="{ value }">
    <output>{{ value }}</output>
  </AnglePicker>
</template>
`,
  },
  {
    filename: "PhoneFieldConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  PhoneField,
  PhoneFieldCountrySelect,
  PhoneFieldInput,
  definePhoneCountries,
} from "./families/form/phone-field/phone-field.ts";

const countries = definePhoneCountries([
  { code: "JP", name: "Japan", dialCode: "81", pattern: "99-9999-9999", trunkPrefix: "0" },
  { code: "US", name: "United States", dialCode: "1", pattern: "(999) 999-9999" },
]);
const phone = ref("");
const country = ref<"JP" | "US">("JP");
</script>

<template>
  <PhoneField v-model="phone" v-model:country="country" :countries="countries" aria-label="Phone" name="phone">
    <template #default="{ international, state }">
      <PhoneFieldCountrySelect />
      <PhoneFieldInput placeholder="90-1234-5678" />
      <output :data-state="state">{{ international }}</output>
    </template>
  </PhoneField>
</template>
`,
  },
] as const;
