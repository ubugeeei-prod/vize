import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { PatternSession } from "./support/upstream/pattern-lsp.ts";
import {
  check,
  compareIdentity,
  diagnosticIdentity,
} from "./support/upstream/vue-language-tools.ts";
import { vueTscDiagnostics } from "./support/vue-tsc-oracle.ts";

test("reserved Options API names keep their instance types in CLI and live LSP", async () => {
  const valid = `<script lang="ts">
import { defineComponent } from "vue";
export default defineComponent({
  data() { return { class: 'vue', default: 1 }; },
  computed: { static() { return this.default + 1; } },
});
</script><template>
<div v-if="class" :title="class">{{ class }}
  {{ default }} {{ static }}
</div><p v-else />
</template>`;
  const editor = new PatternSession();
  try {
    await editor.initialize();
    for (const source of [
      valid,
      valid.replace("class: 'vue'", "class: 123"),
      valid.replace("{{ static }}", "{{ static.toUpperCase() }}"),
      valid,
    ]) {
      fs.writeFileSync(editor.file, source);
      const expected = vueTscDiagnostics(editor.directory).sort(compareIdentity);
      assert.equal(expected.length, source === valid ? 0 : 1, JSON.stringify(expected));
      assert.deepEqual(
        (await check(editor.directory)).map(diagnosticIdentity).sort(compareIdentity),
        expected,
      );
      const lsp = await editor.update(source);
      assert.deepEqual(
        lsp
          .map((d) => ({
            file: "App.vue",
            line: d.range.start.line + 1,
            column: d.range.start.character + 1,
            code: Number(d.code),
          }))
          .sort(compareIdentity),
        expected,
      );
    }
    const reference = valid.indexOf(':title="class"') + ':title="'.length;
    assert.deepEqual(await editor.rename(reference), [
      valid.indexOf("class: 'vue'"),
      valid.indexOf('v-if="class"') + 'v-if="'.length,
      reference,
      valid.indexOf("{{ class }}") + "{{ ".length,
    ]);
  } finally {
    await editor.close();
  }
});
