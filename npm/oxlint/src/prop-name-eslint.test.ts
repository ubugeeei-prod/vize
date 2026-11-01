// Genuine packed public host; the five independent findings stay immutable.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, realpathSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
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

interface Host {
  verify(source: string, config: unknown[], options: { filename: string }): unknown[];
  verifyAndFix(source: string, config: unknown[], options: { filename: string }): unknown;
}
const root = fileURLToPath(new URL("../../../", import.meta.url));
const rule = "vue/prop-name-casing";
const publicRule = "vize/" + rule;
const fixtureRoot = join(root, "tests/_fixtures/differential/linter/prop-name-casing-options-6097");
const hash = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
const pins = {
  "cases.json": "4b8e2215a31b6e77be3ac15c90c6199ae39096521f4774e61d60c8c5f0020633",
  "Probe.vue.txt": "39de4482db4f0cc4619373d9fa7717db48877fbb6618626b2f52f4849b04f17b",
  "TemplateOnly.vue.txt": "41ebfae66c03bac7a1874842c6fe93134c34c97b2c554fbf71197fa9d4b483fb",
};
for (const [name, sha] of Object.entries(pins))
  assert.equal(hash(readFileSync(join(fixtureRoot, name))), sha);
const corpus = asRecord(parseJson(readFileSync(join(fixtureRoot, "cases.json"), "utf8")));
assert.deepEqual(corpus.provider, {
  eslint: "10.4.1",
  "eslint-plugin-vue": "10.9.2",
  "vue-eslint-parser": "10.4.1",
});
const cases = asArray(corpus.cases).map((value) => asRecord(value));
assert.deepEqual(
  cases.map((row) => row.id),
  [
    "default-camel",
    "explicit-camel",
    "explicit-snake",
    "template-only-camelCase",
    "template-only-snake_case",
  ],
);
const directory = join(root, "npm/native");
const receiptBytes = readFileSync(nativeHistoryReceipt(directory));
const receipt = asRecord(parseJson(receiptBytes.toString()));
validateNativeHistoryBuild(directory, receipt);
if (process.env.GITHUB_SHA) assert.equal(asRecord(receipt.source).head, process.env.GITHUB_SHA);
const image = realpathSync(asString(asRecord(receipt.frozen).path));
const require = createRequire(import.meta.url);
const envKeys = [
  "VIZE_PREFER_WORKSPACE_BINDING",
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
] as const;
const previous = envKeys.map((key) => [key, process.env[key]] as const);
const observations: unknown[] = [];

function wholeValue(value: unknown, seen = new Map<object, number>()): unknown {
  if (value === undefined) return { kind: "undefined" };
  if (typeof value === "bigint") return { kind: "bigint", decimal: String(value) };
  if (typeof value === "symbol") return { kind: "symbol", description: value.description ?? null };
  if (typeof value === "number" && (!Number.isFinite(value) || Object.is(value, -0)))
    return { kind: "number", spelling: Object.is(value, -0) ? "-0" : String(value) };
  if (value === null || (typeof value !== "object" && typeof value !== "function")) return value;
  const prior = seen.get(value);
  if (prior !== undefined) return { kind: "reference", id: prior };
  const id = seen.size;
  seen.set(value, id);
  return {
    kind: typeof value === "function" ? "function" : Array.isArray(value) ? "array" : "object",
    id,
    ...(typeof value === "function" ? { source: Function.prototype.toString.call(value) } : {}),
    entries: Reflect.ownKeys(value).map((key) => {
      const descriptor = Object.getOwnPropertyDescriptor(value, key);
      assert(descriptor);
      return [
        typeof key === "symbol" ? { symbol: key.description ?? null } : key,
        {
          enumerable: descriptor.enumerable,
          configurable: descriptor.configurable,
          ...("value" in descriptor
            ? { writable: descriptor.writable, value: wholeValue(descriptor.value, seen) }
            : {
                get: wholeValue(Reflect.get(descriptor, "get"), seen),
                set: wholeValue(Reflect.get(descriptor, "set"), seen),
              }),
        },
      ];
    }),
  };
}

