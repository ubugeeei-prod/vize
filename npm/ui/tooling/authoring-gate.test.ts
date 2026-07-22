import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { auditComponentAuthoring, formatAuthoringViolations } from "./authoring-gate.ts";

const COMPLIANT_SFC = `<script setup lang="ts">
const { open = false } = defineProps<{
  /**
   * Whether the widget starts open.
   *
   * @default false
   */
  readonly open?: boolean;
}>();
</script>

<template>
  <div data-vize-ui="widget">{{ open }}</div>
</template>

<style scoped>
/* Headless by design. */
</style>
`;

async function withFixture(
  files: Readonly<Record<string, string>>,
  run: (directory: string) => Promise<void>,
): Promise<void> {
  const directory = await mkdtemp(path.join(os.tmpdir(), "authoring-gate-"));
  try {
    for (const [name, content] of Object.entries(files)) {
      await mkdir(path.dirname(path.join(directory, name)), { recursive: true });
      await writeFile(path.join(directory, name), content);
    }
    await run(directory);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

void test("accepts a component with a behavior table and an interaction test", async () => {
  await withFixture(
    {
      "TheWidget.vue": COMPLIANT_SFC,
      "widget.behavior.md": "# Widget\n\nContract for `TheWidget.vue`.\n",
      "widget.test.ts": 'import TheWidget from "./TheWidget.vue";\nexport default TheWidget;\n',
    },
    async (directory) => {
      const violations = await auditComponentAuthoring(directory);
      assert.deepEqual(violations, []);
      assert.equal(formatAuthoringViolations(violations), "");
    },
  );
});

void test("requires a behavior table and an interaction test per SFC", async () => {
  await withFixture({ "TheWidget.vue": COMPLIANT_SFC }, async (directory) => {
    const violations = await auditComponentAuthoring(directory);
    assert.deepEqual(
      violations.map((violation) => violation.rule),
      ["behavior-table", "interaction-test"],
    );
    assert.match(formatAuthoringViolations(violations), /TheWidget\.vue \[behavior-table\]/);
  });
});

void test("rejects regex-on-source behavior assertions without a pragma", async () => {
  await withFixture(
    {
      "TheWidget.vue": COMPLIANT_SFC,
      "widget.behavior.md": "TheWidget.vue",
      "widget.test.ts": [
        'import { readFile } from "node:fs/promises";',
        'import TheWidget from "./TheWidget.vue";',
        'const source = await readFile(new URL("./TheWidget.vue", import.meta.url), "utf8");',
        "assert.match(source, /data-vize-ui/);",
        "export default TheWidget;",
      ].join("\n"),
    },
    async (directory) => {
      const violations = await auditComponentAuthoring(directory);
      assert.deepEqual(
        violations.map((violation) => violation.rule),
        ["source-regex-behavior", "source-regex-behavior"],
      );
      assert.match(violations[0]?.message ?? "", /Line 3/);
      assert.match(violations[1]?.message ?? "", /Line 4/);
    },
  );
});

void test("allows annotated source assertions that behavior cannot observe", async () => {
  await withFixture(
    {
      "TheWidget.vue": COMPLIANT_SFC,
      "widget.behavior.md": "TheWidget.vue",
      "widget.test.ts": [
        'import { readFile } from "node:fs/promises";',
        'import TheWidget from "./TheWidget.vue";',
        "// source-contract: computed styles need a real CSS pipeline.",
        'const source = await readFile(new URL("./TheWidget.vue", import.meta.url), "utf8");',
        "// source-contract: computed styles need a real CSS pipeline.",
        "assert.match(source, /clip-path/);",
        "export default TheWidget;",
      ].join("\n"),
    },
    async (directory) => {
      assert.deepEqual(await auditComponentAuthoring(directory), []);
    },
  );
});

void test("rejects SFCs that bypass the explicit authoring contract", async () => {
  const sfc = [
    '<script setup lang="ts">',
    "const props = withDefaults(defineProps<{ open?: boolean }>(), { open: false });",
    "</script>",
    "",
    "<template>",
    "  <div>{{ props.open }}</div>",
    "</template>",
  ].join("\n");
  await withFixture(
    {
      "TheWidget.vue": sfc,
      "widget.behavior.md": "TheWidget.vue",
      "widget.test.ts": 'import TheWidget from "./TheWidget.vue";\nexport default TheWidget;\n',
    },
    async (directory) => {
      const violations = await auditComponentAuthoring(directory);
      assert.deepEqual(
        violations.map((violation) => violation.rule),
        ["explicit-sfc", "explicit-sfc"],
      );
      const report = formatAuthoringViolations(violations);
      assert.match(report, /Missing <style scoped> block/);
      assert.match(report, /without helper indirection/);
    },
  );
});
