import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { baselineIntentionalReason } from "../../tools/support/fixtures/lint_divergence/records.ts";
import { compareLintFindings } from "../../tools/support/fixtures/lint_divergence/compare.ts";

function sample(body: string, call = "ref<HTMLInputElement>()") {
  const source = `<script setup lang="ts">\nimport { ref } from 'vue'\nconst element = ${call}\n${body}\n</script>\n<template><input ref="element" /></template>\n`;
  const position = (offset: number) => {
    const lines = source.slice(0, offset).split("\n");
    return { line: lines.length, column: lines[lines.length - 1].length + 1 };
  };
  const start = source.indexOf(call),
    end = position(start + call.length);
  return {
    source,
    finding: {
      file: "Input.vue",
      ruleId: "vue/prefer-use-template-ref",
      severity: "warning",
      ...position(start),
      endLine: end.line,
      endColumn: end.column,
      message: "Use useTemplateRef() instead of ref().",
    },
  };
}

void test("a proven mutable setup ref is an unsafe readonly migration", () => {
  for (const body of [
    "const reset = () => { element.value = undefined }",
    "const reset = () => { element['value'] = undefined }",
    "const reset = () => { element.value++ }",
  ]) {
    const { source, finding } = sample(body, body.includes("++") ? "ref<number>(0)" : undefined);
    assert.match(baselineIntentionalReason(finding, source) ?? "", /readonly ref/u);
  }
});

void test("Vue value aliases qualify but type-only imports cannot prove a ref factory", () => {
  for (const factory of ["createRef", "shallowRef"]) {
    const { source, finding } = sample(
      "element.value = undefined",
      `${factory}<HTMLInputElement>()`,
    );
    const specifier = factory === "createRef" ? "ref as createRef" : "shallowRef";
    assert.match(
      baselineIntentionalReason(finding, source.replace("{ ref }", `{ ${specifier} }`)) ?? "",
      /readonly ref/u,
    );
  }
  const { source, finding } = sample("element.value = undefined");
  assert.equal(
    baselineIntentionalReason(finding, source.replace("import { ref }", "import type { ref }")),
    null,
  );
  assert.equal(baselineIntentionalReason(finding, source.replace("{ ref }", "{ type ref }")), null);
});

void test("reads, nested mutations, shadows and text cannot waive an upstream warning", () => {
  for (const body of [
    "console.log(element.value)",
    "element.value?.focus()",
    "element.value!.disabled = true",
    "// element.value = undefined",
    "const text = 'element.value = undefined'",
    "const reset = (element: {value: unknown}) => { element.value = undefined }",
    "function reset({element}: {element: {value: unknown}}) { element.value = undefined }",
    "const reset = () => { const element = ref(); element.value = undefined }",
    "const other = ref(); other.value = undefined",
  ]) {
    const { source, finding } = sample(body);
    assert.equal(baselineIntentionalReason(finding, source), null, body);
  }
});

void test("an exception requires the exact reported call, valid source and mapped rule", () => {
  const { source, finding } = sample("element.value = undefined");
  assert.equal(baselineIntentionalReason({ ...finding, column: finding.column + 1 }, source), null);
  assert.equal(
    baselineIntentionalReason({ ...finding, ruleId: "vue/no-ref-as-operand" }, source),
    null,
  );
  assert.equal(
    baselineIntentionalReason(finding, source.replace("element.value = undefined", "const =")),
    null,
  );
  assert.equal(
    baselineIntentionalReason(finding, source.replace("from 'vue'", "from './custom'")),
    null,
  );
});

void test("the report discloses a proven divergence and still counts a missed safe migration", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-readonly-template-ref-"));
  const { source, finding } = sample("element.value = undefined");
  const map = {
    entries: {
      [finding.ruleId]: {
        status: "mapped",
        patinaRule: "script/prefer-use-template-ref",
      },
    },
  };
  try {
    const compare = () =>
      compareLintFindings(
        "fixture",
        root,
        map,
        [],
        [
          {
            filePath: join(root, finding.file),
            messages: [{ ...finding, severity: 1 }],
          },
        ],
        { files: [finding.file], rules: { [finding.ruleId]: "warn" } },
      );
    writeFileSync(join(root, finding.file), source);
    const mutable = compare();
    assert.equal((mutable.summary as { falseNegativeCount: number }).falseNegativeCount, 0);
    assert.equal(
      (mutable.summary as { intentionalDivergenceCount: number }).intentionalDivergenceCount,
      1,
    );
    writeFileSync(
      join(root, finding.file),
      source.replace("element.value = undefined", "console.log(element.value)"),
    );
    const readable = compare();
    assert.equal((readable.summary as { falseNegativeCount: number }).falseNegativeCount, 1);
    assert.equal(
      (readable.summary as { intentionalDivergenceCount: number }).intentionalDivergenceCount,
      0,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
