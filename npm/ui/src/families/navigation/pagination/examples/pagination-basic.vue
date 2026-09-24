<!-- Search results pagination bound with v-model that collapses distant pages into ellipses. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  PaginationEllipsis,
  PaginationItem,
  PaginationList,
  PaginationNext,
  PaginationPage,
  PaginationPrevious,
  PaginationRoot,
} from "../pagination.ts";

const page = ref(5);
</script>

<template>
  <div>
    <p>Showing results {{ (page - 1) * 20 + 1 }}–{{ page * 20 }} of 240</p>
    <PaginationRoot v-slot="{ range }" v-model="page" :page-count="12" label="Search results pages">
      <PaginationList>
        <PaginationItem><PaginationPrevious>Previous</PaginationPrevious></PaginationItem>
        <template v-for="item in range" :key="item.key">
          <PaginationItem v-if="item.type === 'page'" :page="item.page">
            <PaginationPage :page="item.page">{{ item.page }}</PaginationPage>
          </PaginationItem>
          <PaginationItem v-else>
            <PaginationEllipsis :position="item.position" />
          </PaginationItem>
        </template>
        <PaginationItem><PaginationNext>Next</PaginationNext></PaginationItem>
      </PaginationList>
    </PaginationRoot>
  </div>
</template>
