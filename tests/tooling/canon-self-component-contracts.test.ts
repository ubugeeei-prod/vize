import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { offsetToPosition } from "./support/lsp/assertions.ts";
import { PatternSession } from "./support/upstream/pattern-lsp.ts";
import {
  check,
  compareIdentity,
  diagnosticIdentity,
} from "./support/upstream/vue-language-tools.ts";
import { vueTscDiagnostics } from "./support/vue-tsc-oracle.ts";

async function compare(editor: PatternSession, source: string, errors: number): Promise<void> {
  fs.writeFileSync(editor.file, source);
  const expected = vueTscDiagnostics(editor.directory).sort(compareIdentity);
  assert.equal(expected.length, errors, JSON.stringify(expected));
  assert.deepEqual(
    (await check(editor.directory)).map(diagnosticIdentity).sort(compareIdentity),
    expected,
  );
  const actual = await editor.update(source);
  assert.deepEqual(
    actual
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

test("defineOptions names preserve recursive props and slots in CLI and live LSP", async () => {
  const editor = new PatternSession();
  const source = `<script setup lang="tsx">
defineOptions({ name: 'ChosenName' });
defineProps<{ count?: number }>();
defineSlots<{ default(props: { value: number }): unknown }>();
</script><template>
<ChosenName :count="1" v-slot="{ value }">{{ value.toFixed() }}</ChosenName>
</template>`;
  try {
    await editor.initialize();
    for (const spelling of ["ChosenName", "chosen-name"]) {
      const valid = source
        .replaceAll("<ChosenName", `<${spelling}`)
        .replaceAll("</ChosenName", `</${spelling}`);
      await compare(editor, valid, 0);
      await compare(editor, valid.replace("value.toFixed()", "value.toUpperCase()"), 1);
      await compare(editor, valid.replace(':count="1"', 'count="bad"'), 1);
      await compare(editor, valid, 0);
    }
  } finally {
    await editor.close();
  }
});

test("recursive SFC props and inferred slots stay typed in CLI and live LSP", async () => {
  const valid = `<script setup lang="ts">
defineProps<{ count?: number }>();
</script><template>
<slot name="foo" :value="123" />
<App :count="1"><template #foo="{ value }">{{ value.toFixed() }}</template></App>
</template>`;
  const editor = new PatternSession();
  try {
    await editor.initialize();
    await compare(editor, valid, 0);
    await compare(editor, valid.replace("value.toFixed()", "value.toUpperCase()"), 1);
    await compare(editor, valid.replace(':count="1"', 'count="bad"'), 1);
    const lowercase = valid.replaceAll("<App", "<app").replaceAll("</App", "</app");
    await compare(editor, lowercase, 0);
    await compare(editor, lowercase.replace(':count="1"', 'count="bad"'), 1);
    await compare(editor, valid, 0);
    const reference = valid.indexOf(':count="1"') + 1;
    assert.deepEqual(
      await editor.rename(reference),
      [valid.indexOf("count?"), reference],
      JSON.stringify(editor.lastRenameEdits),
    );
    const kebab = valid.replaceAll("count", "countValue").replace(":countValue", ":count-value");
    await compare(editor, kebab, 0);
    const declaration = kebab.indexOf("countValue?");
    const attribute = kebab.indexOf(":count-value") + 1;
    assert.deepEqual(await editor.rename(attribute), [declaration, attribute]);
    assert.deepEqual(
      editor.lastRenameEdits.map((edit) => edit.range),
      [
        {
          start: offsetToPosition(kebab, declaration),
          end: offsetToPosition(kebab, declaration + "countValue".length),
        },
        {
          start: offsetToPosition(kebab, attribute),
          end: offsetToPosition(kebab, attribute + "count-value".length),
        },
      ],
    );
  } finally {
    await editor.close();
  }
});

test("an imported component takes precedence over the implicit self component", async () => {
  const valid = `<script setup lang="ts">
import App from './Child.vue';
defineProps<{ count?: number }>();
</script><template>
<slot name="foo" :value="123" />
<App count="ok"><template #foo="{ value }">{{ value.toUpperCase() }}</template></App>
</template>`;
  const editor = new PatternSession();
  try {
    await editor.initialize();
    fs.writeFileSync(
      path.join(editor.directory, "Child.vue"),
      `<script setup lang="ts">defineProps<{ count: string }>();</script>
<template><slot name="foo" value="ok" /></template>`,
    );
    await compare(editor, valid, 0);
    await compare(editor, valid.replace("value.toUpperCase()", "value.toFixed()"), 1);
    await compare(editor, valid, 0);
  } finally {
    await editor.close();
  }
});

test("generic recursive components preserve their declared slot payload", async () => {
  const valid = `<script setup lang="ts" generic="T extends string">
defineProps<{ value: T }>();
defineSlots<{ default(props: { value: T }): any }>();
</script><template>
<App :value="value"><template #default="{ value: child }">{{ child.toUpperCase() }}</template></App>
</template>`;
  const editor = new PatternSession();
  try {
    await editor.initialize();
    await compare(editor, valid, 0);
    await compare(editor, valid.replace("child.toUpperCase()", "child.toFixed()"), 1);
    await compare(editor, valid, 0);
  } finally {
    await editor.close();
  }
});
