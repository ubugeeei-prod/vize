import assert from "node:assert/strict";
import { readFileSync, realpathSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import {
  nativeHistoryDirectory,
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
} from "../../native/scripts/formatter-history-build.mts";
import {
  asArray,
  asNumber,
  asRecord,
  asString,
  parseJson,
} from "../../../tools/support/json-value.ts";
import {
  cases,
  expected,
  hash,
  policy,
  referenceFor,
  root,
  rule,
  sourceFor,
  type AutofixCase,
} from "../../../tests/tooling/support/patina-v-on-autofix-contract.ts";

interface Host {
  verify(source: string, config: unknown[], options: { filename: string }): unknown[];
  verifyAndFix(source: string, config: unknown[], options: { filename: string }): unknown;
}
const directory = join(root, "npm/native");
const receiptBytes = readFileSync(nativeHistoryReceipt(directory));
const receipt = asRecord(parseJson(receiptBytes.toString()));
validateNativeHistoryBuild(directory, receipt);
if (process.env.GITHUB_SHA) assert.equal(asRecord(receipt.source).head, process.env.GITHUB_SHA);
const image = realpathSync(asString(asRecord(receipt.frozen).path));
const require = createRequire(import.meta.url);
const keys = [
  "VIZE_PREFER_WORKSPACE_BINDING",
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
] as const;
const previous = keys.map((key) => [key, process.env[key]] as const);
const observations: unknown[] = [];

function messages(row: AutofixCase, source: string, findings: unknown): unknown[] {
  const diagnostics = expected(row, source, findings, true, "none").diagnostics;
  return asArray(findings).map((value, index) => {
    const finding = asRecord(value);
    return {
      ruleId: "vize/" + rule,
      severity: 1,
      message: diagnostics[index]!.message,
      line: asNumber(finding.line),
      column: asNumber(finding.column),
      nodeType: null,
      endLine: asNumber(finding.endLine),
      endColumn: asNumber(finding.endColumn),
      ...(finding.fix === undefined ? {} : { fix: finding.fix }),
    };
  });
}

try {
  process.env.VIZE_PREFER_WORKSPACE_BINDING = "1";
  process.env.NAPI_RS_NATIVE_LIBRARY_PATH = image;
  delete process.env.NAPI_RS_FORCE_WASI;
  // Exercise the actual packed public ESLint export, not a test rule or source stub.
  const entry: unknown = await import(new URL("../dist/eslint.mjs", import.meta.url).href);
  const plugin = asRecord(entry).default;
  assert.equal(asRecord(asRecord(asRecord(plugin).rules)[rule]).meta !== undefined, true);
  assert.equal(asRecord(asRecord(asRecord(asRecord(plugin).rules)[rule]).meta).fixable, "code");
  assert(require.cache[image], "the packed host must load the authenticated source addon");
  const workspace = createRequire(new URL("../../../package.json", import.meta.url));
  const oracle = createRequire(workspace.resolve("@nuxt/eslint-config"));
  const Linter = asRecord(oracle("eslint")).Linter as new () => Host;
  const parser: unknown = oracle("vue-eslint-parser");
  const selected = [
    "LocaleMenu.vue/default",
    "LocaleMenu.vue/always-false",
    "LocaleMenu.vue/always-true",
    "LocaleMenu.vue/never-true",
    "Modifiers.vue/always-true",
    "FinalSigma.vue/always-true",
  ];
  for (const id of selected) {
    const row = cases.find((value) => value.id === id);
    assert(row);
    const original = sourceFor(row),
      reference = referenceFor(row);
    const option = policy(row);
    const config = [
      {
        files: ["**/*.vue"],
        languageOptions: { parser },
        plugins: { vize: plugin },
        settings: { vize: { preset: "incremental", helpLevel: "none" } },
        rules: { ["vize/" + rule]: option === undefined ? "warn" : ["warn", option] },
      },
    ];
    const host = new Linter(),
      filename = { filename: row.file.replace(/\.txt$/u, "") };
    const verify = host.verify(original, config, filename);
    const fixResult = host.verifyAndFix(original, config, filename);
    const output = asString(asRecord(fixResult).output);
    const requery = host.verify(output, config, filename);
    const secondFixResult = host.verifyAndFix(output, config, filename);
    const packet = { id, original, verify, fixResult, requery, secondFixResult };
    observations.push(packet);
    console.log("vize-v-on-autofix-eslint-host " + JSON.stringify(packet));
    assert.deepEqual(verify, messages(row, original, reference.messages), id);
    const expectedOutput = asString(asRecord(reference.fixResult).output);
    const remaining = messages(row, expectedOutput, reference.requery);
    assert.deepEqual(
      fixResult,
      { fixed: asRecord(reference.fixResult).fixed, messages: remaining, output: expectedOutput },
      id,
    );
    assert.deepEqual(requery, remaining, id);
    assert.deepEqual(
      secondFixResult,
      { fixed: false, messages: remaining, output: expectedOutput },
      id,
    );
  }
  const proof = {
    schema: "vize.patina.event-autofix-eslint-host.v1",
    complete: true,
    node: process.version,
    source: receipt.source,
    frozen: receipt.frozen,
    receiptSha256: hash(receiptBytes),
    exportSha256: hash(readFileSync(new URL("../dist/eslint.mjs", import.meta.url))),
    observations,
    limits:
      "Six actual packed-host cases. Whole native/CLI reference corpus has separate controls; the Unicode identity-edit frontier is not generalized into a universal fixed point.",
  };
  writeFileSync(
    join(nativeHistoryDirectory(directory), "v-on-event-autofix-host.json"),
    JSON.stringify(proof, null, 2) + "\n",
    { flag: "wx" },
  );
  console.log("vize-v-on-autofix-eslint-host-custody " + JSON.stringify(proof));
} finally {
  for (const [key, value] of previous) {
    if (value === undefined) delete process.env[key];
    else process.env[key] = value;
  }
}
