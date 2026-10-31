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
  referenceFor,
  type AttributeCase,
} from "./support/patina-attribute-ignore-contract.ts";

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
    "target/differential/patina-attribute-hyphenation-ignore-6097/observations.json",
  );
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(
    output,
    JSON.stringify(
      {
        schema: "vize.patina.attribute-ignore-public.v1",
        complete: completedTests === 2,
        nativeBuild,
        packets,
      },
      null,
      2,
    ) + "\n",
  );
}
function observe(
  id: string,
  row: AttributeCase,
  extra: Record<string, unknown>,
  source = sourceFor(row),
) {
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
  console.log("vize-attribute-ignore-packet " + JSON.stringify(packet));
  assert.equal(packet.optionsAfter, before);
  return outcome;
}
const returned = (value: unknown) => ({ kind: "returned", value });
const semanticError = (message: string) => ({
  kind: "threw",
  error: { name: "Error", code: "InvalidArg", message },
});
const policyJson = (policy: unknown) => JSON.stringify({ [rule]: policy });
function flat(row: AttributeCase): Record<string, unknown> {
  if (row.policy === null) return {};
  if (typeof row.policy === "string") return { attributeHyphenation: row.policy };
  return {
    attributeHyphenation: row.policy.style ?? "always",
    attributeHyphenationIgnore: row.policy.ignore ?? [],
  };
}

test("attribute ignore preserves nine complete public outcomes and actual fix/requery through canonical and flat APIs", () => {
  const observations = corpus.valid
    .flatMap((row) => [
      { row, extra: { ruleOptionsJson: policyJson(row.policy) } },
      { row, extra: flat(row) },
    ])
    .map(({ row, extra }, index) => ({
      row,
      extra,
      actual: observe(`${row.id}:${index}`, row, extra),
    }));
  assert.equal(observations.length, 18);
  for (const { row, extra, actual } of observations) {
    assert.deepEqual(actual, returned(expected(row)), row.id);
    record(actual);
    record(actual.value);
    assert(Array.isArray(actual.value.diagnostics));
    const edits = actual.value.diagnostics
      .flatMap((diagnostic: unknown) => {
        record(diagnostic);
        record(diagnostic.fix);
        assert(Array.isArray(diagnostic.fix.edits));
        return diagnostic.fix.edits.map((edit: unknown) => {
          record(edit);
          assert.equal(typeof edit.start, "number");
          assert.equal(typeof edit.end, "number");
          assert.equal(typeof edit.newText, "string");
          return edit as { start: number; end: number; newText: string };
        });
      })
      .sort((a, b) => b.start - a.start);
    let fixed = Buffer.from(sourceFor(row)),
      boundary = fixed.length;
    for (const edit of edits) {
      assert(edit.start <= edit.end && edit.end <= boundary);
      boundary = edit.start;
      fixed = Buffer.concat([
        fixed.subarray(0, edit.start),
        Buffer.from(edit.newText),
        fixed.subarray(edit.end),
      ]);
    }
    assert.equal(fixed.toString(), referenceFor(row).fixedSource);
    for (let query = 0; query < 2; query += 1)
      assert.deepEqual(
        observe(`${row.id}:fixed:${query}`, row, extra, fixed.toString()),
        returned(clean(row)),
      );
  }
  completedTests += 1;
  persist();
});

