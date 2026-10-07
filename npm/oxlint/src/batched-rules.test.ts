import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import type { Context } from "@oxlint/plugins";
import { afterAll, afterEach, it, vi } from "vite-plus/test";

import type { PatinaDiagnostic, PatinaRuleOptions, PatinaSettings } from "./model.js";
import { clearFileStateCache, getDiagnosticsForRule, getFileState } from "./file-state.ts";
import { getRuleOptions } from "./rule-options.ts";
import { parseRuleSelection } from "./rule-selection.ts";

const { nativeLint } = vi.hoisted(() => ({ nativeLint: vi.fn() }));
vi.mock("./binding.js", () => ({ lintPatina: nativeLint }));

const corpus = new URL(
  "../../../tests/_fixtures/differential/linter/oxlint-batched-selection/",
  import.meta.url,
);
const rules = JSON.parse(fs.readFileSync(new URL("n8n-rules.json", corpus), "utf8")) as Record<
  string,
  unknown
>;
const source = fs.readFileSync(new URL("App.vue.txt", corpus), "utf8");
const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-batched-rules-"));

afterEach(() => {
  clearFileStateCache();
  nativeLint.mockReset();
});
afterAll(() => fs.rmSync(root, { force: true, recursive: true }));

function contextFor(filename: string, settings: PatinaSettings): Context {
  return {
    filename,
    physicalFilename: filename,
    settings: { vize: settings },
    sourceCode: { text: "const items = [1, 2];" },
  } as unknown as Context;
}

function writeFixture(name = "App.vue"): string {
  fs.mkdirSync(root, { recursive: true });
  const filename = path.join(root, name);
  fs.writeFileSync(filename, source);
  return filename;
}

function emptyNativeResult(): void {
  nativeLint.mockReturnValue({
    filename: "App.vue",
    diagnostics: [],
    errorCount: 0,
    warningCount: 0,
  });
}

