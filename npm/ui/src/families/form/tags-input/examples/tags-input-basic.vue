<!-- Controlled tags input bound with v-model, labelled by visible text, with removable tags. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import {
  TagsInputInput,
  TagsInputItem,
  TagsInputItemDelete,
  TagsInputItemText,
  TagsInputRoot,
} from "../tags-input.ts";

const topics = ref<readonly string[]>(["vue", "vite"]);
const labelId = useId();
</script>

<template>
  <div>
    <span :id="labelId">Topics</span>
    <TagsInputRoot v-model="topics" name="topics" :max="5" :aria-labelledby="labelId">
      <template #default="{ tags }">
        <TagsInputItem v-for="(tag, index) in tags" :key="tag" :value="tag" :index>
          <TagsInputItemText />
          <TagsInputItemDelete>&times;</TagsInputItemDelete>
        </TagsInputItem>
        <TagsInputInput placeholder="Add a topic" />
      </template>
    </TagsInputRoot>
    <output>{{ topics.join(", ") }}</output>
  </div>
</template>
