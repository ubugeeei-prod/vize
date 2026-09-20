import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
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
  assert.deepEqual(
    (await editor.update(source))
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

test("inferred model defaults remain typed in parent props and update events", async () => {
  const editor = new PatternSession();
  const valid = `<script setup lang="ts">
import Child from './Child.vue';
</script><template>
<Child :count="2" label="hello" @update:count="value => value.toFixed()" @update:label="value => value.toUpperCase()" />
</template>`;
  try {
    await editor.initialize();
    fs.writeFileSync(
      path.join(editor.directory, "defaults.ts"),
      "export const initialLabel = 'hello';\n",
    );
    fs.writeFileSync(
      path.join(editor.directory, "Child.vue"),
      `<script setup lang="ts">
import { initialLabel } from './defaults';
defineModel('count', { default: 1 });
const label = defineModel('label', { default: initialLabel });
</script><template>{{ label }}</template>`,
    );
    await compare(editor, valid, 0);
    assert.deepEqual(
      await check(editor.directory, ["--declaration", "--declaration-dir", "types"]),
      [],
    );
    await compare(editor, valid.replace(':count="2"', 'count="bad"'), 1);
    await compare(editor, valid.replace('label="hello"', ':label="2"'), 1);
    await compare(editor, valid.replace("value.toFixed()", "value.toUpperCase()"), 1);
    await compare(editor, valid.replace("value.toUpperCase()", "value.toFixed()"), 1);
    await compare(editor, valid, 0);
  } finally {
    await editor.close();
  }
});

for (const version of ["workspace", "upstream"] as const) {
  test(`model macros use ${version} Vue inference, defaults, modifiers and transformed reads/writes`, async () => {
    const editor = new PatternSession();
    const valid = `<script setup lang="ts">
type Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false;
const optional = defineModel<string>('optional');
const defaulted = defineModel('defaulted', { default: 1 });
const numeric = defineModel('numeric', { type: Number, default: 1 });
const [transformed, modifiers] = defineModel<number, 'trim', string, string>('transformed', {
  required: true,
  get(value) { return value.toFixed(); },
  set(value) { return Number(value); },
});
const types: [true, true, true, true, true] = [
  true as Equal<typeof optional.value, string | undefined>,
  true as Equal<typeof defaulted.value, ${version === "upstream" ? "1" : "number"}>,
  true as Equal<typeof numeric.value, number>,
  true as Equal<typeof transformed.value, string>,
  true as Equal<typeof modifiers, Record<'trim', true | undefined>>,
];
void types;
transformed.value = '2';
numeric.value = 2;
</script><template>{{ transformed.toUpperCase() }} {{ defaulted.toFixed() }}</template>`;
    try {
      if (version === "upstream") {
        const require = createRequire(import.meta.url);
        const vue = path.dirname(require.resolve("vue-language-tools-fixture-vue/package.json"));
        const modules = path.join(editor.directory, "node_modules");
        fs.unlinkSync(modules);
        fs.mkdirSync(modules);
        for (const [name, target] of [
          ["vue", vue],
          ["@vue", path.join(path.dirname(vue), "@vue")],
        ]) {
          fs.symlinkSync(
            target,
            path.join(modules, name),
            process.platform === "win32" ? "junction" : "dir",
          );
        }
      }
      await editor.initialize();
      await compare(editor, valid, 0);
      await compare(editor, valid.replace("transformed.value = '2'", "transformed.value = 2"), 1);
      await compare(editor, valid.replace("numeric.value = 2", "numeric.value = 'bad'"), 1);
      await compare(editor, valid.replace("defaulted.toFixed()", "defaulted.toUpperCase()"), 1);
      await compare(editor, valid, 0);
    } finally {
      await editor.close();
    }
  });
}

test("model defaults retain SFC generic parameters in public props and declarations", async () => {
  const editor = new PatternSession();
  const valid = `<script setup lang="ts">
import Child from './Child.vue';
</script><template>
<Child value="ok" @update:value="value => value.toUpperCase()" />
</template>`;
  try {
    await editor.initialize();
    fs.writeFileSync(
      path.join(editor.directory, "Child.vue"),
      `<script setup lang="ts" generic="T extends string">
defineModel('value', { default: undefined as unknown as T });
</script><template />`,
    );
    await compare(editor, valid, 0);
    assert.deepEqual(
      await check(editor.directory, ["--declaration", "--declaration-dir", "types"]),
      [],
    );
    await compare(editor, valid.replace('value="ok"', ':value="123"'), 1);
    await compare(editor, valid.replace("value.toUpperCase()", "value.toFixed()"), 1);
    await compare(editor, valid, 0);
  } finally {
    await editor.close();
  }
});
