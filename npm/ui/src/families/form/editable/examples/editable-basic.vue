<!-- Inline-editable document title with explicit edit, save, and cancel buttons. -->
<script setup lang="ts">
import { ref } from "vue";

import { Editable, EditableInput, EditablePreview, EditableTrigger } from "../editable.ts";

const title = ref("Quarterly planning notes");
</script>

<template>
  <div>
    <Editable
      v-model="title"
      name="title"
      aria-label="Document title"
      placeholder="Untitled document"
      activation-mode="dblclick"
      :max-length="80"
    >
      <EditablePreview />
      <EditableInput />
      <EditableTrigger action="edit">Rename</EditableTrigger>
      <EditableTrigger action="submit">Save</EditableTrigger>
      <EditableTrigger action="cancel">Cancel</EditableTrigger>
    </Editable>
    <p>Saved title: {{ title }}</p>
  </div>
</template>
