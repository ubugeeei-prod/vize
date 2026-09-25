<!-- Controlled search field with a native label and a built-in clear button. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { SearchField } from "../search-field.ts";

const query = ref("");
const inputId = useId();
</script>

<template>
  <div>
    <label :for="inputId">Search articles</label>
    <SearchField :id="inputId" v-model="query" name="q" placeholder="Try “hydration”">
      <template #clear>×</template>
    </SearchField>
    <p>{{ query === "" ? "Type to search." : `Results for “${query}”` }}</p>
  </div>
</template>
