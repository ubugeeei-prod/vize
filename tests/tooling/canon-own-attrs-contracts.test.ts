import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { PatternSession } from "./support/upstream/pattern-lsp.ts";
import {
  check,
  compareIdentity,
  diagnosticIdentity,
  workspace,
} from "./support/upstream/vue-language-tools.ts";
import { vueTscDiagnostics } from "./support/vue-tsc-oracle.ts";

async function compare(editor: PatternSession, source: string, count: number): Promise<void> {
  fs.writeFileSync(editor.file, source);
  const expected = vueTscDiagnostics(editor.directory).sort(compareIdentity);
  assert.equal(expected.length, count, JSON.stringify(expected));
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

const anchor = `<!-- @inferTemplateDollarAttrs true -->
<!-- @fallthroughAttributes true -->
<script setup lang="ts">
import { useAttrs } from 'vue';
const attrs = useAttrs();
attrs.href = 'https://example.com';
</script><template><a>{{ $attrs.href = 'https://example.com' }}</a></template>`;

test("useAttrs and $attrs retain inherited native attribute types through errors and repair", async () => {
  const editor = new PatternSession();
  try {
    await editor.initialize();
    for (const [source, count] of [
      [anchor, 0],
      [anchor.replace("attrs.href = 'https://example.com'", "attrs.href = 123"), 1],
      [anchor.replace("$attrs.href = 'https://example.com'", "$attrs.href = 123"), 1],
      [anchor, 0],
    ] as const)
      await compare(editor, source, count);
  } finally {
    await editor.close();
  }
});

test("own attrs inference follows inherited options and only top-level overrides", async () => {
  const directory = workspace("own-attrs-options-");
  try {
    for (const [enabled, comment, errors] of [
      [false, "", 0],
      [true, "", 2],
      [false, "<!-- @inferTemplateDollarAttrs true -->", 2],
      [true, "<!-- @inferTemplateDollarAttrs false -->", 0],
      [false, "<!-- @strictTemplates true -->", 0],
    ] as const) {
      fs.writeFileSync(
        path.join(directory, "base.json"),
        JSON.stringify({
          vueCompilerOptions: { inferTemplateDollarAttrs: enabled, fallthroughAttributes: true },
        }),
      );
      fs.writeFileSync(
        path.join(directory, "tsconfig.json"),
        JSON.stringify({
          extends: "./base.json",
          compilerOptions: {
            strict: true,
            skipLibCheck: true,
            module: "ESNext",
            moduleResolution: "Bundler",
          },
          include: ["*.vue"],
        }),
      );
      fs.writeFileSync(
        path.join(directory, "App.vue"),
        `${comment}
<script setup lang="ts">
import { useAttrs } from 'vue';
const attrs = useAttrs();
attrs.href = 123;
</script><template><!-- @inferTemplateDollarAttrs true -->
<a>{{ $attrs.href = 123 }}</a></template>`,
      );
      const expected = vueTscDiagnostics(directory).sort(compareIdentity);
      assert.equal(expected.length, errors, `${enabled}/${comment}: ${JSON.stringify(expected)}`);
      assert.deepEqual(
        (await check(directory)).map(diagnosticIdentity).sort(compareIdentity),
        expected,
      );
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("useAttrs includes authored ComponentCustomProperties with exact diagnostics", async () => {
  const editor = new PatternSession();
  const source = `<!-- @inferTemplateDollarAttrs true -->
<script setup lang="ts">
import { useAttrs } from 'vue';
declare module 'vue' {
  interface ComponentCustomProperties { $attrs: { invoiceTotal: number } }
}
const attrs = useAttrs();
attrs.invoiceTotal = 1200;
</script><template>{{ $attrs.invoiceTotal = 1200 }}</template>`;
  try {
    await editor.initialize();
    await compare(editor, source, 0);
    await compare(
      editor,
      source.replace("attrs.invoiceTotal = 1200", "attrs.invoiceTotal = 'wrong'"),
      1,
    );
    await compare(
      editor,
      source.replace("$attrs.invoiceTotal = 1200", "$attrs.invoiceTotal = 'wrong'"),
      1,
    );
    await compare(editor, source, 0);
  } finally {
    await editor.close();
  }
});

test("aliased and namespace attrs retain Vue APIs and lexical shadows", async () => {
  const directory = workspace("own-attrs-aliases-");
  try {
    fs.writeFileSync(
      path.join(directory, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          strict: true,
          skipLibCheck: true,
          module: "ESNext",
          moduleResolution: "Bundler",
        },
        include: ["*.vue"],
      }),
    );
    fs.writeFileSync(
      path.join(directory, "App.vue"),
      `<!-- @inferTemplateDollarAttrs true -->
<!-- @fallthroughAttributes true -->
<!-- @inferTemplateDollarSlots true -->
<script setup lang="ts">
import { useAttrs as capture } from 'vue';
import * as Vue from 'vue';
defineSlots<{ summary(props: { total: number }): any }>();
const attrs = capture();
const other = Vue.useAttrs();
const slots = Vue.useSlots();
const style = Vue.useCssModule();
const count = Vue.ref(1);
const href: string | undefined = attrs.href;
const otherHref: string | undefined = other.href;
// @ts-expect-error native attr type must not become any
const wrong: number = attrs.href;
// @ts-expect-error namespace attr type must not become any
const alsoWrong: number = other.href;
slots.summary({ total: count.value });
function shadow(capture: () => { local: boolean }) { return capture().local; }
void [href, otherHref, wrong, alsoWrong, style.card, shadow];
</script><template><a /></template><style module>.card { color: red; }</style>`,
    );
    // Alias/namespace specialization is an intentional authoring improvement over
    // the reference's literal composable-name matching. Assert the native contract.
    assert.deepEqual(await check(directory), []);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
