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

async function withEventFixture(sfc: string, run: (directory: string) => Promise<void>) {
  const directory = await mkdtemp(path.join(os.tmpdir(), "authoring-gate-events-"));
  try {
    await mkdir(path.join(directory, "src"), { recursive: true });
    await writeFile(path.join(directory, "src", "TheWidget.vue"), sfc);
    await writeFile(path.join(directory, "src", "widget.behavior.md"), "TheWidget.vue");
    await writeFile(
      path.join(directory, "src", "widget.test.ts"),
      'import TheWidget from "./TheWidget.vue";\nexport default TheWidget;\n',
    );
    await run(path.join(directory, "src"));
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

function withEmits(lines: readonly string[]): string {
  return COMPLIANT_SFC.replace(
    "</script>",
    ["", "const emit = defineEmits<{", ...lines, "}>();", "void emit;", "</script>"].join("\n"),
  );
}

void test("requires public emits to be documented for editor hover", async () => {
  await withEventFixture(withEmits(["  change: [value: boolean];"]), async (directory) => {
    const violations = await auditComponentAuthoring(directory);
    assert.deepEqual(
      violations.map((violation) => violation.rule),
      ["event-doc"],
    );
    assert.match(formatAuthoringViolations(violations), /Event change is missing documentation/);
  });
});

void test("ignores nested tuple payload fields when checking event docs", async () => {
  await withEventFixture(
    withEmits([
      "  /** Fired after selection changes and carries the selected tuple payload. */",
      "  readonly change: [{ selection: [string, string] }];",
    ]),
    async (directory) => {
      assert.deepEqual(await auditComponentAuthoring(directory), []);
    },
  );
});

void test("keeps scanning events after nested object payload members", async () => {
  await withEventFixture(
    withEmits([
      "  /** Carries the nested selected-range payload after selection changes. */",
      "  change: [{ selection: { start: string; end: string } }];",
      "  submit: readonly [payload: { valid: boolean }];",
    ]),
    async (directory) => {
      const report = formatAuthoringViolations(await auditComponentAuthoring(directory));
      assert.match(report, /Event submit is missing documentation/);
      assert.doesNotMatch(report, /Event (selection|start|end|valid)/);
    },
  );
});

void test("requires readonly tuple emits to be documented", async () => {
  await withEventFixture(
    withEmits(["  readonly change: readonly [value: boolean];"]),
    async (directory) => {
      const violations = await auditComponentAuthoring(directory);
      assert.deepEqual(
        violations.map((violation) => violation.rule),
        ["event-doc"],
      );
      assert.match(formatAuthoringViolations(violations), /Event change is missing documentation/);
    },
  );
});
