<script setup lang="ts">
import { searchParam, useUrlSearchParams } from "@vizejs/composable/use-url-search-params";

const { params } = useUrlSearchParams({
  q: searchParam.string(""),
  page: searchParam.number(1),
});
</script>

<template>
  <div class="composable-example">
    <p>Keep a search query and page number in the address bar.</p>
    <label>
      Search
      <input v-model="params.q" type="search" />
    </label>
    <div class="example-actions">
      <button type="button" :disabled="params.page <= 1" @click="() => (params.page -= 1)">
        Previous page
      </button>
      <button type="button" @click="() => (params.page += 1)">Next page</button>
    </div>
    <output aria-live="polite">Searching “{{ params.q }}”, page {{ params.page }}</output>
  </div>
</template>
