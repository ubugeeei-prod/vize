import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  VIZE_UI_SFC_AUTHORING_CONTRACT,
  VIZE_UI_SFC_AUTHORING_CONTRACT_SCHEMA_VERSION,
  VIZE_UI_SFC_AUTHORING_RULES,
  VIZE_UI_SFC_QUALITY_GATES,
} from "./authoring-contract.ts";
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

void test("publishes a versioned SFC authoring and quality-gate contract", () => {
  assert.equal(VIZE_UI_SFC_AUTHORING_CONTRACT_SCHEMA_VERSION, 1);
  assert.equal(VIZE_UI_SFC_AUTHORING_CONTRACT.schemaVersion, 1);
  assert.equal(VIZE_UI_SFC_AUTHORING_CONTRACT.packageName, "@vizejs/ui");
  assert.equal(VIZE_UI_SFC_AUTHORING_CONTRACT.sourceKind, "vue-sfc");
  assert.equal(VIZE_UI_SFC_AUTHORING_CONTRACT.stability, "stable");

  const ruleIds = VIZE_UI_SFC_AUTHORING_RULES.map((rule) => rule.id);
  const sortedRuleIds = [...ruleIds].sort();
  assert.equal(new Set(ruleIds).size, ruleIds.length, "rule ids must be unique");

  for (const rule of VIZE_UI_SFC_AUTHORING_RULES) {
    assert.ok(rule.title.length > 0, `${rule.id} must publish a title`);
    assert.ok(rule.requirement.length > 0, `${rule.id} must publish a requirement`);
    assert.ok(rule.evidence.length > 0, `${rule.id} must publish evidence`);
    assert.ok(rule.remediation.length > 0, `${rule.id} must publish remediation`);
  }

  const enforcedRuleIds = new Set(
    VIZE_UI_SFC_QUALITY_GATES.flatMap((gate) => gate.enforcedByRules),
  );
  assert.deepEqual(
    [...enforcedRuleIds].sort(),
    sortedRuleIds,
    "every authoring rule must be attached to a quality gate",
  );
});

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

void test("emits only rule ids published by the machine-readable contract", async () => {
  await withFixture(
    {
      "TheWidget.vue":
        "<script setup>defineProps<{ readonly label?: string }>(); " +
        "defineEmits<{ change: [value: boolean] }>(); const value = 1;</script>\n",
      "widget.test.ts": [
        'import { readFile } from "node:fs/promises";',
        'const source = await readFile(new URL("./TheWidget.vue", import.meta.url), "utf8");',
        "assert.match(source, /value/);",
      ].join("\n"),
    },
    async (directory) => {
      const emitted = new Set(
        (await auditComponentAuthoring(directory)).map((violation) => violation.rule),
      );
      assert.deepEqual(
        [...emitted].sort(),
        VIZE_UI_SFC_AUTHORING_RULES.map((rule) => rule.id).sort(),
      );
    },
  );
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

void test("requires public prop defaults to be documented for editor hover", async () => {
  const sfc = COMPLIANT_SFC.replace("   * @default false\n", "");
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
        ["prop-default-doc"],
      );
      assert.match(formatAuthoringViolations(violations), /Prop open is missing @default/);
    },
  );
});

void test("requires public emits to be documented for editor hover", async () => {
  const sfc = COMPLIANT_SFC.replace(
    "</script>",
    [
      "",
      "const emit = defineEmits<{",
      "  change: [value: boolean];",
      "}>();",
      "void emit;",
      "</script>",
    ].join("\n"),
  );
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
        ["event-doc"],
      );
      assert.match(formatAuthoringViolations(violations), /Event change is missing documentation/);
    },
  );
});

void test("ignores nested tuple payload fields when checking event docs", async () => {
  const sfc = COMPLIANT_SFC.replace(
    "</script>",
    [
      "",
      "const emit = defineEmits<{",
      "  /** Fired after the selected range changes and carries the selected tuple payload. */",
      "  readonly change: [{ selection: [string, string] }];",
      "}>();",
      "void emit;",
      "</script>",
    ].join("\n"),
  );
  await withFixture(
    {
      "TheWidget.vue": sfc,
      "widget.behavior.md": "TheWidget.vue",
      "widget.test.ts": 'import TheWidget from "./TheWidget.vue";\nexport default TheWidget;\n',
    },
    async (directory) => {
      assert.deepEqual(await auditComponentAuthoring(directory), []);
    },
  );
});

void test("keeps scanning events after nested object payload members", async () => {
  const sfc = COMPLIANT_SFC.replace(
    "</script>",
    [
      "",
      "const emit = defineEmits<{",
      "  /** Carries the nested selected-range payload after selection changes. */",
      "  change: [{ selection: { start: string; end: string } }];",
      "  submit: readonly [payload: { valid: boolean }];",
      "}>();",
      "void emit;",
      "</script>",
    ].join("\n"),
  );
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
        ["event-doc"],
      );
      assert.match(formatAuthoringViolations(violations), /Event submit is missing documentation/);
      assert.doesNotMatch(
        formatAuthoringViolations(violations),
        /Event (selection|start|end|valid)/,
      );
    },
  );
});

void test("requires readonly tuple emits to be documented", async () => {
  const sfc = COMPLIANT_SFC.replace(
    "</script>",
    [
      "",
      "const emit = defineEmits<{",
      "  readonly change: readonly [value: boolean];",
      "}>();",
      "void emit;",
      "</script>",
    ].join("\n"),
  );
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
        ["event-doc"],
      );
      assert.match(formatAuthoringViolations(violations), /Event change is missing documentation/);
    },
  );
});

void test("rejects SFCs that bypass the explicit authoring contract", async () => {
  const sfc = [
    '<script setup lang="ts">',
    "const props = withDefaults(defineProps<{",
    "  /**",
    "   * Whether the widget starts open.",
    "   *",
    "   * @default false",
    "   */",
    "  readonly open?: boolean;",
    "}>(), { open: false });",
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

void test("accepts semantic block attributes and a companion script", async () => {
  const sfc = COMPLIANT_SFC.replace(
    '<script setup lang="ts">',
    '<script>export const version = 1;</script>\n<script lang="ts" setup>',
  ).replace("<style scoped>", '<style lang="css" scoped>');
  await withFixture(
    {
      "TheWidget.vue": sfc,
      "widget.behavior.md": "TheWidget.vue",
      "widget.test.ts": 'import TheWidget from "./TheWidget.vue";\nexport default TheWidget;\n',
    },
    async (directory) => {
      assert.deepEqual(await auditComponentAuthoring(directory), []);
    },
  );
});

void test("does not accept canonical section spellings in inert script text", async () => {
  const sfc = `<script setup lang="ts">
const fakeSections = "<template></template><style scoped></style>";
</script>
`;
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
      assert.match(report, /Missing <template> block/);
      assert.match(report, /Missing <style scoped> block/);
    },
  );
});

void test("rejects malformed SFC section structure", async () => {
  const sfc = COMPLIANT_SFC.replace(
    "</template>",
    "</template>\n<template><p>Duplicate</p></template>",
  );
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
        ["explicit-sfc"],
      );
      assert.match(formatAuthoringViolations(violations), /SFC source has 1 parse error\(s\)/);
    },
  );
});
