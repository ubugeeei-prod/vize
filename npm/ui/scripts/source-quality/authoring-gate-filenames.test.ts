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
  const directory = await mkdtemp(path.join(os.tmpdir(), "authoring-gate-filenames-"));
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

void test("rejects PascalCase public SFC filenames", async () => {
  await withFixture(
    {
      "ActionButton.vue": COMPLIANT_SFC,
      "widget.behavior.md": "ActionButton.vue",
      "widget.test.ts":
        'import ActionButton from "./ActionButton.vue";\nexport default ActionButton;\n',
    },
    async (directory) => {
      const violations = await auditComponentAuthoring(directory);
      assert.deepEqual(
        violations.map((violation) => violation.rule),
        ["kebab-case-filename"],
      );
      assert.match(
        formatAuthoringViolations(violations),
        /ActionButton\.vue \[kebab-case-filename\] Filename ActionButton\.vue is not kebab-case/,
      );
    },
  );
});

void test("rejects camelCase companion and test filenames", async () => {
  await withFixture(
    {
      "the-widget.vue": COMPLIANT_SFC,
      "widget.behavior.md": "the-widget.vue",
      "widget.test.ts": 'import TheWidget from "./the-widget.vue";\nexport default TheWidget;\n',
      "widgetState.ts": "export const widgetState = 1;\n",
      "myWidget.test.ts": "export {};\n",
    },
    async (directory) => {
      const violations = await auditComponentAuthoring(directory);
      assert.deepEqual(
        violations.map((violation) => [violation.file, violation.rule]),
        [
          ["myWidget.test.ts", "kebab-case-filename"],
          ["widgetState.ts", "kebab-case-filename"],
        ],
      );
    },
  );
});

void test("accepts kebab-case names including dotted test and fixture suffixes", async () => {
  await withFixture(
    {
      "the-widget.vue": COMPLIANT_SFC,
      "widget.behavior.md": "the-widget.vue",
      "widget.test.ts": 'import TheWidget from "./the-widget.vue";\nexport default TheWidget;\n',
      "widget-state.ts": "export const widgetState = 1;\n",
      "widget-ssr.test.ts": "export {};\n",
      "widget.types.test-d.ts": "export {};\n",
      "testing/mount-helpers.ts": "export {};\n",
    },
    async (directory) => {
      assert.deepEqual(await auditComponentAuthoring(directory), []);
    },
  );
});
