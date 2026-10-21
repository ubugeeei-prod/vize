<script setup lang="ts">
import { ref } from "vue";
import MyCell from "./MyCell.vue";

const mode = ref<"a" | "b" | "c">("b");
const rows = [[1, 2]];
</script>

<template>
  <p v-if="mode === 'a'">a</p>
  <table v-else-if="mode === 'b'">
    <tr v-for="(row, i) in rows" :key="i">
      <td v-for="cell in row" :key="cell"><MyCell :label="String(cell)" /></td>
    </tr>
  </table>
</template>
