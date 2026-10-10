<script setup lang="ts">
import { computed, useId, useTemplateRef } from "vue";
import { useActiveElement } from "@vizejs/composable/active-element";

const profileId = useId();
const nameInput = useTemplateRef<HTMLInputElement>("nameInput");
const active = useActiveElement();
const focusedField = computed(() =>
  active.value?.getAttribute("data-profile-owner") === profileId
    ? active.value.getAttribute("data-profile-field")
    : null,
);
const guidance = computed(() => {
  if (focusedField.value === "Display name") return "Use the name you want on your workshop badge.";
  if (focusedField.value === "Email address")
    return "Use an address where you can receive workshop details.";
  if (focusedField.value === "Workshop note") return "Write a note for the workshop organizer.";
  return "Focus a profile field to see its guidance.";
});

function focusName(): void {
  nameInput.value?.focus();
}

function clearFocus(): void {
  if (active.value instanceof HTMLElement) active.value.blur();
}
</script>

<template>
  <div class="composable-example">
    <label :for="`${profileId}-name`">Display name</label>
    <input
      :id="`${profileId}-name`"
      ref="nameInput"
      autocomplete="nickname"
      :data-profile-owner="profileId"
      data-profile-field="Display name"
      :aria-describedby="`${profileId}-guidance`"
    />
    <label :for="`${profileId}-email`">Email address</label>
    <input
      :id="`${profileId}-email`"
      type="email"
      autocomplete="email"
      :data-profile-owner="profileId"
      data-profile-field="Email address"
      :aria-describedby="`${profileId}-guidance`"
    />
    <label :for="`${profileId}-note`">Workshop note</label>
    <input
      :id="`${profileId}-note`"
      :data-profile-owner="profileId"
      data-profile-field="Workshop note"
      :aria-describedby="`${profileId}-guidance`"
    />
    <output aria-live="polite"
      >Focused field: {{ focusedField || "Outside the profile fields" }}</output
    >
    <p :id="`${profileId}-guidance`">{{ guidance }}</p>
    <div class="example-actions">
      <button type="button" @click="focusName">Focus display name</button>
      <button type="button" @click="clearFocus">Clear focus</button>
    </div>
    <p>Press Tab through the fields. The guidance follows the document's actual focused element.</p>
  </div>
</template>
