export const formCompositeRendererFixtures = [
  {
    filename: "CheckboxGroupConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  CheckboxGroup,
  CheckboxGroupItem,
  CheckboxGroupSelectAll,
} from "./families/selection/checkbox-group/checkbox-group.ts";

interface Topping {
  readonly id: number;
  readonly label: string;
}

const toppings: readonly Topping[] = [
  { id: 1, label: "Cheese" },
  { id: 2, label: "Olives" },
];
const picked = ref<readonly Topping[]>([]);
</script>

<template>
  <CheckboxGroup
    v-model="picked"
    :options="toppings"
    :by="(topping) => topping.id"
    name="toppings"
    aria-label="Toppings"
  >
    <template #default="{ state }">
      <label><CheckboxGroupSelectAll /> All ({{ state }})</label>
      <label v-for="topping in toppings" :key="topping.id">
        <CheckboxGroupItem :value="topping" />
        {{ topping.label }}
      </label>
    </template>
  </CheckboxGroup>
</template>
`,
  },
  {
    filename: "PasswordFieldConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  PasswordField,
  PasswordFieldInput,
  PasswordFieldToggle,
  estimatePasswordStrength,
} from "./families/form/password-field/password-field.ts";

const password = ref("");
const visible = ref(false);
</script>

<template>
  <PasswordField
    v-model="password"
    v-model:visible="visible"
    aria-label="New password"
    autocomplete="new-password"
    :evaluate-strength="estimatePasswordStrength"
  >
    <template #default="{ capsLock, strength }">
      <PasswordFieldInput :minlength="8" />
      <PasswordFieldToggle v-slot="{ visible: shown }">{{ shown ? "Hide" : "Show" }}</PasswordFieldToggle>
      <p v-if="capsLock">Caps Lock is on</p>
      <meter :max="4" :value="strength?.score ?? 0">{{ strength?.label }}</meter>
    </template>
  </PasswordField>
</template>
`,
  },
  {
    filename: "PinInputConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { PinInput, PinInputField } from "./families/form/pin-input/pin-input.ts";

const code = ref("");
const last = ref<readonly string[]>([]);
</script>

<template>
  <PinInput
    v-model="code"
    :length="6"
    name="otp"
    aria-label="Verification code"
    @complete="(_, digits) => (last = digits)"
  >
    <template #default="{ indexes }">
      <template v-for="index in indexes" :key="index">
        <span v-if="index === 3" aria-hidden="true">-</span>
        <PinInputField :index="index" />
      </template>
    </template>
  </PinInput>
  <output>{{ last.join("") }}</output>
</template>
`,
  },
  {
    filename: "EditableConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  Editable,
  EditableInput,
  EditablePreview,
  EditableTrigger,
} from "./families/form/editable/editable.ts";

const title = ref("Quarterly report");
const editing = ref(false);
</script>

<template>
  <Editable
    v-model="title"
    v-model:editing="editing"
    aria-label="Document title"
    activation-mode="dblclick"
    placeholder="Untitled"
  >
    <template #default="{ state }">
      <EditablePreview v-slot="{ value, empty }">{{ empty ? "Untitled" : value }}</EditablePreview>
      <EditableInput />
      <EditableTrigger action="edit">Rename</EditableTrigger>
      <EditableTrigger action="submit">Save</EditableTrigger>
      <EditableTrigger action="cancel">Cancel</EditableTrigger>
      <output>{{ state }}</output>
    </template>
  </Editable>
</template>
`,
  },
] as const;
