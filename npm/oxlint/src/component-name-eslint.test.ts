// The existing Casing.vue source stays whole; only its public option transport changes.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  nativeHistoryDirectory,
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
} from "../../native/scripts/formatter-history-build.mts";
import { asRecord, asString, parseJson } from "../../../tools/support/json-value.ts";

interface Host {
  verify(source: string, config: unknown[], options: { filename: string }): unknown[];
  verifyAndFix(source: string, config: unknown[], options: { filename: string }): unknown;
}
type Casing = "PascalCase" | "kebab-case";
const root = fileURLToPath(new URL("../../../", import.meta.url));
const rule = "vue/component-name-in-template-casing",
  publicRule = "vize/" + rule;
const hash = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
const pins = {
  "crates/vize/tests/lint_rule_options_casing_cli.rs":
    "531c092139d0f1a2a44cc1e2838acb9e7a7cf5b3139284110cfe0282277b2792",
  "tests/tooling/support/n8n-lint-parity.ts":
    "a7b7222d1de601a9bd084f2771b6f0680f0f003d16f04dfd296ad009aaf563bd",
  "tests/_fixtures/n8n-adoption.json":
    "b54df7b9100ce00e815a734589476649a0a41bb47ac6e286c89b89c03fa98b4f",
};
for (const [name, expected] of Object.entries(pins))
  assert.equal(hash(readFileSync(join(root, name))), expected);
const rust = readFileSync(join(root, Object.keys(pins)[0]!), "utf8");
const prefix = '"src/Casing.vue",\n        r#"';
const start = rust.indexOf(prefix);
assert(start >= 0);
const end = rust.indexOf('"#,', start + prefix.length);
assert(end > start);
const original = rust.slice(start + prefix.length, end);
assert(original.startsWith("<template>\n") && original.endsWith("</script>\n"));
const template = original.slice(0, original.indexOf("\n\n<script")) + "\n";
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
let custody: Record<string, unknown> = {},
  completed = false;

function whole(value: unknown, seen = new Map<object, number>()): unknown {
  if (value === undefined) return { kind: "undefined" };
  if (typeof value === "bigint") return { kind: "bigint", decimal: String(value) };
  if (typeof value === "symbol") return { kind: "symbol", description: value.description ?? null };
  if (typeof value === "number" && (!Number.isFinite(value) || Object.is(value, -0)))
    return { kind: "number", spelling: String(value) };
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
            ? { writable: descriptor.writable, value: whole(descriptor.value, seen) }
            : {
                get: whole(Reflect.get(descriptor, "get"), seen),
                set: whole(Reflect.get(descriptor, "set"), seen),
              }),
        },
      ];
    }),
  };
}
function record(kind: string, value: unknown): void {
  observations.push(value);
  console.log("vize-component-name-eslint-" + kind + " " + JSON.stringify(value));
}

