<!-- Portal that moves a labelled notice to the document body after hydration. -->
<script setup lang="ts">
import { useId } from "vue";

import { Portal } from "../portal.ts";

const titleId = useId();
</script>

<template>
  <Portal>
    <aside :aria-labelledby="titleId">
      <h2 :id="titleId">Scheduled maintenance</h2>
      <p>The dashboard will be read-only on Saturday from 02:00 to 04:00 UTC.</p>
    </aside>
  </Portal>
</template>
