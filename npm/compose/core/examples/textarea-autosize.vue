<script setup lang="ts">
import { ref, useId, useTemplateRef } from "vue";
import { useTextareaAutosize } from "@vizejs/composable/textarea-autosize";

const draftId = useId();
const hintId = useId();
const compact = ref(false);
const draft = useTemplateRef<HTMLTextAreaElement>("draft");
const { input } = useTextareaAutosize(draft, { input: ref("") });

function insertTemplate(): void {
  input.value = [
    "Release highlights",
    "",
    "Fixed keyboard navigation in the guide catalogue.",
    "Improved compiler diagnostics for invalid templates.",
    "Verified server rendering and hydration.",
    "",
    "Upgrade notes",
    "Review the changes and run your application tests before publishing.",
  ].join("\n");
}
</script>

<template>
  <div class="composable-example">
    <p>Draft a local release note. Your text is not saved or sent to a server.</p>
    <div class="example-actions">
      <button type="button" @click="insertTemplate">Insert release template</button>
      <button type="button" @click="() => (input = '')">Clear draft</button>
      <button type="button" :aria-pressed="compact" @click="() => (compact = !compact)">
        Compact editor
      </button>
    </div>
    <div class="editor" :class="{ compact }">
      <label :for="draftId">Release notes</label>
      <textarea
        :id="draftId"
        ref="draft"
        v-model="input"
        :aria-describedby="hintId"
        rows="2"
        placeholder="What changed in this release?"
      />
      <p :id="hintId">The editor grows up to 12rem. Scroll inside it to read a longer draft.</p>
    </div>
    <output aria-live="polite">{{ input.length }} characters · local draft only</output>
  </div>
</template>

<style scoped>
.editor {
  width: 100%;
  max-width: 32rem;
  &.compact {
    max-width: 15rem;
  }
  & label {
    display: block;
    margin-bottom: 6px;
  }
  & textarea {
    box-sizing: border-box;
    display: block;
    width: 100%;
    max-height: 12rem;
    padding: 10px 12px;
    border: 2px solid #888;
    border-radius: 6px;
    font: inherit;
    line-height: 1.5;
    overflow-y: auto;
    resize: none;
  }
}
</style>