try {
  process.env.VIZE_PREFER_WORKSPACE_BINDING = "1";
  process.env.NAPI_RS_NATIVE_LIBRARY_PATH = image;
  delete process.env.NAPI_RS_FORCE_WASI;
  const entry: unknown = await import(new URL("../dist/eslint.mjs", import.meta.url).href);
  const plugin = asRecord(asRecord(entry).default);
  assert(require.cache[image], "actual packed export loaded the authenticated current addon");
  const workspace = createRequire(new URL("../../../package.json", import.meta.url));
  const oracle = createRequire(workspace.resolve("@nuxt/eslint-config"));
  const Linter = asRecord(oracle("eslint")).Linter as new () => Host;
  const parser: unknown = oracle("vue-eslint-parser"),
    scriptParser: unknown = oracle("@typescript-eslint/parser");
  const officialPlugin: unknown = oracle("eslint-plugin-vue");
  const providers = Object.fromEntries(
    Object.entries({
      eslint: "10.13.0",
      "vue-eslint-parser": "10.4.1",
      "eslint-plugin-vue": "10.9.2",
      "@typescript-eslint/parser": "8.65.0",
    }).map(([name, expected]) => {
      const packagePath = realpathSync(oracle.resolve(name + "/package.json"));
      const version = asRecord(parseJson(readFileSync(packagePath, "utf8"))).version;
      assert.equal(version, expected);
      return [name, { version, packagePath }];
    }),
  );
  const officialDirectory = join(
    realpathSync(oracle.resolve("eslint-plugin-vue/package.json")),
    "..",
    "dist",
  );
  const rulePath = join(officialDirectory, "rules/component-name-in-template-casing.js");
  const ruleBytes = readFileSync(rulePath);
  assert.equal(hash(ruleBytes), "573aadd95f4235ddfae08e955d86a427f947fba29230a334ff3bbc3d3c38c2e6");
  const regexpPath = join(officialDirectory, "utils/regexp.js");
  custody = {
    source: receipt.source,
    frozen: receipt.frozen,
    receiptSha256: hash(receiptBytes),
    exportSha256: hash(readFileSync(new URL("../dist/eslint.mjs", import.meta.url))),
    providers,
    originalSource: { source: original, sha256: hash(original) },
    templateOnlyControl: { source: template, sha256: hash(template) },
    pins,
    officialRule: { path: rulePath, bytes: ruleBytes.length, sha256: hash(ruleBytes) },
    officialRegexp: { path: regexpPath, sha256: hash(readFileSync(regexpPath)) },
    sourceFiles: Object.fromEntries(
      ["eslint.ts", "rule-options.ts", "component-name-options.ts", "plugin.ts", "model.ts"].map(
        (name) => [name, hash(readFileSync(new URL(name, import.meta.url)))],
      ),
    ),
  };
  console.log("vize-component-name-eslint-source " + JSON.stringify(custody));
  const filename = { filename: "ComponentNamePublic.vue" };
  function config(
    options: readonly unknown[] | null,
    hostPlugin: unknown,
    official = false,
    severity = "error",
  ): unknown[] {
    const prefix = official ? "vue/" : "vize/vue/";
    return [
      {
        files: ["**/*.vue"],
        languageOptions: {
          parser,
          parserOptions: { parser: scriptParser, ecmaVersion: 2022, sourceType: "module" },
        },
        plugins: { [official ? "vue" : "vize"]: hostPlugin },
        settings: { vize: { preset: "incremental", helpLevel: "none" } },
        rules:
          options === null
            ? {}
            : { [prefix + "component-name-in-template-casing"]: [severity, ...options] },
      },
    ];
  }
  function inspect(
    id: string,
    source: string,
    options: readonly unknown[] | null,
    referenceOptions: readonly unknown[] | null,
    casing: Casing,
    finding: boolean,
    severity = "error",
  ): void {
    const before = structuredClone(options),
      current = config(options, plugin, false, severity);
    const host = new Linter(),
      reference = new Linter(),
      upstream = config(referenceOptions, officialPlugin, true, severity);
    const verify = host.verify(source, current, filename);
    const fixed = host.verifyAndFix(source, current, filename),
      output = asString(asRecord(fixed).output);
    const requery = host.verify(output, current, filename),
      secondFix = host.verifyAndFix(output, current, filename);
    const referenceVerify = reference.verify(source, upstream, filename);
    const referenceFixed = reference.verifyAndFix(source, upstream, filename),
      referenceOutput = asString(asRecord(referenceFixed).output);
    const referenceRequery = reference.verify(referenceOutput, upstream, filename),
      referenceSecondFix = reference.verifyAndFix(referenceOutput, upstream, filename);
    record("host", {
      id,
      source,
      options,
      referenceOptions,
      severity,
      verify,
      fixed,
      requery,
      secondFix,
      official: {
        verify: referenceVerify,
        fixed: referenceFixed,
        requery: referenceRequery,
        secondFix: referenceSecondFix,
      },
    });
    const active = finding && severity !== "off" && options !== null;
    const tag = casing === "PascalCase" ? "my-widget" : "MyWidget",
      replacement = casing === "PascalCase" ? "MyWidget" : "my-widget";
    const tagStart = source.indexOf("<" + tag + " />");
    assert(tagStart >= 0);
    const nameStart = tagStart + 1,
      nameEnd = nameStart + tag.length;
    const expectedOutput = active
      ? source.slice(0, nameStart) + replacement + source.slice(nameEnd)
      : source;
    const expected = active
      ? [
          {
            ruleId: publicRule,
            severity: severity === "warn" ? 1 : 2,
            message: "Component should use " + casing,
            line: casing === "PascalCase" ? 2 : 3,
            column: 3,
            endLine: casing === "PascalCase" ? 2 : 3,
            endColumn: 3 + tag.length + 4,
            fix: { range: [nameStart, nameEnd], text: replacement },
          },
        ]
      : [];
    assert.deepEqual(verify, expected, id);
    assert.deepEqual(fixed, { fixed: active, messages: [], output: expectedOutput }, id);
    assert.deepEqual(requery, [], id);
    assert.deepEqual(secondFix, { fixed: false, messages: [], output: expectedOutput }, id);
    assert.equal(referenceVerify.length, active ? 1 : 0, id);
    assert.deepEqual(referenceFixed, { fixed: active, messages: [], output: expectedOutput }, id);
    assert.deepEqual(referenceRequery, [], id);
    assert.deepEqual(
      referenceSecondFix,
      { fixed: false, messages: [], output: expectedOutput },
      id,
    );
    assert.deepEqual(options, before, "caller policy is not mutated");
  }
  const actualRule = asRecord(asRecord(plugin.rules)[rule]);
  const oldSchema = [
    {
      oneOf: [
        { enum: ["PascalCase", "kebab-case"] },
        {
          type: "object",
          properties: {
            casing: { enum: ["PascalCase", "kebab-case"] },
            registeredComponentsOnly: { type: "boolean" },
            globals: { type: "array", items: { type: "string" } },
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
  let baselineError: unknown;
  try {
    new Linter().verify(
      template,
      config(["PascalCase", { registeredComponentsOnly: false }], baselinePlugin),
      filename,
    );
  } catch (error) {
    baselineError = error;
  }
  record("baseline", {
    kind: "actual-host-baseline-metadata-only-counterfactual",
    source: "ec8474e0ea9ed05062542c57c857f36b9467ac15",
    sourcePath: "npm/oxlint/src/plugin.ts",
    sourceBlob: "87d07c3ff2c68352bfb8de16095deeafa8c82a42",
    sourceSha256: "5f5ed0762a9c9e8fc8c35a4bd5b7e7811111bcd35b6f8496bb83571f300fbf6e",
    schema: oldSchema,
    error:
      baselineError instanceof Error
        ? {
            name: baselineError.name,
            message: baselineError.message,
            stack: baselineError.stack ?? null,
            wholeOwnFields: whole(baselineError),
          }
        : whole(baselineError),
  });
  assert(baselineError instanceof Error);
  assert(baselineError.message.includes(publicRule));
  assert(baselineError.message.includes("more than 1 items"));
  inspect("original-default", original, [], [], "PascalCase", true);
  inspect("original-pascal-style", original, ["PascalCase"], ["PascalCase"], "PascalCase", true);
  inspect("original-kebab-style", original, ["kebab-case"], ["kebab-case"], "kebab-case", true);
  inspect("template-default-registered-only", template, [], [], "PascalCase", false);
  const policies = [
    {
      id: "all-pascal",
      casing: "PascalCase" as const,
      policy: { registeredComponentsOnly: false },
      finding: true,
    },
    {
      id: "all-kebab",
      casing: "kebab-case" as const,
      policy: { registeredComponentsOnly: false },
      finding: true,
    },
    {
      id: "literal-global-pascal",
      casing: "PascalCase" as const,
      policy: { globals: ["MyWidget"] },
      finding: true,
    },
    {
      id: "literal-global-kebab",
      casing: "kebab-case" as const,
      policy: { globals: ["my-widget"] },
      finding: true,
    },
    {
      id: "registered-global",
      casing: "PascalCase" as const,
      policy: { registeredComponentsOnly: true, globals: ["MyWidget"] },
      finding: true,
    },
    { id: "empty-policy-default", casing: "PascalCase" as const, policy: {}, finding: false },
    { id: "empty-globals", casing: "PascalCase" as const, policy: { globals: [] }, finding: false },
  ];
  for (const row of policies) {
    const tuple = [row.casing, row.policy];
    inspect(row.id + ":official-tuple", template, tuple, tuple, row.casing, row.finding);
    inspect(
      row.id + ":canonical-object",
      template,
      [{ casing: row.casing, ...row.policy }],
      tuple,
      row.casing,
      row.finding,
    );
  }
  inspect("canonical-empty-reset-original", original, [{}], [], "PascalCase", true);
  inspect("canonical-empty-reset-template", template, [{}], [], "PascalCase", false);
  inspect(
    "warning-tuple",
    template,
    ["PascalCase", { globals: ["MyWidget"] }],
    ["PascalCase", { globals: ["MyWidget"] }],
    "PascalCase",
    true,
    "warn",
  );
  inspect(
    "disabled-tuple",
    template,
    ["PascalCase", { registeredComponentsOnly: false }],
    ["PascalCase", { registeredComponentsOnly: false }],
    "PascalCase",
    true,
    "off",
  );
  inspect("unselected", template, null, null, "PascalCase", false);
  for (const [id, options] of [
    ["unknown-policy", ["PascalCase", { unknown: true }]],
    ["unsupported-ignores", ["PascalCase", { ignores: [] }]],
    ["unsupported-regexp-global", ["PascalCase", { globals: ["/^My/i"] }]],
    ["canonical-plus-second", [{ casing: "PascalCase" }, { globals: [] }]],
    ["third-option", ["PascalCase", {}, {}]],
    ["bad-boolean", ["PascalCase", { registeredComponentsOnly: "false" }]],
  ] as const) {
    let failure: unknown;
    try {
      new Linter().verify(template, config(options, plugin), filename);
    } catch (error) {
      failure = error;
    }
    record("refusal", {
      id,
      options,
      error:
        failure instanceof Error
          ? {
              name: failure.name,
              message: failure.message,
              stack: failure.stack ?? null,
              wholeOwnFields: whole(failure),
            }
          : whole(failure),
    });
    assert(failure instanceof Error, id);
    assert(failure.message.includes(publicRule), id);
  }
  assert.equal(observations.length, 30);
  completed = true;
} finally {
  const proof = {
    schema: "vize.component-name-casing.eslint-host.v1",
    complete: completed,
    node: process.version,
    ...custody,
    observations,
    limits:
      "Existing Casing.vue whole source and explicitly derived template-only control; real packed ESLint and pinned official verify/fix/requery/second-fix packets. Finite literal globals/registeredComponentsOnly transport only. Official regex globals and ignores are unsupported by the new tuple. Existing canonical globals semantics retained; no new Core/default/reset, broad parity, direct Oxlint CLI or public release claim. Baseline metadata-only clone is not an old packed-product execution.",
  };
  const invocation = mkdtempSync(join(nativeHistoryDirectory(directory), "component-name-eslint-"));
  const destination = join(invocation, "component-name-eslint-host.json");
  writeFileSync(destination, JSON.stringify(proof, null, 2) + "\n", { flag: "wx" });
  console.log("vize-component-name-eslint-custody " + JSON.stringify(proof));
  console.log("vize-component-name-eslint-path " + destination);
  for (const [key, value] of previous) {
    if (value === undefined) delete process.env[key];
    else process.env[key] = value;
  }
}