it("the exact 51-rule n8n selection uses one native call for each of 1,127 SFCs", () => {
  emptyNativeResult();
  const selection = parseRuleSelection(rules)!;
  assert.equal(selection.names.length, 51);

  try {
    for (let index = 0; index < 1127; index += 1) {
      const context = contextFor(writeFixture(`${index}.vue`), { preset: "incremental", rules });
      for (const name of selection.names) {
        getDiagnosticsForRule(
          context,
          getFileState(context),
          name,
          selection.optionsByRule.get(name),
        );
      }
    }

    assert.equal(nativeLint.mock.calls.length, 1127);
    for (const call of nativeLint.mock.calls) {
      assert.deepEqual(call[3], selection.names);
      assert.deepEqual(call[4], {
        attributeHyphenation: "always",
        componentNameInTemplateCasing: "PascalCase",
        sfcElementOrder: { order: ["script", "template", "style"] },
      });
    }
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

it("a selected rule without diagnostics does not repeat the native pass", () => {
  fs.mkdirSync(root, { recursive: true });
  emptyNativeResult();
  const context = contextFor(writeFixture(), {
    preset: "incremental",
    rules: ["vize/vue/no-v-html", "vue/require-v-for-key", "vue/no-v-html"],
  });
  const state = getFileState(context);
  assert.deepEqual(getDiagnosticsForRule(context, state, "vue/no-v-html"), []);
  assert.deepEqual(getDiagnosticsForRule(context, state, "vue/require-v-for-key"), []);
  assert.deepEqual(getDiagnosticsForRule(context, state, "vue/no-v-html"), []);
  assert.equal(nativeLint.mock.calls.length, 1);
  assert.deepEqual(nativeLint.mock.calls[0]?.[3], ["vue/no-v-html", "vue/require-v-for-key"]);
});

it("an override can add a rule or change options without reusing the selected result", () => {
  emptyNativeResult();
  const context = contextFor(writeFixture(), {
    preset: "incremental",
    rules: { "vize/vue/attribute-hyphenation": ["error", "always"] },
  });
  const state = getFileState(context);
  const always: PatinaRuleOptions = { attributeHyphenation: "always" };
  const never: PatinaRuleOptions = { attributeHyphenation: "never" };
  getDiagnosticsForRule(context, state, "vue/attribute-hyphenation", always);
  getDiagnosticsForRule(context, state, "vue/attribute-hyphenation", never);
  getDiagnosticsForRule(context, state, "vue/no-v-html");
  getDiagnosticsForRule(context, state, "vue/attribute-hyphenation", never);
  assert.equal(nativeLint.mock.calls.length, 3);
  assert.deepEqual(nativeLint.mock.calls[1]?.[3], ["vue/attribute-hyphenation"]);
  assert.deepEqual(nativeLint.mock.calls[1]?.[4], never);
  assert.deepEqual(nativeLint.mock.calls[2]?.[3], ["vue/no-v-html"]);
});

it("source and selection option changes each invalidate the native result", () => {
  emptyNativeResult();
  const filename = writeFixture();
  const makeContext = (style: string) =>
    contextFor(filename, {
      preset: "incremental",
      rules: { "vize/vue/attribute-hyphenation": ["warn", style] },
    });
  const first = makeContext("always");
  getDiagnosticsForRule(first, getFileState(first), "vue/attribute-hyphenation", {
    attributeHyphenation: "always",
  });
  fs.writeFileSync(filename, `${source}\n<!-- changed -->`);
  const changed = makeContext("always");
  getDiagnosticsForRule(changed, getFileState(changed), "vue/attribute-hyphenation", {
    attributeHyphenation: "always",
  });
  const optionChange = makeContext("never");
  getDiagnosticsForRule(optionChange, getFileState(optionChange), "vue/attribute-hyphenation", {
    attributeHyphenation: "never",
  });
  assert.equal(nativeLint.mock.calls.length, 3);
});

it("in-place settings edits cannot revive stale selection options", () => {
  emptyNativeResult();
  const rules: Record<string, unknown> = {
    "vize/vue/attribute-hyphenation": ["error", "always"],
  };
  const context = contextFor(writeFixture(), { preset: "incremental", rules });
  const first = getFileState(context);
  getDiagnosticsForRule(context, first, "vue/attribute-hyphenation", {
    attributeHyphenation: "always",
  });
  rules["vize/vue/attribute-hyphenation"] = ["error", "never"];
  const changed = getFileState(context);
  assert.notStrictEqual(first, changed);
  getDiagnosticsForRule(context, changed, "vue/attribute-hyphenation", {
    attributeHyphenation: "never",
  });
  assert.equal(nativeLint.mock.calls.length, 2);
  assert.deepEqual(nativeLint.mock.calls[1]?.[4], { attributeHyphenation: "never" });
});

it("mixed type-aware selections eagerly enable native type checks and report runtime failure once", () => {
  const runtime: PatinaDiagnostic = {
    rule: "type/corsa-runtime",
    severity: "error",
    message: "Corsa unavailable",
    help: null,
    location: { start: { line: 1, column: 1, offset: 0 }, end: { line: 1, column: 2, offset: 1 } },
  };
  nativeLint.mockReturnValue({
    filename: "App.vue",
    diagnostics: [runtime],
    errorCount: 1,
    warningCount: 0,
  });
  const context = contextFor(writeFixture(), {
    preset: "incremental",
    rules: ["vue/no-v-html", "type/require-typed-props", "type/no-unsafe-assignment"],
  });
  const state = getFileState(context);
  assert.deepEqual(getDiagnosticsForRule(context, state, "vue/no-v-html"), []);
  assert.deepEqual(getDiagnosticsForRule(context, state, "type/require-typed-props"), [runtime]);
  assert.deepEqual(getDiagnosticsForRule(context, state, "type/no-unsafe-assignment"), []);
  assert.equal(nativeLint.mock.calls.length, 1);
  assert.equal(nativeLint.mock.calls[0]?.[2].typeAware, true);
});

it("complete Oxlint maps select only Vize entries and ignore disabled rules", () => {
  const selection = parseRuleSelection({
    "no-console": "error",
    "vue/no-v-html": "error",
    "typescript/no-explicit-any": "error",
    "vize/vue/no-v-html": "off",
    "vize/vue/require-v-for-key": [2],
    "vize/vue/attribute-hyphenation": [1, "always"],
  })!;
  assert.deepEqual(selection.names, ["vue/attribute-hyphenation", "vue/require-v-for-key"]);
  assert.deepEqual(selection.options, { attributeHyphenation: "always" });
  assert.deepEqual(getRuleOptions("vue/attribute-hyphenation", ["never"]), {
    attributeHyphenation: "never",
  });
});
