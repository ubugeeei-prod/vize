<script setup lang="ts">
import { DiffView } from "../index.ts";

const patch = `--- a/readme.md
+++ b/readme.md
@@ -1,2 +1,2 @@
 keep
-old
+new
`;
</script>

<art title="Diff View" status="ready">
  <variant name="Default" default>
    <DiffView label="readme.md" :patch="patch" />
  </variant>
  <variant name="Structured">
    <DiffView
      label="notes"
      :lines="[
        { kind: 'context', text: 'keep' },
        { kind: 'remove', text: 'old' },
        { kind: 'add', text: 'new' },
      ]"
    />
  </variant>
</art>
