import { fileURLToPath } from "node:url";
import { currentRuleDocumentationJson } from "./support/current-rule-documentation.ts";
import { currentDocumentation, exact } from "./support/nuxt-render-scope-packets.ts";
import type { ProcessPacket } from "./support/nuxt-render-scope-packets.ts";
import { wholeJson as currentLintOracle } from "./lint-default-correctness-reference.mjs";
import { loadCases as loadSlotCases, wholeJson as slotOracle } from "./slot-style-fix-reference.mjs";
import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { asArray, asRecord, asString } from "../../tools/support/json-value.ts";
import { wholeJson } from "./lint-default-correctness-reference.mjs";
import { withoutHelp } from "./support/lint-help-policy.mjs";
import {
  currentRuleDocumentationMessage,
  currentRuleDocumentationMessages,
  currentRuleDocumentationPacket,
  currentRuleDocumentationPlain,
} from "./support/current-rule-documentation.ts";

const histories = [
  ["a11y/interactive-supports-focus", "accessibility"],
  ["a11y/no-aria-hidden-on-focusable", "accessibility"],
  ["css/no-display-none", "musea-and-css"],
  ["script/no-import-compiler-macros", "type-and-script"],
  ["script/no-required-prop-with-default", "type-and-script"],
  ["script/no-reserved-keys", "type-and-script"],
  ["script/no-reserved-props", "type-and-script"],
  ["script/return-in-emits-validator", "type-and-script"],
  ["script/valid-define-emits", "type-and-script"],
  ["vue/no-deprecated-scope-attribute", "vue"],
  ["vue/no-deprecated-slot-scope-attribute", "vue"],
  ["vue/no-deprecated-v-bind-sync", "vue"],
  ["vue/no-deprecated-v-on-native-modifier", "vue"],
  ["vue/v-bind-style", "vue"],
  ["vue/v-on-style", "vue"],
  ["css/no-id-selectors", "musea-and-css"],
  ["css/no-utility-classes", "musea-and-css"],
  ["html/no-empty-palpable-content", "html"],
  ["script/no-async-in-computed", "type-and-script"],
  ["script/define-props-destructuring", "type-and-script"],
  ["script/no-deep-destructure-in-props", "type-and-script"],
  ["script/no-export-in-script-setup", "type-and-script"],
  ["script/no-options-api", "type-and-script"],
  ["script/no-with-defaults", "type-and-script"],
  ["script/prefer-import-from-vue", "type-and-script"],
  ["script/return-in-computed-property", "type-and-script"],
  ["type/no-reactivity-loss", "type-and-script"],
  ["vue/component-name-in-template-casing", "vue"],
  ["vue/html-self-closing", "vue"],
  ["vue/no-boolean-attr-value", "vue"],
  ["vue/no-deprecated-v-on-number-modifiers", "vue"],
  ["vue/no-deprecated-functional-template", "vue"],
  ["vue/no-unused-properties", "vue"],
  ["vue/no-unused-vars", "vue"],
  ["vue/require-component-registration", "vue"],
  ["vue/require-scoped-style", "vue"],
  ["vue/sfc-element-order", "vue"],
  ["vue/require-v-for-key", "vue"],
  ["vue/v-slot-style", "vue"],
  ["nuxt/nuxt-config-keys-order", "index"],
] as const;
const message = (ruleId: string, category: string) => ({
  ruleId,
  ruleDocsPath: `docs/content/rules/${category}.md`,
  severity: 2,
  message: "complete authored message",
  help: "complete authored help",
  line: 2,
  column: 3,
  endLine: 2,
  endColumn: 9,
});

