import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  baselineIntentionalReason,
  patinaIntentionalReason,
} from "../../tools/support/fixtures/lint_divergence/records.ts";
import { compareLintFindings } from "../../tools/support/fixtures/lint_divergence/compare.ts";
import { mutableTemplateRefReason } from "../../tools/support/fixtures/lint_divergence/template_ref_intentional.ts";

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

void test("an auto-imported factory qualifies only while no binding claims its name", () => {
  const { source, finding } = sample(
    "useElementVisibility(element)\nconst reset = () => { element.value = undefined }",
  );
  const autoImported = source.replace("import { ref } from 'vue'\n", "");
  const shifted = { ...finding, line: finding.line - 1, endLine: finding.endLine - 1 };
  assert.match(baselineIntentionalReason(shifted, autoImported) ?? "", /readonly ref/u);
  for (const claim of [
    "function ref() { return { value: null } }",
    "const ref = (value?: unknown) => ({ value })",
  ])
    assert.equal(
      baselineIntentionalReason(
        shifted,
        autoImported.replace("const element", `${claim}\nconst element`),
      ),
      null,
      claim,
    );
  const sibling = `<script lang="ts">\nimport { ref } from './custom'\n</script>\n${autoImported}`;
  // Checked directly: the record classifier has an unrelated two-script reason.
  const below = { ...shifted, line: shifted.line + 3, endLine: shifted.endLine + 3 };
  assert.equal(mutableTemplateRefReason(below, sibling), null);
  assert.match(
    mutableTemplateRefReason(
      below,
      sibling.replace("import { ref } from './custom'", "export const id = 1"),
    ) ?? "",
    /readonly ref/u,
  );
});

function patinaSample(declaration: string, template: string, message: string, ruleId: string) {
  const source = `<script setup lang="ts">\nimport { ref, useTemplateRef, type Ref } from 'vue'\n${declaration}\nconsole.log(element.value)\n</script>\n<template>${template}</template>\n`;
  const start = source.indexOf(declaration.includes("useTemplateRef") ? 'ref="element"' : "ref(");
  const end = start + (declaration.includes("useTemplateRef") ? 'ref="element"'.length : 9);
  const lines = (offset: number) => source.slice(0, offset).split("\n");
  return {
    source,
    finding: {
      file: "Input.vue",
      ruleId,
      severity: "warning",
      line: lines(start).length,
      column: lines(start).at(-1)!.length + 1,
      endLine: lines(end).length,
      endColumn: lines(end).at(-1)!.length + 1,
      message,
    },
  };
}

void test("an asserted ref() initializer is a deliberately paired template ref", () => {
  const message =
    "[vize:script/prefer-use-template-ref] Template ref 'element' is declared with ref(); use useTemplateRef() instead (Vue 3.5+).";
  for (const declaration of [
    "const element = ref(null) as unknown as Ref<HTMLElement>",
    "const element = ref(null) satisfies Ref<HTMLElement | null>",
    "const element = ref(null)!",
  ]) {
    const { source, finding } = patinaSample(
      declaration,
      '<div ref="element" />',
      message,
      "script/prefer-use-template-ref",
    );
    assert.match(patinaIntentionalReason(finding, source) ?? "", /TypeScript `as`/u, declaration);
  }
  for (const declaration of [
    "const element = ref(null)",
    "const element = cast(ref(null)) as Ref<HTMLElement>",
  ]) {
    const { source, finding } = patinaSample(
      declaration,
      '<div ref="element" />',
      message,
      "script/prefer-use-template-ref",
    );
    assert.equal(patinaIntentionalReason(finding, source), null, declaration);
  }
});

void test("a template ref named after another key's useTemplateRef handle stays unread", () => {
  const message =
    "[vize:vue/no-unused-refs] Template ref 'element' is never referenced in <script>";
  const report = (declaration: string) => {
    const { source, finding } = patinaSample(
      declaration,
      '<div ref="element" />',
      message,
      "vue/no-unused-refs",
    );
    return patinaIntentionalReason(finding, source);
  };
  assert.match(
    report("const element = useTemplateRef<HTMLElement>('root-el')") ?? "",
    /another key/u,
  );
  for (const declaration of [
    "const element = useTemplateRef<HTMLElement>('element')",
    "const element = useTemplateRef<HTMLElement>(key)",
    "const element = ref<HTMLElement>()",
  ])
    assert.equal(report(declaration), null, declaration);
});
