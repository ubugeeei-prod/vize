<script setup lang="ts">
import { ref } from "vue";

const count = ref(42);
const items = ref([1, 2, 3]);
</script>

<template>
  <div>
    <input :value="count.toUpperCase()" />
    <ul :class="items.split('-')">
      <li>{{ count }}</li>
    </ul>
    <!-- `href` is `string`, so binding a number is TS2322 at the attribute
         name — exactly what vue-tsc reports. `data-count` is not declared by
         the element type, so neither tool checks it. -->
    <a :href="count">link</a>
    <span :data-count="count" />
  </div>
</template>