void test("only the finite known documentation fields change, preserving complete packets and frozen inputs", () => {
  for (const [rule, category] of histories) {
    const expected = [
      { file: "Probe.vue", messages: [message(rule, category)], errorCount: 1, warningCount: 0 },
    ];
    const before = structuredClone(expected);
    const projected = currentRuleDocumentationPacket(expected);
    const target = `docs/content/rules/reference/${rule.replace("/", "-")}.md`;
    assert.equal(asArray(projected[0]!.messages).length, 1);
    assert.equal(asRecord(asArray(projected[0]!.messages)[0]).ruleDocsPath, target);
    const inverse = structuredClone(projected);
    asRecord(asArray(inverse[0]!.messages)[0]).ruleDocsPath = before[0]!.messages[0]!.ruleDocsPath;
    assert.deepEqual(inverse, before);
    assert.deepEqual(expected, before, "never mutate the original");
    for (const locale of ["", "ja/"])
      assert.match(
        fs.readFileSync(
          new URL(
            `../../${target.replace("docs/content/", `docs/content/${locale}`)}`,
            import.meta.url,
          ),
          "utf8",
        ),
        new RegExp(`^title: "${rule}"$`, "mu"),
      );
  }
});

void test("synthetic directives retain their operational guide and whole message fields", () => {
  for (const [rule, category, target] of [
    ["vue/comment-directive", "vue", "docs/content/guide/lint-execution.md"],
    ["cross-file", "cross-file", "docs/content/rules/cross-file.md"],
  ]) {
    const original = [message(rule!, category!)];
    const current = currentRuleDocumentationMessages(original);
    assert.equal(current[0]!.ruleDocsPath, target);
    current[0]!.ruleDocsPath = original[0]!.ruleDocsPath;
    assert.deepEqual(current, original);
  }
});

void test("plain reports keep all non-reference bytes and reject unrelated documentation", () => {
  for (const [rule, category] of histories) {
    const before = `whole report\n    Reference: docs/content/rules/${category}.md\n    Help:\n      original help\n`;
    const current = currentRuleDocumentationPlain(before, rule);
    const afterLink = `docs/content/rules/reference/${rule.replace("/", "-")}.md`;
    assert.equal(current, before.replace(`docs/content/rules/${category}.md`, afterLink));
    assert.throws(() => currentRuleDocumentationPlain(current, rule));
    assert.throws(() =>
      currentRuleDocumentationPlain(before.replace(`${category}.md`, "foreign.md"), rule),
    );
  }
  assert.equal(
    currentRuleDocumentationPlain("clean report\n", "css/no-display-none"),
    "clean report\n",
  );
  assert.throws(() => currentRuleDocumentationPlain("clean report\n", "css/future-rule"));
});

void test("complete stored CSS, listbox and runtime-prop packets keep every non-URL field", () => {
  let packets = 0;
  const visit = (value: unknown): void => {
    if (Array.isArray(value)) {
      if (
        value.length &&
        value.every((row) => row && typeof row === "object" && "file" in row && "messages" in row)
      ) {
        const before = structuredClone(value);
        const projected = currentRuleDocumentationPacket(value);
        for (let index = 0; index < projected.length; index++) {
          const original = asRecord(before[index]);
          const messages = asArray(projected[index]!.messages);
          const originalMessages = asArray(original.messages);
          assert.equal(messages.length, originalMessages.length);
          for (let offset = 0; offset < messages.length; offset++)
            asRecord(messages[offset]).ruleDocsPath = asRecord(
              originalMessages[offset],
            ).ruleDocsPath;
        }
        assert.deepEqual(projected, before);
        assert.deepEqual(value, before);
        packets++;
      }
      value.forEach(visit);
    } else if (value && typeof value === "object") Object.values(value).forEach(visit);
  };
  for (const path of [
    "crates/vize_patina/tests/fixtures/issue-7976-compound/cases.json",
    "crates/vize_patina/tests/fixtures/issue-7976-universal/cases.json",
    "tests/_fixtures/differential/linter/a11y-listbox-focus-10925/cases.json",
    "tests/_fixtures/differential/lint/runtime-prop-defaults-8564/scoped-route-contract.json",
    "tests/_fixtures/differential/lint/runtime-prop-defaults-8564/default-route-controls.json",
  ])
    visit(
      JSON.parse(fs.readFileSync(new URL(`../../${path}`, import.meta.url), "utf8")) as unknown,
    );
  assert.equal(packets, 535, "retain every stored whole packet in the five frozen corpora");
});

