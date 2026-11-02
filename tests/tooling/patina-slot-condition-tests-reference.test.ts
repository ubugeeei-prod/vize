import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, realpathSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  asArray,
  asNumber,
  asRecord,
  asString,
  parseJson,
} from "../../tools/support/json-value.ts";

interface Host {
  verify(
    source: string,
    config: unknown[],
    options: { filename: string; allowInlineConfig: false },
  ): unknown[];
  verifyAndFix(
    source: string,
    config: unknown[],
    options: { filename: string; allowInlineConfig: false },
  ): unknown;
}
const root = fileURLToPath(new URL("../../", import.meta.url));
const directory = join(root, "tests/_fixtures/differential/linter/slot-existence-conditions-10822");
const rule = "vue/require-slots-as-functions";
const hash = (value: Buffer | string) => createHash("sha256").update(value).digest("hex");
const caseBytes = readFileSync(join(directory, "cases.json"));
assert.equal(hash(caseBytes), "49fe9e0fe0544c6b41f047bb2c67eabcb067f7c5f8e62430085a65b038055da7");
const corpus = asRecord(parseJson(caseBytes.toString()));
const rows = asArray(corpus.cases).map((value) => {
  const row = asRecord(value);
  return { id: asString(row.id), script: asString(row.script), spans: asArray(row.officialSpans) };
});
assert.equal(rows.length, 18);
const originals = [
  {
    id: "original-collapse",
    file: "Collapse.vue.txt",
    sha256: "c1f3d53987b9d6177d123ba26e632f9ca843cbf14a01461ef10100e8631cd11a",
  },
  {
    id: "original-field",
    file: "Field.vue.txt",
    sha256: "5ea1d76f10f26cba807c105cdd3b9906e0d438c9ebfd678ee58a3de45cffa00b",
  },
].map((row) => {
  const bytes = readFileSync(join(directory, row.file));
  assert.equal(hash(bytes), row.sha256);
  return { id: row.id, source: bytes.toString(), sourceSha256: hash(bytes), spans: [] };
});
const provenanceBytes = readFileSync(join(directory, "source-provenance.json"));
assert.equal(
  hash(provenanceBytes),
  "75f0d546557a17ef12f7c79547a98a495c12d4af142ae9608515d0e1e5a19e4e",
);
const provenance = asRecord(parseJson(provenanceBytes.toString()));

