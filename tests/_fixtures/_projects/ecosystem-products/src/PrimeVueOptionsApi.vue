<script lang="ts">
import type {} from "./shims";
import { defineComponent } from "vue";
import PrimeButton from "primevue/button";
import InputText from "primevue/inputtext";

interface PrimeAction {
  label: string;
  value: string;
  severity: "secondary" | "contrast" | "help";
}

const actions: PrimeAction[] = [
  { label: "Draft", value: "draft", severity: "secondary" },
  { label: "Publish", value: "publish", severity: "contrast" },
  { label: "Review", value: "review", severity: "help" },
];

export default defineComponent({
  name: "PrimeVueOptionsApi",
  components: {
    InputText,
    PrimeButton,
  },
  data() {
    return {
      query: "PrimeVue",
      selectedAction: actions[0]!,
      actions,
    };
  },
  computed: {
    buttonLabel(): string {
      return `${this.selectedAction.label}: ${this.query}`;
    },
    normalizedQuery(): string {
      return this.query.trim().toLowerCase();
    },
  },
  methods: {
    chooseAction(value: string): void {
      this.selectedAction =
        this.actions.find((action) => action.value === value) ?? this.actions[0]!;
    },
  },
});
</script>

<template>
  <section class="prime-options-panel" aria-label="PrimeVue Options API coverage">
    <InputText v-model="query" :aria-label="`${selectedAction.label} search`" />
    <PrimeButton :label="buttonLabel" :severity="selectedAction.severity" />
    <PrimeButton
      v-for="action in actions"
      :key="action.value"
      :label="action.label"
      :severity="action.severity"
      @click="chooseAction(action.value)"
    />
    <p>{{ normalizedQuery }}</p>
  </section>
</template>

<style scoped>
.prime-options-panel {
  display: grid;
  gap: 8px;
}
</style>
