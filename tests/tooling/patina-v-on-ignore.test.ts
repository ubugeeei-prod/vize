import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { test } from "node:test";
import {
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
} from "../../npm/native/scripts/formatter-history-build.mts";
import {
  clean,
  corpus,
  expected,
  hash,
  root,
  rowFor,
  rule,
  sourceFor,
  type IgnoreCase,
} from "./support/patina-v-on-ignore-contract.ts";

interface Binding {
  lintPatinaSfc(source: string, options: Record<string, unknown>): unknown;
}
const packets: unknown[] = [];
let nativeBuild: unknown;
let binding: Binding | undefined;
let completedTests = 0;
function record(value: unknown): asserts value is Record<string, unknown> {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
}
function native() {
  if (binding) return binding;
  const nativeDir = path.join(root, "npm/native");
  const bytes = fs.readFileSync(nativeHistoryReceipt(nativeDir));
  const receipt: unknown = JSON.parse(bytes.toString());
  validateNativeHistoryBuild(nativeDir, receipt);
  record(receipt);
  record(receipt.source);
  record(receipt.frozen);
  const image = receipt.frozen.path;
  assert.ok(typeof image === "string");
  if (process.env.GITHUB_SHA) assert.equal(receipt.source.head, process.env.GITHUB_SHA);
  nativeBuild = { receiptSha256: hash(bytes), source: receipt.source, frozen: receipt.frozen };
  const require = createRequire(import.meta.url);
  const keys = [
    "VIZE_PREFER_WORKSPACE_BINDING",
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
  ] as const;
  const previous = keys.map((key) => [key, process.env[key]] as const);
  let loaded: unknown;
  try {
    process.env.VIZE_PREFER_WORKSPACE_BINDING = "1";
    process.env.NAPI_RS_NATIVE_LIBRARY_PATH = image;
    delete process.env.NAPI_RS_FORCE_WASI;
    loaded = require(path.join(nativeDir, "index.js"));
  } finally {
    for (const [key, value] of previous) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
  }
  assert.ok(
    require.cache[fs.realpathSync(image)],
    "load the genuine authenticated physical source addon",
  );
  record(loaded);
  assert.equal(typeof loaded.lintPatinaSfc, "function");
  binding = loaded as unknown as Binding;
  return binding;
}
function persist() {
  const output = path.join(
    root,
    "target/differential/patina-v-on-event-ignore-6097/observations.json",
  );
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(
    output,
    JSON.stringify(
      {
        schema: "vize.patina.v-on-ignore-public.v1",
        complete: completedTests === 2,
        nativeBuild,
        packets,
      },
      null,
      2,
    ) + "\n",
  );
}
function observe(id: string, row: IgnoreCase, extra: Record<string, unknown>) {
  const source = sourceFor(row);
  const options = {
    filename: row.filename,
    preset: "opinionated",
    enabledRules: [rule],
    includeFixes: true,
    helpLevel: "full",
    typeAware: false,
    ...extra,
  };
  const before = JSON.stringify(options);
  let outcome: unknown;
  try {
    outcome = { kind: "returned", value: native().lintPatinaSfc(source, options) };
  } catch (error) {
    outcome =
      error instanceof Error
        ? {
            kind: "threw",
            error: {
              name: error.name,
              message: error.message,
              code: "code" in error ? error.code : null,
            },
          }
        : { kind: "non-error-thrown", value: error };
  }
  const packet = {
    id,
    source,
    sourceSha256: hash(source),
    options,
    optionsBefore: before,
    optionsAfter: JSON.stringify(options),
    outcome,
  };
  packets.push(packet);
  persist();
  console.log("vize-v-on-ignore-packet " + JSON.stringify(packet));
  assert.equal(packet.optionsAfter, before);
  return outcome;
}
const returned = (value: unknown) => ({ kind: "returned", value });
const semanticError = (message: string) => ({
  kind: "threw",
  error: { name: "Error", code: "InvalidArg", message },
});
const policyJson = (policy: unknown) => JSON.stringify({ [rule]: policy });
function flat(row: IgnoreCase): Record<string, unknown> {
  if (row.policy === null) return {};
  if (typeof row.policy === "string") return { vOnEventHyphenation: row.policy };
  return {
    vOnEventHyphenation: row.policy.style ?? "always",
    vOnEventHyphenationIgnore: row.policy.ignore ?? [],
  };
}

test("event ignore preserves thirteen complete authored outcomes through JSON, flat and repeated public calls", () => {
  const observations = corpus.valid.flatMap((row) => {
    const json = row.policy === null ? "{}" : policyJson(row.policy);
    return [
      { row, actual: observe(row.id + ":json", row, { ruleOptionsJson: json }) },
      { row, actual: observe(row.id + ":flat", row, flat(row)) },
      { row, actual: observe(row.id + ":requery", row, { ruleOptionsJson: json }) },
    ];
  });
  assert.equal(observations.length, 39);
  for (const { row, actual } of observations)
    assert.deepEqual(actual, returned(expected(row)), row.id);
  completedTests += 1;
  persist();
});

