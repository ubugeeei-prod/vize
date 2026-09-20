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

test("inferred own slots share checked payloads between useSlots and template calls", async () => {
  const editor = new PatternSession();
  const valid = `<!-- @inferTemplateDollarSlots true -->
<script setup lang="ts">
import { useSlots } from 'vue';
const slots = useSlots();
slots.summary?.({ label: 'April', total: 1200 });
</script><template>
<slot name="summary" :label="'April'" :total="1200" />
{{ $slots.summary?.({ label: 'April', total: 1200 }) }}
</template>`;
  try {
    await editor.initialize();
    for (const [source, count] of [
      [valid, 0],
      [
        valid.replace(
          "slots.summary?.({ label: 'April', total: 1200 })",
          "slots.summary?.({ label: 'April', total: 'wrong' })",
        ),
        1,
      ],
      [
        valid.replace(
          "$slots.summary?.({ label: 'April', total: 1200 })",
          "$slots.summary?.({ label: 'April', total: 'wrong' })",
        ),
        1,
      ],
      [valid, 0],
    ] as const) {
      await compare(editor, source, count);
    }
  } finally {
    await editor.close();
  }
});

test("own slot inference follows inherited options and only top-level file overrides", async () => {
  const directory = workspace("own-slot-options-");
  try {
    for (const [enabled, comment, errors] of [
      [false, "", 0],
      [true, "", 2],
      [false, "<!-- @inferTemplateDollarSlots true -->", 2],
      [true, "<!-- @inferTemplateDollarSlots false -->", 0],
      [false, "<!-- @strictTemplates true -->", 0],
    ] as const) {
      fs.writeFileSync(
        path.join(directory, "base.json"),
        JSON.stringify({
          vueCompilerOptions: { inferTemplateDollarSlots: enabled },
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
import { useSlots } from 'vue';
const slots = useSlots();
slots.summary?.({ total: 'wrong' });
</script><template>
<!-- @inferTemplateDollarSlots true -->
<slot name="summary" :total="1200" />
{{ $slots.summary?.({ total: 'wrong' }) }}
</template>`,
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

test("declared generic own slots retain setup type parameters", async () => {
  const editor = new PatternSession();
  const valid = `<!-- @inferTemplateDollarSlots true -->
<script setup lang="ts" generic="T extends { total: number }">
import { useSlots } from 'vue';
const { invoice } = defineProps<{ invoice: T }>();
defineSlots<{ summary(props: { invoice: T }): any }>();
const slots = useSlots();
slots.summary({ invoice });
</script><template>
<slot name="summary" :invoice="invoice" />
{{ $slots.summary({ invoice }) }}
</template>`;
  try {
    await editor.initialize();
    await compare(editor, valid, 0);
    await compare(
      editor,
      valid.replace("slots.summary({ invoice })", "slots.summary({ invoice: 'wrong' })"),
      1,
    );
    await compare(
      editor,
      valid.replace("$slots.summary({ invoice })", "$slots.summary({ invoice: 'wrong' })"),
      1,
    );
    await compare(editor, valid, 0);
  } finally {
    await editor.close();
  }
});

test("own slot imports preserve aliases, namespace helpers and lexical shadows", async () => {
  const directory = workspace("own-slot-imports-");
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
        vueCompilerOptions: { inferTemplateDollarSlots: true, strictCssModules: true },
        include: ["*.vue"],
      }),
    );
    fs.writeFileSync(
      path.join(directory, "App.vue"),
      `<script setup lang="ts">
import { useSlots as ownSlots, useCssModule as css } from 'vue';
import * as Vue from 'vue';
type Slots = { summary(props: { total: number }): any };
defineSlots<Slots>();
const slots: Slots = ownSlots();
slots.summary({ total: 1 });
// @ts-expect-error
ownSlots().summary({ total: 'wrong' });
// @ts-expect-error
Vue.useSlots().summary({ total: 'wrong' });
Vue.useSlots().summary({ total: Vue.ref(1).value });
const root: string = css().root;
const active: string = Vue.useCssModule('tokens').active;
// @ts-expect-error
Vue.useCssModule().missing;
// @ts-expect-error
css().missing;
function shadow(ownSlots: () => { authored: 1 }) { return ownSlots().authored; }
const one: 1 = shadow(() => ({ authored: 1 }));
void root; void active; void one;
</script><template><slot name="summary" :total="1" /></template>
<style module>.root {}</style><style module="tokens">.active {}</style>`,
    );
    // The reference's call-only replacement does not specialize namespace or
    // aliased imports. This independently checks the full authored binding API.
    assert.deepEqual(await check(directory), []);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("unused own-slot imports retain the authored unused diagnostic", async () => {
  const directory = workspace("own-slot-unused-");
  try {
    fs.writeFileSync(
      path.join(directory, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          strict: true,
          noUnusedLocals: true,
          skipLibCheck: true,
          module: "ESNext",
          moduleResolution: "Bundler",
        },
        include: ["*.vue"],
      }),
    );
    fs.writeFileSync(
      path.join(directory, "App.vue"),
      `<!-- @inferTemplateDollarSlots true -->
<script setup lang="ts">
import { useSlots } from 'vue';
const label = 'Summary';
</script><template><slot name="summary" :label="label" /></template>`,
    );
    const expected = vueTscDiagnostics(directory).sort(compareIdentity);
    assert.equal(expected.length, 1, JSON.stringify(expected));
    assert.equal(expected[0]?.code, 6133);
    // TypeScript Go highlights the unused binding, while vue-tsc highlights
    // the entire import. Both must preserve the real authored diagnostic.
    assert.deepEqual((await check(directory)).map(diagnosticIdentity).sort(compareIdentity), [
      { file: "App.vue", line: 3, column: 10, code: 6133 },
    ]);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
