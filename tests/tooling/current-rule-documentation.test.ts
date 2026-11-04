import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  currentRuleDocumentationJson,
  currentRuleDocumentationPlain,
  currentRuleDocumentationPacket,
  currentRuleDocumentationMessages,
} from "./support/current-rule-documentation.ts";
import { currentDocumentation, exact } from "./support/nuxt-render-scope-packets.ts";
import type { ProcessPacket } from "./support/nuxt-render-scope-packets.ts";
import { wholeJson as currentLintOracle } from "./lint-default-correctness-reference.mjs";
import {
  loadCases as loadSlotCases,
  wholeJson as slotOracle,
} from "./slot-style-fix-reference.mjs";

await test("finite current rule links preserve complete authored packets, messages and counters", () => {
  let packets = 0;
  let messageArrays = 0;
  const verify = (original: unknown) => {
    const saved = JSON.stringify(original);
    const projected = currentRuleDocumentationPacket(original) as {
      messages: Record<string, unknown>[];
    }[];
    const before = original as { messages: Record<string, unknown>[] }[];
    for (const [fileIndex, file] of projected.entries()) {
      for (const [index, message] of file.messages.entries()) {
        assert.equal(
          message.ruleDocsPath,
          message.ruleId === "cross-file"
            ? "docs/content/rules/cross-file.md"
            : `docs/content/rules/reference/${String(message.ruleId).replace("/", "-")}.md`,
        );
        message.ruleDocsPath = before[fileIndex]!.messages[index]!.ruleDocsPath;
      }
    }
    assert.deepEqual(projected, original, "all non-documentation fields remain exact");
    assert.equal(JSON.stringify(original), saved, "historical packet remains untouched");
    packets++;
  };
  const visit = (value: unknown) => {
    if (Array.isArray(value)) {
      for (const row of value) visit(row);
      return;
    }
    if (value === null || typeof value !== "object") return;
    const row = value as Record<string, unknown>;
    if (
      typeof row.filename === "string" &&
      typeof row.source === "string" &&
      Array.isArray(row.diagnostics)
    )
      verify(currentLintOracle(row));
    for (const field of ["afterCli", "afterCliPacket", "afterPacket"])
      if (Array.isArray(row[field])) verify(row[field]);
    if (Array.isArray(row.expectedCliErrors)) {
      const projected = currentRuleDocumentationMessages(row.expectedCliErrors);
      for (const [index, message] of projected.entries())
        message.ruleDocsPath = row.expectedCliErrors[index].ruleDocsPath;
      assert.deepEqual(projected, row.expectedCliErrors);
      messageArrays++;
    }
    for (const child of Object.values(row)) visit(child);
  };
  for (const relative of [
    "differential/lint-regressions/computed-local-reactivity-8756/controls.json",
    "differential/lint/bare-reserved-8090/cases.json",
    "differential/lint/props-deep-destructure-8090/cases.json",
    "differential/lint/public-vue-imports-8090/cases.json",
    "differential/lint/reserved-prefix-8090/cases.json",
    "differential/lint/reserved-suppression-8090/cases.json",
    "differential/lint/runtime-prop-defaults-8564/cases.json",
    "differential/lint/setup-emits-6097/cases.json",
    "differential/lint/setup-export-default-8090/cases.json",
    "differential/lint/setup-export-default-8090/default-route-controls.json",
    "differential/lint/typed-prop-default-prefilter-8564/current-range-cases.json",
    "differential/linter/a11y-listbox-focus-10925/cases.json",
    "lint-default-correctness/current-range-cases.json",
  ]) {
    const file = new URL(`../_fixtures/${relative}`, import.meta.url);
    const original = fs.readFileSync(file);
    visit(JSON.parse(original.toString("utf8")));
    assert.ok(fs.readFileSync(file).equals(original));
  }
  const slot = loadSlotCases(fileURLToPath(new URL("../../", import.meta.url)));
  for (const row of slot.cases)
    for (const fixed of [false, true]) verify(slotOracle(row, fixed, row.originalRules));
  assert.ok(packets > 500, String(packets));
  assert.ok(messageArrays > 0);
});

await test("cross-file directives retain complete historical packets with current rule pages", () => {
  const file = new URL(
    "../_fixtures/differential/linter/cross-file-directives-10912/source-cases.json",
    import.meta.url,
  );
  const original = fs.readFileSync(file);
  const corpus = JSON.parse(original.toString("utf8")) as {
    cases: {
      beforePacket: { messages: Record<string, unknown>[] }[];
      afterPacket: { messages: Record<string, unknown>[] }[];
    }[];
  };
  assert.equal(corpus.cases.length, 25);
  for (const entry of corpus.cases) {
    for (const before of [entry.beforePacket, entry.afterPacket]) {
      const current = currentRuleDocumentationPacket(before);
      for (const [fileIndex, row] of current.entries()) {
        for (const [index, message] of row.messages.entries()) {
          assert.equal(
            message.ruleDocsPath,
            message.ruleId === "vue/comment-directive"
              ? "docs/content/guide/lint-execution.md"
              : message.ruleId === "cross-file"
                ? "docs/content/rules/cross-file.md"
                : `docs/content/rules/reference/${String(message.ruleId).replace("/", "-")}.md`,
          );
          message.ruleDocsPath = before[fileIndex]!.messages[index]!.ruleDocsPath;
        }
      }
      assert.deepEqual(current, before);
    }
  }
  assert.ok(fs.readFileSync(file).equals(original));
});