test("attribute ignore owns canonical resets, strict refusals, disabled selection and state isolation", () => {
  const original = rowFor("always-default"),
    ignored = rowFor("always-substring");
  const controls = [
    {
      id: "canonical-over-flat",
      options: {
        attributeHyphenation: "never",
        attributeHyphenationIgnore: ["foo"],
        ruleOptionsJson: policyJson({ ignore: ["active"] }),
      },
      want: returned(expected(ignored)),
    },
    {
      id: "empty-reset",
      options: {
        attributeHyphenation: "never",
        attributeHyphenationIgnore: ["active"],
        ruleOptionsJson: policyJson({}),
      },
      want: returned(expected(original)),
    },
    {
      id: "legacy-reset",
      options: {
        attributeHyphenation: "never",
        attributeHyphenationIgnore: ["active"],
        ruleOptionsJson: policyJson("always"),
      },
      want: returned(expected(original)),
    },
    {
      id: "absent-retains-flat",
      options: { attributeHyphenationIgnore: ["active"], ruleOptionsJson: "{}" },
      want: returned(expected(ignored)),
    },
    {
      id: "canonical-skips-invalid-flat",
      options: { attributeHyphenationIgnore: [""], ruleOptionsJson: policyJson({}) },
      want: returned(expected(original)),
    },
    {
      id: "off",
      options: { enabledRules: [], ruleOptionsJson: policyJson({ ignore: [] }) },
      want: returned(clean(original)),
    },
    {
      id: "does-not-enable-rule",
      options: { preset: "incremental", enabledRules: undefined, ruleOptionsJson: policyJson({}) },
      want: returned(clean(original)),
    },
    {
      id: "legacy-policy-does-not-enable-rule",
      options: {
        preset: "incremental",
        enabledRules: undefined,
        ruleOptionsJson: policyJson("always"),
      },
      want: returned(clean(original)),
    },
    {
      id: "flat-policy-does-not-enable-rule",
      options: {
        preset: "incremental",
        enabledRules: undefined,
        attributeHyphenation: "always",
        attributeHyphenationIgnore: ["active"],
      },
      want: returned(clean(original)),
    },
    {
      id: "without-fix",
      options: { includeFixes: false, attributeHyphenationIgnore: ["active"] },
      want: returned(expected(ignored, false)),
    },
    {
      id: "null-array",
      options: { attributeHyphenationIgnore: null },
      // Characterize this authenticated addon's provider error completely;
      // the vendor wording is not a permanent public API promise.
      want: {
        kind: "threw",
        error: {
          name: "Error",
          code: "ArrayExpected",
          message: "Failed to get Array length on PatinaLintOptionsNapi.attributeHyphenationIgnore",
        },
      },
    },
  ].map((control) => ({ ...control, actual: observe(control.id, original, control.options) }));
  const invalid = corpus.invalid.map((row) => ({
    id: row.id,
    actual: observe(row.id, original, { ruleOptionsJson: policyJson({ ignore: row.ignore }) }),
  }));
  const flatErrors = [
    { ignore: [""], message: "ignore entries must not be blank" },
    { ignore: ["\uFEFF"], message: "ignore entries must not be blank" },
    { ignore: ["name:exit"], message: "ignore entries must not end with :exit" },
    { ignore: ["active", "active"], message: "ignore entries must be unique" },
  ].map((row, index) => ({
    ...row,
    actual: observe(`flat-invalid:${index}`, original, { attributeHyphenationIgnore: row.ignore }),
  }));
  const malformed = observe("malformed", original, { ruleOptionsJson: "{" });
  const repair = observe("valid-after-errors", original, { ruleOptionsJson: policyJson({}) });
  for (const control of controls) assert.deepEqual(control.actual, control.want, control.id);
  for (const row of invalid) {
    record(row.actual);
    assert.equal(row.actual.kind, "threw", row.id);
    record(row.actual.error);
    assert.equal(row.actual.error.name, "Error");
    assert.equal(row.actual.error.code, "InvalidArg");
    assert.match(String(row.actual.error.message), /^Invalid Patina rule options: .+/s);
  }
  for (const row of flatErrors) assert.deepEqual(row.actual, semanticError(row.message));
  // Keep the complete serde observation for the source-qualified provider.
  assert.deepEqual(
    malformed,
    semanticError("Invalid Patina rule options: EOF while parsing an object at line 1 column 1"),
  );
  assert.deepEqual(repair, returned(expected(original)));
  completedTests += 1;
  persist();
});
