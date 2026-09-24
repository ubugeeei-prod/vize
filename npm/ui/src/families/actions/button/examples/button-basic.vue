<!-- Native buttons showing press handling, a loading state, and a disabled action. -->
<script setup lang="ts">
import { ref } from "vue";

import { Button } from "../button.ts";

const saves = ref(0);
</script>

<template>
  <div>
    <Button type="button" @press="() => (saves += 1)">Save draft</Button>
    <Button v-slot="{ loading }" type="button" loading>
      {{ loading ? "Uploading…" : "Upload" }}
    </Button>
    <Button type="button" disabled>Publish</Button>
    <output>Saved {{ saves }} times</output>
  </div>
</template>
