## Show the patch as a table

`DiffView` parses unified diffs. `---` and `+++` headers stay metadata, so they
are not reported as deletions or additions. Context lines keep their text
without the leading space. Pass `lines` when the application already
classified the edit.

```vue
<script setup lang="ts">
import { DiffView } from "@vizejs/ui/diff-view";

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
```

Rows expose `data-kind="context|add|remove|meta"`. Color belongs to the consumer.
