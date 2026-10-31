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
  clean,
  corpus,
  expected,
  filename,
  hash,
  repaired,
  root,
  rule,
  sourceFor,
} from "./support/patina-v-bind-prop-contract.ts";

interface Binding {
  lintPatinaSfc(source: string, options: Record<string, unknown>): unknown;
}
const packets: unknown[] = [];
let binding: Binding | undefined;
let nativeBuild: unknown;
let complete = false;
function object(value: unknown): asserts value is Record<string, unknown> {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
}
function native(): Binding {
  if (binding) return binding;
  const nativeDir = path.join(root, "npm/native");
  const bytes = fs.readFileSync(nativeHistoryReceipt(nativeDir));
  const receipt: unknown = JSON.parse(bytes.toString());
  validateNativeHistoryBuild(nativeDir, receipt);
  object(receipt);
  object(receipt.source);
  object(receipt.frozen);
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
    "load the authenticated physical current-source addon",
  );
  object(loaded);
  assert.equal(typeof loaded.lintPatinaSfc, "function");
  binding = loaded as unknown as Binding;
  return binding;
}
function observe(id: string, source: string, extra: Record<string, unknown>) {
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
  console.log("vize-v-bind-prop-packet " + JSON.stringify(packet));
  assert.equal(packet.optionsAfter, before);
  return outcome;
}
function persist() {
  const output = path.join(
    root,
    "target/differential/patina-v-bind-prop-longform-6097/observations.json",
  );
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(
    output,
    JSON.stringify(
      { schema: "vize.patina.v-bind-prop-public.v1", complete, nativeBuild, packets },
      null,
      2,
    ) + "\n",
  );
}

test("property shorthand reaches nine whole public vectors, exact independent fixes and repeated clean queries", () => {
  const initial = corpus.cases.map((row) => {
    const source = sourceFor(row);
    const options = {
      ruleOptionsJson: JSON.stringify(row.style === null ? {} : { [rule]: row.style }),
    };
    return { row, source, options, actual: observe(row.id, source, options) };
  });
  for (const { row, source, options, actual } of initial) {
    const want = expected(row);
    assert.deepEqual(actual, { kind: "returned", value: want }, row.id);
    object(actual);
    object(actual.value);
    // Apply the returned public edits, not a second unobserved implementation.
    assert.deepEqual(actual.value, want);
    const fixed = repaired(source, actual.value as unknown as ReturnType<typeof expected>);
    assert.equal(fixed, row.reference.fixResult.output, row.id);
    assert.deepEqual(
      observe(row.id + ":fixed", fixed, options),
      { kind: "returned", value: clean() },
      row.id,
    );
    assert.deepEqual(
      observe(row.id + ":fixed-again", fixed, options),
      { kind: "returned", value: clean() },
      row.id,
    );
  }
});

test("longform options preserve disabled selection, no-fix output and strict invalid/reset controls", () => {
  const row = corpus.cases.find((entry) => entry.id === "original-longform")!;
  const source = sourceFor(row);
  const options = { ruleOptionsJson: JSON.stringify({ [rule]: "longform" }) };
  const off = observe("off", source, { ...options, enabledRules: [] });
  const noFix = observe("without-fixes", source, { ...options, includeFixes: false });
  const invalidJson = JSON.stringify({ [rule]: "invalid" });
  const invalid = observe("invalid-style", source, { ruleOptionsJson: invalidJson });
  const reset = observe("reset-default", source, { ruleOptionsJson: "{}" });
  assert.deepEqual(off, { kind: "returned", value: clean() });
  assert.deepEqual(noFix, { kind: "returned", value: expected(row, false) });
  assert.deepEqual(invalid, {
    kind: "threw",
    error: {
      name: "Error",
      message: `Invalid Patina rule options: unknown variant \`invalid\`, expected \`shorthand\` or \`longform\` at line 1 column ${invalidJson.length - 1}`,
      code: "InvalidArg",
    },
  });
  assert.deepEqual(reset, { kind: "returned", value: clean() });
  assert.equal(packets.length, 31);
  complete = true;
  persist();
});
