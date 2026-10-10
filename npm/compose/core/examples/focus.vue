<script setup lang="ts">
import { ref, useId, useTemplateRef } from "vue";
import { useFocus, useFocusWithin } from "@vizejs/composable/focus";

const profileId = useId();
const nameInput = useTemplateRef<HTMLInputElement>("nameInput");
const profile = useTemplateRef<HTMLElement>("profile");
const { focused: nameFocused, stop: stopName } = useFocus(nameInput, { preventScroll: true });
const { focused: profileFocused, stop: stopProfile } = useFocusWithin(profile);
const stopped = ref(false);

function stopTracking(): void {
  stopName();
  stopProfile();
  stopped.value = true;
}
</script>

<template>
  <div class="composable-example">
    <div ref="profile" role="group" aria-label="Workshop profile">
      <label :for="`${profileId}-name`">Badge name</label>
      <input
        :id="`${profileId}-name`"
        ref="nameInput"
        autocomplete="nickname"
        @keydown.esc="() => (nameFocused = false)"
      />
      <label :for="`${profileId}-email`">Workshop contact</label>
      <input :id="`${profileId}-email`" type="email" autocomplete="email" />
      <button type="button" :disabled="stopped" @click="stopTracking">Stop focus tracking</button>
    </div>
    <output aria-live="polite"
      >{{ stopped ? "Last observed" : "Current" }}: name focused {{ nameFocused ? "yes" : "no" }} ·
      profile focus {{ profileFocused ? "yes" : "no" }}</output
    >
    <div class="example-actions">
      <button type="button" :disabled="stopped" @click="() => (nameFocused = true)">
        Focus badge name
      </button>
      <button type="button" :disabled="stopped" @click="() => (nameFocused = false)">
        Blur badge name
      </button>
    </div>
    <p>
      {{
        stopped
          ? "Tracking stopped. The last observed flags stay fixed; the fields remain editable."
          : "Tab between the two fields: the individual flag changes while profile focus stays yes. Tab outside the group to clear it, or press Escape in Badge name to blur it."
      }}
    </p>
  </div>
</template>
