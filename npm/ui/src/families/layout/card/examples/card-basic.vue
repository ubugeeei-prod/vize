<!-- Article card labelled by its heading, with summary text and a call to action. -->
<script setup lang="ts">
import { useId } from "vue";

import { Card } from "../card.ts";

const headingId = useId();
</script>

<template>
  <Card as="article" :aria-labelledby="headingId">
    <h3 :id="headingId">Release 2.4</h3>
    <p>Faster SSR, a new calendar family, and smaller bundles across every entry point.</p>
    <a href="#release-notes">Read the release notes</a>
  </Card>
</template>
