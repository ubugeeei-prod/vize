<!-- Polymorphic primitive rendered as a labelled section element. -->
<script setup lang="ts">
import { useId } from "vue";

import { Primitive } from "../primitive.ts";

const headingId = useId();
</script>

<template>
  <Primitive as="section" :aria-labelledby="headingId">
    <h2 :id="headingId">Release notes</h2>
    <p>Version 2.4 adds keyboard shortcuts to the editor.</p>
  </Primitive>
</template>
