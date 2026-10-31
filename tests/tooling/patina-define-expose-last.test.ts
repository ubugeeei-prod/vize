import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import {
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
} from "../../npm/native/scripts/formatter-history-build.mts";
import {
  corpus,
  directory,
  expected,
  filename,
  hash,
  root,
  rule,
  sourceFor,
  type ExposeCase,
} from "./support/patina-expose-last-contract.ts";

interface Binding {
  lintPatinaSfc(source: string, options: Record<string, unknown>): unknown;
}
const output = path.join(
  root,
  "target/differential/patina-define-expose-last-6097/observations.json",
);
const packets: unknown[] = [];
let nativeBuild: unknown;
let binding: Binding | undefined;
let completedTests = 0;
function record(value: unknown): asserts value is Record<string, unknown> {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
}
function persist() {
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(
    output,
    JSON.stringify(
      {
        schema: "vize.patina.define-expose-last-public.v1",
        complete: completedTests === 3,
        nativeBuild,
        corpusSha256: hash(fs.readFileSync(path.join(directory, "cases.json"))),
        packets,
      },
      null,
      2,
    ) + "\n",
  );
}
function native(): Binding {
  if (binding) return binding;
  const nativeDir = path.join(root, "npm/native");
  const bytes = fs.readFileSync(nativeHistoryReceipt(nativeDir));
  const receipt: unknown = JSON.parse(bytes.toString());
  validateNativeHistoryBuild(nativeDir, receipt);
  record(receipt);
  record(receipt.source);
  record(receipt.frozen);
  const image = receipt.frozen.path;
  assert.equal(typeof image, "string");
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
    "the genuine current-source physical image must be loaded",
  );
  record(loaded);
  assert.equal(typeof loaded.lintPatinaSfc, "function");
  binding = loaded as unknown as Binding;
  return binding;
}
function observe(id: string, row: ExposeCase, extra: Record<string, unknown>) {
  const source = sourceFor(row);
  const options = {
    filename,
    preset: "opinionated",
    enabledRules: [rule],
    includeFixes: true,
    helpLevel: "full",
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
  console.log("vize-define-expose-last-packet " + JSON.stringify(packet));
  assert.equal(packet.optionsAfter, before);
  return outcome;
}

test("defineExposeLast preserves seven complete independent direct/JSON/requery observations", () => {
  const observations = corpus.cases.flatMap((row) => {
    const json = JSON.stringify({ [rule]: row.options });
    return [
      {
        id: row.id + ":direct",
        row,
        actual: observe(row.id + ":direct", row, { defineMacrosOrder: row.options }),
      },
      {
        id: row.id + ":json",
        row,
        actual: observe(row.id + ":json", row, { ruleOptionsJson: json }),
      },
      {
        id: row.id + ":requery",
        row,
        actual: observe(row.id + ":requery", row, { ruleOptionsJson: json }),
      },
    ];
  });
  for (const observation of observations)
    assert.deepEqual(
      observation.actual,
      { kind: "returned", value: expected(observation.row) },
      observation.id,
    );
  completedTests++;
  persist();
});

test("JSON resets the compatibility option and an unselected rule stays disabled", () => {
  const row = corpus.cases[0]!;
  const clean = corpus.cases[1]!;
  const reset = observe("json-empty-resets-direct-true", row, {
    defineMacrosOrder: row.options,
    ruleOptionsJson: JSON.stringify({ [rule]: {} }),
  });
  const override = observe("json-true-overrides-direct-false", row, {
    defineMacrosOrder: { defineExposeLast: false },
    ruleOptionsJson: JSON.stringify({ [rule]: row.options }),
  });
  const disabled = observe("unselected-new-option", row, {
    defineMacrosOrder: row.options,
    enabledRules: [],
  });
  assert.deepEqual(reset, { kind: "returned", value: expected(clean) });
  assert.deepEqual(override, { kind: "returned", value: expected(row) });
  assert.deepEqual(disabled, { kind: "returned", value: expected(clean) });
  completedTests++;
  persist();
});

test("a conflicting direct policy fails completely and the next repaired query stays clean", () => {
  const row = corpus.cases[0]!;
  const repair = corpus.cases[3]!;
  const rejected = observe("direct-conflicting-expose-order", row, {
    defineMacrosOrder: { order: ["defineExpose"], defineExposeLast: true },
  });
  const repaired = observe("valid-query-after-conflicting-policy", repair, {
    defineMacrosOrder: repair.options,
  });
  assert.deepEqual(rejected, {
    kind: "threw",
    error: {
      name: "Error",
      message: "`defineExpose` macro can't be in the `order` array if `defineExposeLast` is true.",
      code: "InvalidArg",
    },
  });
  assert.deepEqual(repaired, { kind: "returned", value: expected(repair) });
  completedTests++;
  persist();
});
