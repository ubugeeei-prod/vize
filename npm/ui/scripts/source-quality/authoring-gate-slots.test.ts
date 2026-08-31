import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { auditComponentAuthoring, formatAuthoringViolations } from "./authoring-gate.ts";

const COMPLIANT_SFC = `<script setup lang="ts">
defineProps<{
  /**
   * Whether the widget starts open.
   *
   * @default false
   */
  readonly open?: boolean;
}>();
</script>

<template>
  <div data-vize-ui="widget"><slot name="default" /></div>
</template>

<style scoped>
/* Headless by design. */
</style>
`;

async function withSlotFixture(sfc: string, run: (directory: string) => Promise<void>) {
  const directory = await mkdtemp(path.join(os.tmpdir(), "authoring-gate-slots-"));
  try {
    await mkdir(path.join(directory, "src"), { recursive: true });
    await writeFile(path.join(directory, "src", "the-widget.vue"), sfc);
    await writeFile(path.join(directory, "src", "widget.behavior.md"), "the-widget.vue");
    await writeFile(
      path.join(directory, "src", "widget.test.ts"),
      'import TheWidget from "./the-widget.vue";\nexport default TheWidget;\n',
    );
    await run(path.join(directory, "src"));
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

function withSlots(lines: readonly string[]): string {
  return COMPLIANT_SFC.replace(
    "</script>",
    ["", "const slots = defineSlots<{", ...lines, "}>();", "void slots;", "</script>"].join("\n"),
  );
}

void test("requires public slots to be documented for editor hover", async () => {
  await withSlotFixture(
    withSlots(["  default: (props: { readonly open: boolean }) => unknown;"]),
    async (directory) => {
      const violations = await auditComponentAuthoring(directory);
      assert.deepEqual(
        violations.map((violation) => violation.rule),
        ["slot-doc"],
      );
      assert.match(formatAuthoringViolations(violations), /Slot default is missing documentation/);
    },
  );
});

void test("accepts documented slot functions and nested slot props", async () => {
  await withSlotFixture(
    withSlots([
      "  /** Renders the widget body with its current controlled state. */",
      "  default: (props: { readonly state: { readonly open: boolean } }) => unknown;",
      "  /** Renders an optional named footer after widget content. */",
      "  footer?: (props: { readonly close: () => void }) => unknown;",
    ]),
    async (directory) => {
      assert.deepEqual(await auditComponentAuthoring(directory), []);
    },
  );
});

void test("rejects slot interface indirection before slot docs are inferred", async () => {
  const sfc = COMPLIANT_SFC.replace(
    "</script>",
    [
      "interface Slots {",
      "  /** Renders the widget body. */",
      "  default: () => unknown;",
      "}",
      "const slots = defineSlots<Slots>();",
      "void slots;",
      "</script>",
    ].join("\n"),
  );
  await withSlotFixture(sfc, async (directory) => {
    const violations = await auditComponentAuthoring(directory);
    assert.deepEqual(
      violations.map((violation) => violation.rule),
      ["explicit-sfc"],
    );
    assert.match(formatAuthoringViolations(violations), /defineSlots types/);
  });
});

void test("rejects slot type alias indirection before slot docs are inferred", async () => {
  const sfc = COMPLIANT_SFC.replace(
    "</script>",
    [
      "type SlotShape = {",
      "  /** Renders the widget body. */",
      "  default: () => unknown;",
      "};",
      "const slots = defineSlots<SlotShape>();",
      "void slots;",
      "</script>",
    ].join("\n"),
  );
  await withSlotFixture(sfc, async (directory) => {
    const violations = await auditComponentAuthoring(directory);
    assert.deepEqual(
      violations.map((violation) => violation.rule),
      ["explicit-sfc"],
    );
    assert.match(formatAuthoringViolations(violations), /defineSlots types/);
  });
});
