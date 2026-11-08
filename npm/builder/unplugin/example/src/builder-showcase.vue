<script setup lang="ts">
import { computed, ref } from "vue";

type Capability = {
  id: string;
  label: string;
  detail: string;
};

const capabilities: Capability[] = [
  {
    id: "script-setup",
    label: "Typed script setup",
    detail: "The component state is authored in TypeScript and compiled by Vize.",
  },
  {
    id: "reactivity",
    label: "Vue reactivity",
    detail: "The same runtime behavior is bundled by every host in the matrix.",
  },
  {
    id: "scoped-css",
    label: "Scoped CSS",
    detail: "Each builder receives the same generated style module from the adapter.",
  },
];

const enabledCapabilityIds = ref(capabilities.map(({ id }) => id));
const enabledCount = computed(() => enabledCapabilityIds.value.length);
const buildTarget = document.querySelector<HTMLMetaElement>('meta[name="vize-builder"]')?.content;

function toggleCapability(capabilityId: string): void {
  enabledCapabilityIds.value = enabledCapabilityIds.value.includes(capabilityId)
    ? enabledCapabilityIds.value.filter((id) => id !== capabilityId)
    : [...enabledCapabilityIds.value, capabilityId];
}
</script>

<template>
  <main class="workspace">
    <p class="eyebrow">Vize builder matrix</p>
    <h1>{{ buildTarget ?? "Unknown builder" }}</h1>
    <p class="summary">
      One readable Vue SFC, compiled through independent builder adapters. Toggle a capability to
      confirm that the generated application remains interactive.
    </p>

    <section class="status" aria-live="polite">
      <strong>{{ enabledCount }} / {{ capabilities.length }}</strong>
      capabilities enabled
    </section>

    <ul class="capabilities">
      <li v-for="capability in capabilities" :key="capability.id">
        <label>
          <input
            type="checkbox"
            :checked="enabledCapabilityIds.includes(capability.id)"
            @change="toggleCapability(capability.id)"
          />
          <span>
            <strong>{{ capability.label }}</strong>
            <small>{{ capability.detail }}</small>
          </span>
        </label>
      </li>
    </ul>
  </main>
</template>

<style scoped>
:global(*) {
  box-sizing: border-box;
}

:global(body) {
  margin: 0;
  min-width: 20rem;
  color: #17231d;
  background: #eef8f2;
  font-family: Inter, ui-sans-serif, system-ui, sans-serif;
}

.workspace {
  width: min(44rem, calc(100% - 2rem));
  margin: 4rem auto;
  padding: clamp(1.5rem, 4vw, 3rem);
  border: 1px solid #c8ddd0;
  border-radius: 1.5rem;
  background: #fff;
  box-shadow: 0 1.5rem 4rem rgb(28 79 51 / 12%);
}

.eyebrow {
  margin: 0;
  color: #18794e;
  font-size: 0.75rem;
  font-weight: 800;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

h1 {
  margin: 0.35rem 0 0;
  font-size: clamp(2.2rem, 7vw, 4rem);
  line-height: 1;
}

.summary {
  max-width: 38rem;
  margin: 1.25rem 0 2rem;
  color: #52635a;
  line-height: 1.7;
}

.status {
  display: flex;
  gap: 0.45rem;
  align-items: baseline;
  padding: 0.8rem 1rem;
  border-radius: 0.8rem;
  color: #145c3c;
  background: #e7f7ed;
}

.capabilities {
  display: grid;
  gap: 0.75rem;
  margin: 1rem 0 0;
  padding: 0;
  list-style: none;
}

.capabilities label {
  display: flex;
  gap: 0.9rem;
  align-items: flex-start;
  padding: 1rem;
  border: 1px solid #dce8e0;
  border-radius: 0.9rem;
  cursor: pointer;
}

.capabilities label:hover {
  border-color: #78b990;
}

.capabilities input {
  width: 1.1rem;
  height: 1.1rem;
  margin-top: 0.15rem;
  accent-color: #18794e;
}

.capabilities span {
  display: grid;
  gap: 0.25rem;
}

.capabilities small {
  color: #65736b;
  line-height: 1.5;
}
</style>
