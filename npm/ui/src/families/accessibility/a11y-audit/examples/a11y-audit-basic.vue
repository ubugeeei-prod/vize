<!-- Development-only audit that warns about ui parts without accessible names. -->
<script setup lang="ts">
import { useTemplateRef } from "vue";

import { useA11yAudit } from "../a11y-audit.ts";

const root = useTemplateRef<HTMLDivElement>("root");
const audit = useA11yAudit(root);
</script>

<template>
  <div ref="root">
    <button type="button" data-vize-ui="button" aria-label="Close panel">×</button>
    <output>{{ audit.issues.value.length }} accessibility issues</output>
  </div>
</template>
