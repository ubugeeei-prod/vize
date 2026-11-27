<!-- Unified diff of a readme, with headers kept as metadata. -->
<script setup lang="ts">
import { DiffView } from "../index.ts";

const patch = `--- a/readme.md
+++ b/readme.md
@@ -1 +1 @@
-old
+new
`;
</script>

<template>
  <DiffView label="readme.md" :patch="patch" />
</template>
