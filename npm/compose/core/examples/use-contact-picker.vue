<script setup lang="ts">
import { ref } from "vue";
import { useContactPicker } from "@vizejs/composable/use-contact-picker";
import { useMounted } from "@vizejs/composable/use-mounted";

const mounted = useMounted();
const { supported, select } = useContactPicker();
const invited = ref<string[]>([]);

async function invite() {
  const result = await select(["name", "email"], { multiple: true });
  if (result.status !== "selected") return;
  invited.value = result.contacts.map(
    (contact) => contact.name?.[0] ?? contact.email?.[0] ?? "Unknown",
  );
}
</script>

<template>
  <div class="composable-example">
    <p>Invite people from the device's contacts instead of typing their addresses.</p>
    <button type="button" :disabled="!mounted || !supported" @click="invite">Pick contacts</button>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : !supported
          ? "The Contact Picker is not available; type the addresses instead."
          : invited.length
            ? `Inviting ${invited.join(", ")}`
            : "Ready to pick contacts."
    }}</output>
  </div>
</template>