function messages(row: Record<string, unknown>, severity: number): unknown[] {
  return asArray(row.findings).map((value) => {
    const finding = asRecord(value);
    assert.equal(finding.ruleId, rule);
    assert.equal(finding.messageId, "invalidCase");
    const message = asString(finding.message);
    assert(message.endsWith("."));
    // The existing CLI oracle specifies this complete Vize public presentation.
    return {
      ruleId: publicRule,
      severity,
      message: `[vize:${rule}] ${message.slice(0, -1)}`,
      line: asNumber(finding.line),
      column: asNumber(finding.column),
      endLine: asNumber(finding.endLine),
      endColumn: asNumber(finding.endColumn),
    };
  });
}

try {
  process.env.VIZE_PREFER_WORKSPACE_BINDING = "1";
  process.env.NAPI_RS_NATIVE_LIBRARY_PATH = image;
  delete process.env.NAPI_RS_FORCE_WASI;
  const entry: unknown = await import(new URL("../dist/eslint.mjs", import.meta.url).href);
  const plugin = asRecord(asRecord(entry).default);
  assert(
    require.cache[image],
    "actual packed public export loaded the authenticated current addon",
  );
  const workspace = createRequire(new URL("../../../package.json", import.meta.url));
  const oracle = createRequire(workspace.resolve("@nuxt/eslint-config"));
  const Linter = asRecord(oracle("eslint")).Linter as new () => Host;
  const parser: unknown = oracle("vue-eslint-parser");
  const providers = Object.fromEntries(
    Object.entries({
      eslint: "10.13.0",
      "vue-eslint-parser": "10.4.1",
      "eslint-plugin-vue": "10.9.2",
    }).map(([name, expected]) => {
      const packagePath = realpathSync(oracle.resolve(name + "/package.json"));
      const version = asRecord(parseJson(readFileSync(packagePath, "utf8"))).version;
      assert.equal(version, expected);
      return [name, { version, packagePath }];
    }),
  );
  const officialRulePath = join(
    realpathSync(oracle.resolve("eslint-plugin-vue/package.json")),
    "..",
    "dist/rules/prop-name-casing.js",
  );
  const officialRule = readFileSync(officialRulePath);
  assert.equal(
    hash(officialRule),
    "b637beea7d7acd891b6fdfa299b7b64e8efc76b07916e8f3e3690e8155700341",
  );
  const custody = {
    source: receipt.source,
    frozen: receipt.frozen,
    receiptSha256: hash(receiptBytes),
    exportSha256: hash(readFileSync(new URL("../dist/eslint.mjs", import.meta.url))),
    providers,
    referenceProviders: corpus.provider,
    fixturePins: pins,
    officialRule: {
      path: officialRulePath,
      sha256: hash(officialRule),
      bytes: officialRule.length,
    },
    sourceFiles: Object.fromEntries(
      ["eslint.ts", "rule-options.ts", "prop-name-options.ts", "plugin.ts", "model.ts"].map(
        (name) => [name, hash(readFileSync(new URL(name, import.meta.url)))],
      ),
    ),
  };
  console.log("vize-prop-name-eslint-source " + JSON.stringify(custody));
  const config = (selection: unknown, hostPlugin: unknown = plugin, severity = "error") => [
    {
      files: ["**/*.vue"],
      languageOptions: { parser },
      plugins: { vize: hostPlugin },
      settings: { vize: { preset: "incremental", helpLevel: "none" } },
      rules:
        selection === null
          ? {}
          : { [publicRule]: selection === undefined ? severity : [severity, selection] },
    },
  ];
  const sourceFor = (row: Record<string, unknown>) => {
    const name = asString(row.source);
    assert(Object.hasOwn(pins, name));
    return readFileSync(join(fixtureRoot, name), "utf8");
  };
  const filename = { filename: "PropNamePublic.vue" };
  const actualRule = asRecord(asRecord(plugin.rules)[rule]);
  // Restore ONLY the immutable baseline metadata. This is a host schema
  // counterfactual, not a claim to execute an entire historical packed product.
  const baselinePlugin = {
    ...plugin,
    rules: {
      ...asRecord(plugin.rules),
      [rule]: { ...actualRule, meta: { ...asRecord(actualRule.meta), schema: [] } },
    },
  };
  let schemaFailure: unknown;
  try {
    new Linter().verify(sourceFor(cases[2]!), config("snake_case", baselinePlugin), filename);
  } catch (error) {
    schemaFailure = error;
  }
  const baseline = {
    kind: "actual-host-baseline-schema-only-counterfactual",
    source: "3eed492a5dcc7aa29d5efd4cece820f29cdee1ae",
    sourcePath: "npm/oxlint/src/plugin.ts",
    sourceBlob: "9a42e0c12acee561a298285e80b681155fe60640",
    sourceSha256: "5d9f9d1c6ee394166728f47e6ff4eb6fbe6eed159589cd5099bc20b45ec79588",
    schema: [],
    options: ["snake_case"],
    error:
      schemaFailure instanceof Error
        ? {
            name: schemaFailure.name,
            message: schemaFailure.message,
            stack: schemaFailure.stack ?? null,
            wholeOwnFields: wholeValue(schemaFailure),
          }
        : wholeValue(schemaFailure),
  };
  observations.push(baseline);
  console.log("vize-prop-name-eslint-baseline " + JSON.stringify(baseline));
  assert(schemaFailure instanceof Error);
  assert(schemaFailure.message.includes(publicRule));
  assert(schemaFailure.message.includes("more than 0 items"));

  const inspect = (
    id: string,
    row: Record<string, unknown>,
    option: unknown,
    severity = "error",
  ) => {
    const original = sourceFor(row);
    const currentConfig = config(option, plugin, severity);
    const before = structuredClone(option);
    const host = new Linter();
    const verify = host.verify(original, currentConfig, filename);
    const fixed = host.verifyAndFix(original, currentConfig, filename);
    const output = asString(asRecord(fixed).output);
    const requery = host.verify(output, currentConfig, filename);
    const secondFix = host.verifyAndFix(output, currentConfig, filename);
    const packet = { id, original, option, severity, verify, fixed, requery, secondFix };
    observations.push(packet);
    console.log("vize-prop-name-eslint-host " + JSON.stringify(packet));
    const expected =
      severity === "off" || option === null ? [] : messages(row, severity === "warn" ? 1 : 2);
    assert.deepEqual(verify, expected, id);
    assert.deepEqual(fixed, { fixed: false, messages: expected, output: original }, id);
    assert.deepEqual(requery, expected, id);
    assert.deepEqual(secondFix, { fixed: false, messages: expected, output: original }, id);
    assert.deepEqual(option, before, "caller-owned policy remains unchanged");
  };
  for (const row of cases) {
    if (row.casing === null) inspect(asString(row.id), row, undefined);
    else {
      const casing = asString(row.casing);
      inspect(asString(row.id) + ":official-tuple", row, casing);
      inspect(asString(row.id) + ":canonical-object", row, { casing });
    }
  }
  inspect("empty-object-reset", cases[0]!, {});
  inspect("selected-snake-warning", cases[2]!, "snake_case", "warn");
  inspect("disabled-snake", cases[2]!, { casing: "snake_case" }, "off");
  inspect("unselected", cases[2]!, null);
  assert.equal(observations.length, 14);
  const proof = {
    schema: "vize.prop-name-casing.eslint-host.v1",
    complete: true,
    node: process.version,
    ...custody,
    observations,
    limits:
      "Five unchanged official finding vectors through both supported public forms. Complete target verify/fix/requery/second-fix packets; no automatic prop rename, new Core/default/reset behavior, ignoreProps support or whole-rule parity claim. Baseline is an explicitly metadata-only counterfactual on the current physical host.",
  };
  writeFileSync(
    join(nativeHistoryDirectory(directory), "prop-name-eslint-host.json"),
    JSON.stringify(proof, null, 2) + "\n",
    { flag: "wx" },
  );
  console.log("vize-prop-name-eslint-custody " + JSON.stringify(proof));
} finally {
  for (const [key, value] of previous) {
    if (value === undefined) delete process.env[key];
    else process.env[key] = value;
  }
}
