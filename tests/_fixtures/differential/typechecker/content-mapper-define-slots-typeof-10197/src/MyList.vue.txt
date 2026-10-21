<script setup lang="ts">
defineSlots<{ item?(scope: ReturnType<typeof itemScope>): unknown }>();

function itemScope(index: number) {
  return { index, label: `#${index}` };
}
</script>

<template>
  <ul class="my-list">
    <li><slot name="item" v-bind="itemScope(1)"></slot></li>
  </ul>
</template>