void test("unknown rules, old/current URLs, severity, packet fields and counter changes are refused", () => {
  const original = message("script/no-reserved-keys", "type-and-script");
  for (const patch of [
    { ruleId: "script/future-rule" },
    { ruleDocsPath: "docs/content/rules/vue.md" },
    { ruleDocsPath: "docs/content/rules/reference/script-no-reserved-keys.md" },
    { ruleDocsPath: "https://example.invalid/rule" },
    { severity: 0 },
  ])
    assert.throws(() => currentRuleDocumentationMessage({ ...original, ...patch }));
  const packet = [{ file: "Probe.vue", messages: [original], errorCount: 1, warningCount: 0 }];
  for (const patch of [
    { errorCount: 0 },
    { warningCount: 1 },
    { errorCount: "1" },
    { foreignCounter: 0 },
    { file: 1 },
    { messages: null },
  ])
    assert.throws(() => currentRuleDocumentationPacket([{ ...packet[0], ...patch }]));
  assert.deepEqual(
    currentRuleDocumentationPacket([
      { file: "Clean.vue", messages: [], errorCount: 0, warningCount: 0 },
    ]),
    [{ file: "Clean.vue", messages: [], errorCount: 0, warningCount: 0 }],
  );
  assert.equal(asString(original.ruleDocsPath), "docs/content/rules/type-and-script.md");
});

void test("every authored public-import and reserved-suppression route projects without losing fields", () => {
  const read = (path: string) =>
    asRecord(
      JSON.parse(
        fs.readFileSync(new URL(`../_fixtures/differential/lint/${path}`, import.meta.url), "utf8"),
      ),
    );
  const verify = (entry: Record<string, unknown>, findings: unknown) => {
    const before = withoutHelp(wholeJson(entry, findings));
    const after = currentRuleDocumentationPacket(before);
    const oldMessages = asArray(asRecord(asArray(before)[0]).messages);
    const newMessages = after[0]!.messages;
    assert.equal(newMessages.length, oldMessages.length);
    for (let i = 0; i < newMessages.length; i++)
      newMessages[i]!.ruleDocsPath = asRecord(oldMessages[i]).ruleDocsPath;
    assert.deepEqual(after, before);
  };
  const publicCorpus = read("public-vue-imports-8090/cases.json");
  const entries = asArray(publicCorpus.cases).map((value) => asRecord(value));
  let publicRoutes = 0;
  for (const entry of entries) {
    verify(entry, entry.diagnostics);
    publicRoutes++;
  }
  const controls = read("public-vue-imports-8090/default-route-controls.json");
  const routeMap = asRecord(
    read("public-vue-imports-8090/default-route-controls-successor.json").routes,
  );
  for (const raw of asArray(controls.cases)) {
    const control = asRecord(raw);
    const entry = entries.find((candidate) => candidate.id === control.id)!;
    const routes = asArray(control.routes).map((value) => asRecord(value));
    for (const route of routes) {
      const selected = routeMap[asString(route.id)] ?? route.id;
      verify(entry, routes.find((candidate) => candidate.id === selected)!.diagnostics);
      publicRoutes++;
    }
  }
  for (const id of [
    "original-element-plus",
    "public-ref",
    "mixed-named",
    "namespace",
    "default-import",
    "private-alias",
  ]) {
    const entry = entries.find((candidate) => candidate.id === id)!;
    verify({ ...entry, source: entry.fixedSource }, entry.fixedDiagnostics);
    publicRoutes++;
  }
  assert.equal(publicRoutes, 63);
  const reserved = read("reserved-suppression-8090/cases.json");
  const successors = asArray(
    read("reserved-suppression-8090/default-route-successors.json").cases,
  ).map((value) => asRecord(value));
  let reservedRoutes = 0;
  for (const [index, raw] of asArray(reserved.cases).entries()) {
    const entry = asRecord(raw),
      successor = successors[index]!;
    assert.equal(successor.id, entry.id);
    for (const field of [
      "defaultDiagnostics",
      "defaultDiagnostics",
      "essentialDiagnostics",
      "defaultDiagnostics",
      "incrementalDiagnostics",
      "offDiagnostics",
      "projectionDiagnostics",
    ]) {
      verify(entry, successor[field]);
      reservedRoutes++;
    }
  }
  const original = read("reserved-suppression-8090/original-vue-select-successor.json");
  original.source = fs.readFileSync(
    new URL(
      `../_fixtures/differential/lint/reserved-suppression-8090/${asString(original.sourceFile)}`,
      import.meta.url,
    ),
    "utf8",
  );
  verify(original, original.diagnostics);
  assert.equal(reservedRoutes + 1, 169);
});

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

