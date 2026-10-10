<script setup lang="ts">
import { onScopeDispose } from "vue";
import { useAsyncState } from "@vizejs/composable/use-async-state";

interface Profile {
  name: string;
  team: string;
}

const pendingTimers = new Set<ReturnType<typeof setTimeout>>();

async function loadSampleProfile(id: "ada" | "grace" | "missing"): Promise<Profile> {
  // Local asynchronous sample data; this example sends no network request.
  await new Promise<void>((resolve) => {
    const timer = setTimeout(
      () => {
        pendingTimers.delete(timer);
        resolve();
      },
      id === "grace" ? 100 : 500,
    );
    pendingTimers.add(timer);
  });
  if (id === "missing") throw new Error("This profile could not be loaded. Try another profile.");
  return id === "ada"
    ? { name: "Ada Lovelace", team: "Compiler engineering" }
    : { name: "Grace Hopper", team: "Developer experience" };
}

onScopeDispose(() => {
  for (const timer of pendingTimers) clearTimeout(timer);
  pendingTimers.clear();
});

const {
  state: profile,
  isLoading,
  isReady,
  error,
  execute,
} = useAsyncState(loadSampleProfile, null, {
  immediate: false,
  resetOnExecute: true,
});
</script>

<template>
  <div class="composable-example">
    <p>Load sample team profiles with visible loading, error, and recovery states.</p>
    <div class="example-actions">
      <button type="button" @click="() => execute('ada')">Load Ada</button>
      <button type="button" @click="() => execute('grace')">Load Grace</button>
      <button type="button" @click="() => execute('missing')">Try missing profile</button>
    </div>
    <output aria-live="polite">{{
      isLoading
        ? "Loading profile…"
        : error instanceof Error
          ? error.message
          : profile !== null
            ? `Loaded ${profile.name}`
            : "Choose a profile to load."
    }}</output>
    <section v-if="profile !== null" aria-label="Loaded profile">
      <strong>{{ profile.name }}</strong>
      <p>{{ profile.team }}</p>
    </section>
    <p>
      {{ isReady ? "A profile has been loaded successfully." : "No profile has been loaded yet." }}
    </p>
    <p>Ada takes 500 ms; Grace takes 100 ms. The newest request owns the result.</p>
  </div>
</template>
