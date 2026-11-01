// The frozen 22-case oracle is unchanged; exercise its actual public tuple carrier.
import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
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
import { captureWholeValue } from "../../../tests/tooling/support/sfc-entry-validation-reference.ts";
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
const publicRule = "vize/" + rule;
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
let custody: Record<string, unknown> = {},
  complete = false;
function record(kind: string, packet: unknown) {
  observations.push(packet);
  console.log("vize-v-on-tuple-eslint-" + kind + " " + JSON.stringify(packet));
}
function wholeError(value: unknown): unknown {
  return value instanceof Error
    ? {
        name: value.name,
        message: value.message,
        stack: value.stack ?? null,
        whole: captureWholeValue(value),
      }
    : captureWholeValue(value);
}
try {
  process.env.VIZE_PREFER_WORKSPACE_BINDING = "1";
  process.env.NAPI_RS_NATIVE_LIBRARY_PATH = image;
  delete process.env.NAPI_RS_FORCE_WASI;
  const entry: unknown = await import(new URL("../dist/eslint.mjs", import.meta.url).href);
  const plugin = asRecord(asRecord(entry).default);
  assert(require.cache[image], "the actual packed export loaded the authenticated current addon");
  const workspace = createRequire(new URL("../../../package.json", import.meta.url));
  const oracle = createRequire(workspace.resolve("@nuxt/eslint-config"));
  const Linter = asRecord(oracle("eslint")).Linter as new () => Host;
  const parser: unknown = oracle("vue-eslint-parser");
  const officialPlugin: unknown = oracle("eslint-plugin-vue");
  const providers = Object.fromEntries(
    Object.entries({
      eslint: "10.13.0",
      "eslint-plugin-vue": "10.9.2",
      "vue-eslint-parser": "10.4.1",
    }).map(([name, version]) => {
      const packagePath = realpathSync(oracle.resolve(name + "/package.json"));
      assert.equal(asRecord(parseJson(readFileSync(packagePath, "utf8"))).version, version);
      return [name, { version, packagePath }];
    }),
  );
  const officialDirectory = dirname(oracle.resolve("eslint-plugin-vue/package.json"));
  const rulePath = join(officialDirectory, "dist/rules/v-on-event-hyphenation.js");
  const casingPath = join(officialDirectory, "dist/utils/casing.js");
  assert.equal(
    hash(readFileSync(rulePath)),
    "f91e547b8e0275dac03081fab804190e619fe87ee376302b50814f929c503849",
  );
  assert.equal(
    hash(readFileSync(casingPath)),
    "911df4eed3c7d75afac7c6df4a13527d104afdbbd43821993c2b4ece6108f44c",
  );
  const converter = asRecord(oracle(casingPath)).kebabCase;
  assert.equal(typeof converter, "function");
  const kebab = converter as (name: string) => string;
  custody = {
    source: receipt.source,
    frozen: receipt.frozen,
    receiptSha256: hash(receiptBytes),
    packedExport: {
      path: new URL("../dist/eslint.mjs", import.meta.url).href,
      sha256: hash(readFileSync(new URL("../dist/eslint.mjs", import.meta.url))),
    },
    providers,
    officialRule: { path: rulePath, sha256: hash(readFileSync(rulePath)) },
    officialCasing: { path: casingPath, sha256: hash(readFileSync(casingPath)) },
    sourceFiles: Object.fromEntries(
      ["eslint.ts", "plugin.ts", "rule-options.ts", "v-on-event-options.ts"].map((name) => [
        name,
        hash(readFileSync(new URL(name, import.meta.url))),
      ]),
    ),
    corpus: {
      cases: 22,
      sha256: "c077afcd6f6cacc74a3f18cc6fefe3b2c5ec4182706fbf15d2eeec839df39662",
      authority:
        "unchanged official10.9.2 whole reference packets; original ESLint10.4.1 capture is distinct from current actual host providers",
    },
  };
  console.log("vize-v-on-tuple-eslint-source " + JSON.stringify(custody));
  function config(
    options: readonly unknown[] | null,
    hostPlugin: unknown,
    official = false,
    severity = "warn",
  ) {
    return [
      {
        files: ["**/*.vue"],
        languageOptions: { parser },
        plugins: { [official ? "vue" : "vize"]: hostPlugin },
        settings: { vize: { preset: "incremental", helpLevel: "none" } },
        rules: options === null ? {} : { [official ? rule : publicRule]: [severity, ...options] },
      },
    ];
  }
  function execute(
    row: AutofixCase,
    options: readonly unknown[] | null,
    hostPlugin: unknown,
    official = false,
    severity = "warn",
  ) {
    const source = sourceFor(row),
      host = new Linter(),
      settings = config(options, hostPlugin, official, severity);
    const filename = { filename: row.file.replace(/\.txt$/u, "") };
    const partial: Record<string, unknown> = {};
    try {
      const verify = host.verify(source, settings, filename);
      partial.verify = verify;
      const fixResult = host.verifyAndFix(source, settings, filename);
      partial.fixResult = fixResult;
      const output = asString(asRecord(fixResult).output);
      const requery = host.verify(output, settings, filename);
      partial.requery = requery;
      const secondFixResult = host.verifyAndFix(output, settings, filename);
      return { verify, fixResult, requery, secondFixResult };
    } catch (error) {
      return { failure: wholeError(error), partial };
    }
  }
  function messages(row: AutofixCase, source: string, findings: unknown): unknown[] {
    const diagnostics = expected(row, source, findings, true, "none", kebab).diagnostics;
    return asArray(findings).map((value, index) => {
      const finding = asRecord(value);
      return {
        ruleId: publicRule,
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
  function intended(row: AutofixCase) {
    const reference = referenceFor(row),
      source = sourceFor(row);
    const first = asRecord(reference.fixResult),
      output = asString(first.output);
    const second = asRecord(reference.secondFixResult);
    return {
      verify: messages(row, source, reference.messages),
      fixResult: { fixed: first.fixed, messages: messages(row, output, first.messages), output },
      requery: messages(row, output, reference.requery),
      secondFixResult: {
        fixed: second.fixed,
        messages: messages(row, output, second.messages),
        output: second.output,
      },
    };
  }
  const actualRule = asRecord(asRecord(plugin.rules)[rule]);
  const oldSchema = [
    {
      oneOf: [
        { enum: ["always", "never"] },
        {
          type: "object",
          properties: {
            style: { enum: ["always", "never"] },
            autofix: { type: "boolean" },
            ignore: {
              type: "array",
              uniqueItems: true,
              items: { type: "string", pattern: "^(?!\\s*$)(?![\\s\\S]*:exit$)" },
            },
          },
          additionalProperties: false,
        },
      ],
    },
  ];
  const baselinePlugin = {
    ...plugin,
    rules: {
      ...asRecord(plugin.rules),
      [rule]: { ...actualRule, meta: { ...asRecord(actualRule.meta), schema: oldSchema } },
    },
  };
  const original = cases.find((row) => row.id === "LocaleMenu.vue/always-true");
  assert(original);
  let baselineFailure: unknown;
  try {
    new Linter().verify(sourceFor(original), config(original.options, baselinePlugin), {
      filename: original.file.replace(/\.txt$/u, ""),
    });
  } catch (error) {
    baselineFailure = error;
  }
  record("baseline", {
    kind: "BASELINE-METADATA schema-only counterfactual; not old packed/native execution",
    base: "41a34d040670fdcbc934cb2b224b7ffb985d4d15",
    sourcePath: "npm/oxlint/src/plugin.ts",
    oldSchema,
    baseSchemaSources: [
      {
        path: "npm/oxlint/src/plugin.ts",
        blob: "ce86620c95cff0f711d1aba129b73a0fd08f527a",
        sha256: "fd94bb554a1e3bf6343d0f2899fd68a470722d44c946b1bf79b7c06bb279b1a1",
        bytes: 10942,
      },
      {
        path: "npm/oxlint/src/rule-options.ts",
        blob: "f06ca34c0b46d7e8814ba34a0305ce354bbbfae5",
        sha256: "e7ef5bfb5f16e659cda087d2fabe690482740708f85f8ce39b5d0f92349e631c",
        bytes: 7256,
      },
      {
        path: "npm/oxlint/src/v-on-event-options.ts",
        blob: "ce7b4a6e2d5ec4db1b44e6f862dc66334027ec87",
        sha256: "b83a3bedcc9316d2b6c8847e2ab60e18f02de8c7b193233faac19086f6925fcb",
        bytes: 1644,
      },
    ],
    error: wholeError(baselineFailure),
    options: original.options,
  });
  const packets = cases.map((row) => {
    const option = policy(row);
    const canonicalOptions = option === undefined ? [] : [option];
    const before = structuredClone(row.options);
    const packet = {
      id: row.id,
      source: sourceFor(row),
      options: row.options,
      canonicalOptions,
      tuple: execute(row, row.options, plugin),
      canonical: execute(row, canonicalOptions, plugin),
      official: execute(row, row.options, officialPlugin, true),
      before,
    };
    record("host", packet);
    return { row, packet };
  });
  const defaultRow = cases.find((row) => row.id === "LocaleMenu.vue/default");
  assert(defaultRow);
  const extra = [
    { id: "canonical-empty-reset", options: [{}], severity: "warn" },
    { id: "tuple-empty-policy", options: ["always", {}], severity: "warn" },
    { id: "disabled-tuple", options: ["always", { autofix: true }], severity: "off" },
    { id: "unselected", options: null, severity: "warn" },
  ].map((row) => {
    const packet = {
      ...row,
      source: sourceFor(defaultRow),
      actual: execute(defaultRow, row.options, plugin, false, row.severity),
    };
    record("control", packet);
    return packet;
  });
  const refusals = [
    ["always", { ignoreTags: [] }],
    ["always", { unknown: true }],
    ["always", { autofix: "true" }],
    ["always", { ignore: null }],
    [{ style: "always" }, { autofix: true }],
    ["always", {}, {}],
    ["always", { style: "never" }],
  ].map((options) => {
    let failure: unknown;
    try {
      new Linter().verify(sourceFor(defaultRow), config(options, plugin), {
        filename: defaultRow.file.replace(/\.txt$/u, ""),
      });
    } catch (error) {
      failure = error;
    }
    record("refusal", { options, error: wholeError(failure) });
    return failure;
  });
  // Every complete reference and both actual carrier outcomes are retained before verdict.
  assert(baselineFailure instanceof Error);
  assert(baselineFailure.message.includes(publicRule));
  assert(baselineFailure.message.includes("more than 1 items"));
  for (const { row, packet } of packets) {
    const reference = referenceFor(row);
    assert.deepEqual(
      packet.official,
      {
        verify: reference.messages,
        fixResult: reference.fixResult,
        requery: reference.requery,
        secondFixResult: reference.secondFixResult,
      },
      row.id + ": whole official packet",
    );
    assert.deepEqual(packet.tuple, intended(row), row.id + ": official tuple");
    assert.deepEqual(packet.canonical, intended(row), row.id + ": canonical policy");
    assert.deepEqual(packet.options, packet.before, row.id + ": immutable caller policy");
  }
  for (const packet of extra) {
    const source = sourceFor(defaultRow);
    const none = {
      verify: [],
      fixResult: { fixed: false, messages: [], output: source },
      requery: [],
      secondFixResult: { fixed: false, messages: [], output: source },
    };
    assert.deepEqual(
      packet.actual,
      packet.severity === "off" || packet.options === null ? none : intended(defaultRow),
      packet.id,
    );
  }
  for (const failure of refusals) {
    assert(failure instanceof Error);
    assert(failure.message.includes(publicRule));
  }
  assert.equal(observations.length, 34);
  complete = true;
} finally {
  const proof = {
    schema: "vize.patina.event-tuple-eslint-host.v1",
    complete,
    node: process.version,
    ...custody,
    observations,
    limits:
      "22 unchanged original provider vectors: 44 actual tuple/canonical host outcomes plus 22 current official outcomes, reset/off/refusals. Unicode identity-edit/circular frontier retained whole; no universal fixedpoint claim. New tuple supports existing autofix/ignore only; ignoreTags stays unsupported. Schema-only baseline metadata clone is not an old product execution. No Core/native/default change, whole-matrix or public-release claim.",
  };
  const invocation = mkdtempSync(join(nativeHistoryDirectory(directory), "v-on-event-tuple-"));
  const destination = join(invocation, "v-on-event-tuple-eslint-host.json");
  writeFileSync(destination, JSON.stringify(proof, null, 2) + "\n", { flag: "wx" });
  console.log("vize-v-on-tuple-eslint-custody " + JSON.stringify(proof));
  console.log("vize-v-on-tuple-eslint-path " + destination);
  for (const [key, value] of previous) {
    if (value === undefined) delete process.env[key];
    else process.env[key] = value;
  }
}