await test("remaining CLI documentation contracts preserve all migration and return packet fields", () => {
  let checked = 0;
  const verify = (packet: unknown) => {
    const saved = JSON.stringify(packet);
    const current = currentRuleDocumentationPacket(packet);
    const before = packet as { messages: Record<string, unknown>[] }[];
    for (const [fileIndex, row] of current.entries()) {
      for (const [index, message] of row.messages.entries()) {
        assert.equal(
          message.ruleDocsPath,
          `docs/content/rules/reference/${String(message.ruleId).replace("/", "-")}.md`,
        );
        message.ruleDocsPath = before[fileIndex]!.messages[index]!.ruleDocsPath;
      }
    }
    assert.deepEqual(current, packet);
    assert.equal(JSON.stringify(packet), saved);
    checked++;
  };
  for (const [relative, kind, count] of [
    ["lint-default-migrations/current-range-cases.json", "migration", 12],
    ["differential/lint/return-prefilter-8090/cases.json", "return", 16],
  ] as const) {
    const file = new URL(`../_fixtures/${relative}`, import.meta.url);
    const original = fs.readFileSync(file);
    const corpus = JSON.parse(original.toString("utf8"));
    assert.equal(corpus.cases.length, count);
    for (const row of corpus.cases) {
      const findings =
        kind === "migration"
          ? [
              row.diagnostics,
              row.diagnostics,
              row.essentialDiagnostics,
              row.diagnostics,
              row.diagnostics,
              row.diagnostics,
              [],
              row.baselineDiagnostics,
              ...corpus.configuredVueVersions.map((version: string) =>
                version === "3" ? row.diagnostics : row.baselineDiagnostics,
              ),
              row.essentialDiagnostics,
            ]
          : [row.diagnostics, row.diagnostics, row.diagnostics, row.diagnostics, [], [], []];
      for (const diagnostics of findings) verify(withoutHelp(currentLintOracle(row, diagnostics)));
    }
    assert.ok(fs.readFileSync(file).equals(original));
  }
  assert.equal(checked, 144 + 112);
  const component = {
    ruleId: "vue/require-component-registration",
    ruleDocsPath: "docs/content/rules/vue.md",
    severity: 1,
    message: "Component is used but not explicitly imported",
    line: 3,
    column: 6,
    endLine: 3,
    endColumn: 14,
  };
  verify([{ file: "my-panel.vue", messages: [component], errorCount: 0, warningCount: 1 }]);
  assert.throws(
    () =>
      currentRuleDocumentationMessages([
        { ...component, ruleDocsPath: "docs/content/rules/type-and-script.md" },
      ]),
    /frozen rule documentation URL/u,
  );
  assert.throws(
    () =>
      currentRuleDocumentationMessages([
        {
          ...component,
          ruleDocsPath: "docs/content/rules/reference/vue-require-component-registration.md",
        },
      ]),
    /frozen rule documentation URL/u,
  );
});