test("the pinned Vue rule retains complete direct-condition originals and boundary outcomes", () => {
  assert.equal(process.env.GITHUB_ACTIONS, "true", "reference execution is Actions-only");
  const workspace = createRequire(join(root, "package.json"));
  const require = createRequire(workspace.resolve("@nuxt/eslint-config"));
  const providers = Object.fromEntries(
    Object.entries({
      eslint: "10.13.0",
      "eslint-plugin-vue": "10.9.2",
      "vue-eslint-parser": "10.4.1",
      "@typescript-eslint/parser": "8.65.0",
    }).map(([name, version]) => {
      const path = realpathSync(require.resolve(name + "/package.json"));
      assert.equal(asRecord(parseJson(readFileSync(path, "utf8"))).version, version);
      return [name, { path, version }];
    }),
  );
  const pluginDirectory = dirname(require.resolve("eslint-plugin-vue/package.json"));
  const rulePath = join(pluginDirectory, "dist/rules/require-slots-as-functions.js");
  const helperPath = join(pluginDirectory, "dist/utils/index.js");
  assert.equal(
    hash(readFileSync(rulePath)),
    "94857af48a201bc454f37630d1176a373a60d8fd0f6dc9d22a758a3243d76575",
  );
  assert.equal(
    hash(readFileSync(helperPath)),
    "4727c10651f789ee6308b4311f3694dd302faae9e5b821ae4b31432c3b105925",
  );
  const Linter = asRecord(require("eslint")).Linter as new () => Host;
  const plugin: unknown = require("eslint-plugin-vue"),
    parser: unknown = require("vue-eslint-parser");
  const scriptParser: unknown = require("@typescript-eslint/parser");
  const source = {
    head: process.env.GITHUB_SHA ?? null,
    node: process.version,
    providers,
    rule: { path: rulePath, sha256: hash(readFileSync(rulePath)) },
    helper: { path: helperPath, sha256: hash(readFileSync(helperPath)) },
    originalSources: originals.map(({ id, sourceSha256 }) => ({ id, sourceSha256 })),
    casesSha256: hash(caseBytes),
    provenance,
    inlineConfig:
      "disabled for this isolated-rule host; original non-target directives remain literal input",
    currentRuleSha256: hash(
      readFileSync(join(root, "crates/vize_patina/src/rules/script/require_slots_as_functions.rs")),
    ),
    scope:
      "Historical matrix ESLint10.4.1 is distinct from current explicit-rule ESLint10.13.0. Other logical/arithmetic/comparison/unary/value-use differences remain outside this two-parent correction.",
  };
  console.log("vize-slot-condition-reference-source " + JSON.stringify(source));
  const inputs = [
    ...originals,
    ...rows.map((row) => ({ ...row, source: `<script>\n${row.script}\n</script>\n` })),
  ];
  const config = [
    {
      files: ["**/*.vue"],
      languageOptions: {
        parser,
        parserOptions: { parser: scriptParser, ecmaVersion: 2022, sourceType: "module" },
      },
      plugins: { vue: plugin },
      rules: { [rule]: "error" },
    },
  ];
  const observations = inputs.map((input) => {
    const host = new Linter(),
      // Keep every source byte; run only the explicitly configured target rule.
      filename = { filename: "SlotPresence.vue", allowInlineConfig: false as const };
    const verify = host.verify(input.source, config, filename);
    const fixResult = host.verifyAndFix(input.source, config, filename);
    const output = asString(asRecord(fixResult).output);
    const requery = host.verify(output, config, filename);
    const secondFixResult = host.verifyAndFix(output, config, filename);
    const packet = {
      id: input.id,
      source: input.source,
      sourceSha256: hash(input.source),
      verify,
      fixResult,
      requery,
      secondFixResult,
    };
    console.log("vize-slot-condition-reference-observation " + JSON.stringify(packet));
    return { input, packet };
  });
  for (const { input, packet } of observations) {
    const messages = input.spans.map((value) => {
      const span = asRecord(value),
        needle = asString(span.needle);
      assert.equal(input.source.split(needle).length, 2, input.id);
      const start = input.source.indexOf(needle) + asNumber(span.shift);
      const end = start + asNumber(span.length);
      const position = (offset: number) => {
        const prefix = input.source.slice(0, offset),
          lines = prefix.split("\n");
        return { line: lines.length, column: lines.at(-1)!.length + 1 };
      };
      const from = position(start),
        to = position(end);
      return {
        ruleId: rule,
        severity: 2,
        message: "Property in `$slots` should be used as function.",
        messageId: "unexpected",
        line: from.line,
        column: from.column,
        endLine: to.line,
        endColumn: to.column,
      };
    });
    const fixed = { fixed: false, messages, output: input.source };
    assert.deepEqual(packet.verify, messages, input.id);
    assert.deepEqual(packet.fixResult, fixed, input.id);
    assert.deepEqual(packet.requery, messages, input.id);
    assert.deepEqual(packet.secondFixResult, fixed, input.id);
  }
  console.log(
    "vize-slot-condition-reference-complete " +
      JSON.stringify({
        head: process.env.GITHUB_SHA ?? null,
        cases: observations.length,
        complete: true,
        limits:
          "Whole original Collapse and Field plus distinct authored controls. Native Field fallback-value findings remain outside this two-test slice; no project-matrix, all24FP, public-release or native-history qualification claim.",
      }),
  );
});
