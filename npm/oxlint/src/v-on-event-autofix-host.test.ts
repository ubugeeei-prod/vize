import assert from "node:assert/strict";
import { readFileSync, realpathSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
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

function messages(
  row: AutofixCase,
  source: string,
  findings: unknown,
  kebab: (name: string) => string,
): unknown[] {
  const diagnostics = expected(row, source, findings, true, "none", kebab).diagnostics;
  return asArray(findings).map((value, index) => {
    const finding = asRecord(value);
    return {
      ruleId: "vize/" + rule,
      severity: 1,
      message: diagnostics[index]!.message,
      line: asNumber(finding.line),
      column: asNumber(finding.column),
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
  // This filtered package job owns the Nuxt reference stack, not benchmark
  // dependencies. Authenticate the same independent official casing source.
  const pluginRoot = dirname(oracle.resolve("eslint-plugin-vue/package.json"));
  assert.equal(asRecord(oracle("eslint-plugin-vue/package.json")).version, "10.9.2");
  const casingPath = join(pluginRoot, "dist/utils/casing.js");
  const casingSha256 = hash(readFileSync(casingPath));
  assert.equal(casingSha256, "911df4eed3c7d75afac7c6df4a13527d104afdbbd43821993c2b4ece6108f44c");
  const converter = asRecord(oracle(casingPath)).kebabCase;
  assert.equal(typeof converter, "function");
  const kebab = converter as (name: string) => string;
  const providers = Object.fromEntries(
    ["eslint", "vue-eslint-parser", "eslint-plugin-vue"].map((name) => [
      name,
      {
        version: asRecord(oracle(name + "/package.json")).version,
        path: realpathSync(oracle.resolve(name + "/package.json")),
      },
    ]),
  );
  const selected = [
    "LocaleMenu.vue/default",
    "LocaleMenu.vue/always-false",
    "LocaleMenu.vue/always-true",
    "LocaleMenu.vue/reset-empty",
    "LocaleMenu.vue/never-true",
    "LocaleMenu.vue/always-true-ignore",
    "Modifiers.vue/always-true",
    "FinalSigma.vue/always-true",
    "LocaleMenu.vue/off",
  ];
  const exportSha256 = hash(readFileSync(new URL("../dist/eslint.mjs", import.meta.url)));
  const custody = {
    source: receipt.source,
    frozen: receipt.frozen,
    receiptSha256: hash(receiptBytes),
    exportSha256,
    providers,
    expectedCasingSha256: casingSha256,
    sourceFiles: Object.fromEntries(
      ["eslint.ts", "rule-options.ts", "v-on-event-options.ts"].map((path) => [
        path,
        hash(readFileSync(new URL(path, import.meta.url))),
      ]),
    ),
  };
  console.log("vize-v-on-autofix-eslint-host-source " + JSON.stringify(custody));
  const host = new Linter();
  for (const id of selected) {
    const reset = id === "LocaleMenu.vue/reset-empty",
      disabled = id === "LocaleMenu.vue/off";
    const originalCase = cases.find(
      (value) => value.id === (reset || disabled ? "LocaleMenu.vue/default" : id),
    );
    assert(originalCase);
    const row: AutofixCase = reset
      ? { ...originalCase, id, options: ["always", {}] }
      : { ...originalCase, id };
    const original = sourceFor(row),
      reference = referenceFor(row);
    const option = policy(row);
    const config = [
      {
        files: ["**/*.vue"],
        languageOptions: { parser },
        plugins: { vize: plugin },
        settings: { vize: { preset: "incremental", helpLevel: "none" } },
        rules: {
          ["vize/" + rule]: disabled ? "off" : option === undefined ? "warn" : ["warn", option],
        },
      },
    ];
    const filename = { filename: row.file.replace(/\.txt$/u, "") };
    const verify = host.verify(original, config, filename);
    const fixResult = host.verifyAndFix(original, config, filename);
    const output = asString(asRecord(fixResult).output);
    const requery = host.verify(output, config, filename);
    const secondFixResult = host.verifyAndFix(output, config, filename);
    const packet = { id, original, verify, fixResult, requery, secondFixResult };
    observations.push(packet);
    console.log("vize-v-on-autofix-eslint-host " + JSON.stringify(packet));
    assert.deepEqual(
      verify,
      disabled ? [] : messages(row, original, reference.messages, kebab),
      id,
    );
    const expectedOutput = disabled ? original : asString(asRecord(reference.fixResult).output);
    const remaining = disabled ? [] : messages(row, expectedOutput, reference.requery, kebab);
    assert.deepEqual(
      fixResult,
      {
        fixed: disabled ? false : asRecord(reference.fixResult).fixed,
        messages: remaining,
        output: expectedOutput,
      },
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
    ...custody,
    observations,
    limits:
      "Nine actual packed-host cases, including all six original cases plus same-host empty reset, disabled selection and pinned ignore. Whole native/CLI reference corpus has separate controls; the Unicode identity-edit frontier is not generalized into a universal fixed point.",
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