test("event ignore public configuration owns resets, refusals, disabled selection and exact JS string semantics", () => {
  const original = rowFor("original-default");
  const update = rowFor("original-ignore-update");
  const controls = [
    {
      id: "canonical-over-flat",
      options: {
        vOnEventHyphenation: "never",
        vOnEventHyphenationIgnore: ["row"],
        ruleOptionsJson: policyJson({ ignore: ["update:"] }),
      },
      want: returned(expected(update)),
    },
    {
      id: "canonical-empty-reset",
      options: {
        vOnEventHyphenation: "never",
        vOnEventHyphenationIgnore: ["update:"],
        ruleOptionsJson: policyJson({}),
      },
      want: returned(expected(original)),
    },
    {
      id: "canonical-legacy-reset",
      options: {
        vOnEventHyphenation: "never",
        vOnEventHyphenationIgnore: ["update:"],
        ruleOptionsJson: policyJson("always"),
      },
      want: returned(expected(original)),
    },
    {
      id: "canonical-omission-retains-flat",
      options: {
        vOnEventHyphenation: "always",
        vOnEventHyphenationIgnore: ["update:"],
        ruleOptionsJson: "{}",
      },
      want: returned(expected(update)),
    },
    {
      id: "canonical-skips-flat-blank-validation",
      options: { vOnEventHyphenationIgnore: [""], ruleOptionsJson: policyJson({}) },
      want: returned(expected(original)),
    },
    {
      id: "canonical-skips-flat-duplicate-validation",
      options: { vOnEventHyphenationIgnore: ["row", "row"], ruleOptionsJson: policyJson("always") },
      want: returned(expected(original)),
    },
    {
      id: "canonical-skips-flat-exit-validation",
      options: {
        vOnEventHyphenationIgnore: ["name:exit"],
        ruleOptionsJson: policyJson({ ignore: [] }),
      },
      want: returned(expected(original)),
    },
    {
      id: "off",
      options: { enabledRules: [], ruleOptionsJson: policyJson({ ignore: [] }) },
      want: returned(clean(original)),
    },
    {
      id: "options-do-not-enable-unselected-rule",
      options: {
        preset: "incremental",
        enabledRules: undefined,
        ruleOptionsJson: policyJson({ ignore: [] }),
      },
      want: returned(clean(original)),
    },
    {
      id: "without-fix-field",
      options: { includeFixes: false, vOnEventHyphenationIgnore: ["update:"] },
      want: returned(expected(update, false)),
    },
    {
      id: "flat-null-is-rejected-by-array-binding",
      options: { vOnEventHyphenationIgnore: null },
      want: {
        kind: "threw",
        error: {
          name: "Error",
          code: "ArrayExpected",
          message: "Failed to get Array length on PatinaLintOptionsNapi.vOnEventHyphenationIgnore",
        },
      },
    },
    {
      id: "omitted-json-policy-validates-flat",
      options: { vOnEventHyphenationIgnore: [""], ruleOptionsJson: "{}" },
      want: semanticError("ignore entries must not be blank"),
    },
  ].map((control) => ({ ...control, actual: observe(control.id, original, control.options) }));
  const invalidJson = corpus.invalid.map((row) => ({
    id: row.id,
    actual: observe(row.id + ":invalid-json", original, {
      ruleOptionsJson: policyJson(row.policy),
    }),
  }));
  const malformed = observe("malformed-json", original, { ruleOptionsJson: "{" });
  const flatErrors = [
    { id: "duplicate", ignore: ["update:", "update:"], message: "ignore entries must be unique" },
    { id: "empty", ignore: [""], message: "ignore entries must not be blank" },
    { id: "blank", ignore: [" \t"], message: "ignore entries must not be blank" },
    { id: "bom", ignore: ["\uFEFF"], message: "ignore entries must not be blank" },
    { id: "exit-suffix", ignore: ["name:exit"], message: "ignore entries must not end with :exit" },
  ].map((row) => ({
    ...row,
    actual: observe(row.id + ":invalid-flat", original, { vOnEventHyphenationIgnore: row.ignore }),
  }));
  const repair = observe("valid-after-all-refusals", original, {
    ruleOptionsJson: policyJson({ ignore: [] }),
  });
  for (const row of controls) assert.deepEqual(row.actual, row.want, row.id);
  for (const row of invalidJson) {
    record(row.actual);
    assert.equal(row.actual.kind, "threw", row.id);
    record(row.actual.error);
    assert.equal(row.actual.error.name, "Error", row.id);
    assert.equal(row.actual.error.code, "InvalidArg", row.id);
    assert.equal(typeof row.actual.error.message, "string", row.id);
    // Serde owns the full untagged string/object decoder text; retain it in the
    // actual packet instead of inventing a normalized replacement error.
    assert.match(String(row.actual.error.message), /^Invalid Patina rule options: .+/s, row.id);
  }
  assert.deepEqual(
    malformed,
    semanticError("Invalid Patina rule options: EOF while parsing an object at line 1 column 1"),
  );
  for (const row of flatErrors) assert.deepEqual(row.actual, semanticError(row.message), row.id);
  assert.deepEqual(repair, returned(expected(original)));
  assert.equal(packets.length, 68);
  completedTests += 1;
  persist();
});
