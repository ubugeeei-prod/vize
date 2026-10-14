<script setup lang="ts">
import { formatName } from "./format";
import { computed } from "vue";
import type { User } from "./types";
import { useRoute } from "vue-router";

const props = defineProps<{ user: User }>();
const route = useRoute();
const name = computed(() => formatName(props.user, route.path));
</script>

<template>
  <p>{{ name }}</p>
</template>