await test("current rule projections refuse unknown rules, foreign paths and counter changes", () => {
  const message = {
    ruleId: "script/no-async-in-computed",
    ruleDocsPath: "docs/content/rules/type-and-script.md",
    severity: 2,
    message: "exact authored message",
    line: 1,
    column: 2,
    endLine: 1,
    endColumn: 8,
  };
  const packet = [{ file: "App.vue", messages: [message], errorCount: 1, warningCount: 0 }];
  for (const ruleDocsPath of [
    undefined,
    "docs/content/rules/vue.md",
    "docs/content/rules/reference/script-no-async-in-computed.md",
    "../foreign.md",
  ])
    assert.throws(
      () => currentRuleDocumentationMessages([{ ...message, ruleDocsPath }]),
      /frozen rule documentation URL/u,
    );
  assert.throws(
    () => currentRuleDocumentationMessages([{ ...message, ruleId: "script/foreign" }]),
    /finite authored documentation rule/u,
  );
  assert.throws(
    () => currentRuleDocumentationPacket([{ ...packet[0], errorCount: 0 }]),
    /authored error counter/u,
  );
  assert.throws(
    () => currentRuleDocumentationPacket([{ ...packet[0], warningCount: 1 }]),
    /authored warning counter/u,
  );
  assert.throws(
    () => currentRuleDocumentationPacket([{ ...packet[0], futureCounter: 0 }]),
    /complete authored packet fields/u,
  );
  const expected = currentRuleDocumentationPacket(packet);
  for (const field of ["message", "line", "column", "severity", "ruleId", "ruleDocsPath"]) {
    const actual = structuredClone(expected) as typeof packet;
    Object.assign(actual[0]!.messages[0]!, { [field]: "changed" });
    assert.notDeepEqual(actual, expected, field);
  }
});

await test("Nuxt process docs expectations preserve every non-URL byte and process field", () => {
  const packet = [
    {
      file: "src/layouts/bare.vue",
      messages: [
        {
          ruleId: "vue/no-unused-vars",
          ruleDocsPath: "docs/content/rules/vue.md",
          severity: 1,
          message: "Variable 'item' is defined by v-for but never used",
          line: 1,
          column: 44,
          endLine: 1,
          endColumn: 48,
        },
      ],
      errorCount: 0,
      warningCount: 1,
    },
  ];
  const oldPath = "docs/content/rules/vue.md";
  const currentPath = "docs/content/rules/reference/vue-no-unused-vars.md";
  for (const ending of ["", "\n"]) {
    const json = JSON.stringify(packet, null, 2) + ending;
    assert.equal(currentRuleDocumentationJson(json), json.replace(oldPath, currentPath));
  }
  for (const newline of ["\n", "\r\n"]) {
    const plain = [
      "Patina lint report: 1 warning in 1 file",
      "src/layouts/bare.vue",
      "  src/layouts/bare.vue:1:44 warning vue/no-unused-vars Variable 'item' is defined by v-for but never used",
      `    Reference: ${oldPath}`,
      "    Help:",
      "      Preserve this exact authored explanation",
      "",
    ].join(newline);
    assert.equal(currentRuleDocumentationPlain(plain), plain.replace(oldPath, currentPath));
  }
  for (const format of ["json", "plain"]) {
    const stdout =
      format === "json"
        ? JSON.stringify(packet, null, 2)
        : `  bare.vue:1:44 warning vue/no-unused-vars unused item\n    Reference: ${oldPath}\n`;
    const before: ProcessPacket = {
      command: "/public/vize",
      cwd: "/authored",
      args: ["lint", "--format", format],
      status: 0,
      signal: null,
      error: null,
      stdout,
      stderr: "retained exact stderr",
    };
    const saved = structuredClone(before);
    const actual = {
      ...before,
      command: "/source/vize",
      stdout: stdout.replace(oldPath, currentPath),
    };
    assert.throws(() => exact(actual, before));
    const expected = currentDocumentation(before);
    exact(actual, expected);
    assert.deepEqual(before, saved);
    assert.deepEqual({ ...expected, stdout: before.stdout }, before);
    assert.throws(() => exact({ ...actual, status: 1 }, expected));
    assert.throws(() => exact({ ...actual, stderr: "changed" }, expected));
  }
});

await test("Nuxt process docs expectations reject unknown formats, rules and references", () => {
  const header = "  bare.vue:1:44 warning vue/no-unused-vars unused item\n";
  const oldPath = "docs/content/rules/vue.md";
  for (const reference of [
    "docs/content/rules/index.md",
    "docs/content/rules/reference/vue-no-unused-vars.md",
    "../foreign.md",
  ])
    assert.throws(
      () => currentRuleDocumentationPlain(header + `    Reference: ${reference}\n`),
      /frozen rule documentation URL/u,
    );
  assert.throws(
    () =>
      currentRuleDocumentationPlain(
        header.replace("vue/no-unused-vars", "vue/foreign") + `    Reference: ${oldPath}\n`,
      ),
    /finite authored documentation rule/u,
  );
  assert.throws(
    () => currentRuleDocumentationPlain(`    Reference: ${oldPath}\n`),
    /preceding authored diagnostic rule/u,
  );
  assert.throws(() => currentRuleDocumentationJson(" [ ] "), /canonical authored JSON bytes/u);
  const process: ProcessPacket = {
    command: "/public/vize",
    cwd: "/authored",
    args: ["lint"],
    status: 0,
    signal: null,
    error: null,
    stdout: "[]",
    stderr: "",
  };
  assert.throws(() => currentDocumentation(process), /explicit authored report format/u);
  assert.throws(
    () => currentDocumentation({ ...process, args: ["lint", "--format", "future"] }),
    /finite authored report format/u,
  );
});
