import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { offsetToPosition } from "./support/lsp/assertions.ts";
import { PatternSession } from "./support/upstream/pattern-lsp.ts";
import { check } from "./support/upstream/vue-language-tools.ts";

test(
  "CLI and live editor use the same authored-node diagnostic directives",
  { timeout: 120_000 },
  async () => {
    const editor = new PatternSession();
    const errors = async (source: string) => {
      fs.writeFileSync(editor.file, source);
      const cli = (await check(editor.directory))
        .filter((d) => d.severity === "error")
        .map((d) => ({
          code: d.code,
          line: d.line - 1,
          character: d.column - 1,
        }));
      const lsp = (await editor.update(source))
        .filter((d) => d.severity === 1)
        .map((d) => ({
          code: Number(d.code),
          ...d.range.start,
        }));
      const sort = (a: (typeof cli)[number], b: (typeof cli)[number]) =>
        a.line - b.line || a.character - b.character || a.code - b.code;
      assert.deepEqual(
        cli.sort(sort),
        lsp.sort(sort),
        "CLI and editor must agree after every edit",
      );
      return cli;
    };
    const expected = (source: string, token: string, code = 2304) => ({
      code,
      ...offsetToPosition(source, source.indexOf(token)),
    });
    try {
      await editor.initialize();
      const source = `<script setup lang="ts">const valid = 1;</script>\r\n<template>😀\r\n<!-- @vue-ignore --><div\r\n :id="ignored">{{ child }}</div>\r\n<!-- @vue-expect-error --><div :id="expectedA" :title="expectedB">\r\n<!-- @vue-expect-error -->{{ nested }}\r\n</div>\r\n<!-- @vue-skip --><div :id="skipped">{{ skippedChild }}<!-- @vue-expect-error -->{{ valid }}</div>\r\n<!-- prose @vue-ignore -->{{ visible }}\r\n<div title="<!-- @vue-ignore -->">{{ another }}</div>\r\n</template>`;
      assert.deepEqual(
        await errors(source),
        ["child", "visible", "another"].map((token) => expected(source, token)),
      );

      const expectation = `<script setup lang="ts">const valid = 1;</script>\r\n<template>😀<!-- @vue-expect-error -->\r\n{{ missing }}\r\n</template>`;
      assert.deepEqual(await errors(expectation), []);
      const repaired = expectation.replace("missing", "valid");
      assert.deepEqual(await errors(repaired), [
        expected(repaired, "<!-- @vue-expect-error", 2578),
      ]);
      assert.deepEqual(await errors(repaired.replace("<!-- @vue-expect-error -->", "")), []);
      const unguarded = expectation.replace("<!-- @vue-expect-error -->", "");
      assert.deepEqual(await errors(unguarded), [expected(unguarded, "missing")]);

      const strict = `<!-- @strictTemplates true -->
<script setup lang="ts">
import { defineComponent } from 'vue';
const Comp = defineComponent({ props: { title: String } });
</script>
<template>
<Comp unknownComponent="value" />
<div unknownNative="value" data-custom="value" />
</template>`;
      assert.equal((await errors(strict)).length, 2);
      const relaxed = strict.replace("@strictTemplates true", "@strictTemplates false");
      assert.deepEqual(await errors(relaxed), []);
      assert.deepEqual(
        await errors(relaxed.replace("<template>", "<template><!-- @strictTemplates true -->")),
        [],
      );
      assert.deepEqual(
        await errors(strict.replace("<template>", "<!-- @checkUnknownProps false --><template>")),
        [],
      );
    } finally {
      await editor.close();
    }
  },
);
