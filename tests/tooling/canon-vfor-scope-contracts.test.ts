import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { PatternSession } from "./support/upstream/pattern-lsp.ts";
import { check } from "./support/upstream/vue-language-tools.ts";

const ordinary = `<script setup lang="ts">
import { defineComponent } from 'vue';
const rows = [1, 2];
const Child = defineComponent({ props: { value: Number } });
</script><template>
<Child v-for="rows in rows" :value="rows"><slot :row="rows" />{{ rows.toFixed() }}</Child>
</template>`;
const patterned = `<script setup lang="ts">
type Result = { kind: 'ok'; rows: number[] } | { kind: 'err'; message: string };
const result = {} as Result;
</script>
<template v-match="result">
  <template v-when="{ kind: &quot;ok&quot;, const rows } as whole if (rows.length &gt; 0)">
    <p v-for="rows in rows">{{ rows.toFixed() }} {{ whole.rows.length }} {{ result.rows.length }}</p>
  </template>
  <p v-when="_">Fallback</p>
</template>`;

for (const [name, valid] of Object.entries({ ordinary, patterned })) {
  test(`v-for source keeps the outer ${name} binding in CLI and live LSP`, async () => {
    const editor = new PatternSession();
    try {
      await editor.initialize();
      for (const source of [valid, valid.replace("rows.toFixed()", "rows.toUpperCase()"), valid]) {
        fs.writeFileSync(editor.file, source);
        const cli = await check(editor.directory);
        const lsp = await editor.update(source);
        const expected = source === valid ? [] : [2339];
        assert.deepEqual(
          cli.map((d) => d.code),
          expected,
          JSON.stringify(cli),
        );
        assert.deepEqual(
          lsp.map((d) => Number(d.code)),
          expected,
          JSON.stringify(lsp),
        );
      }
      if (name === "ordinary") {
        const sourceOffset = valid.indexOf("rows in rows") + "rows in ".length;
        assert.deepEqual(await editor.rename(sourceOffset), [
          valid.indexOf("rows ="),
          sourceOffset,
        ]);
      }
    } finally {
      await editor.close();
    }
  });
}
