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
  cleanExpected,
  corpus,
  filename,
  hash,
  inputRoot,
  nativeExpected,
  root,
  rule,
  sourceFor,
  type MacroCase,
} from "./support/patina-macro-order-contract.ts";

interface Binding {
  lintPatinaSfc(source: string, options: Record<string, unknown>): unknown;
}
interface Failure {
  name: string;
  message: string;
  code: unknown;
}
type Outcome =
  | { kind: "returned"; value: unknown }
  | { kind: "threw"; error: Failure }
  | { kind: "non-error-thrown"; value: unknown };
const directory = path.join(root, "target/differential/patina-macro-order-config-6097");
const observation = path.join(directory, "observations.json");
const packets: unknown[] = [];
let nativeBuild: unknown;
let binding: Binding | undefined;
let completedTests = 0;
const persist = () => {
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(
    observation,
    JSON.stringify(
      {
        schema: "vize.patina.macro-order-config-public",
        version: 1,
        complete: completedTests === 4,
        nativeBuild,
        corpusSha256: hash(fs.readFileSync(path.join(inputRoot, "cases.json"))),
        packets,
      },
      null,
      2,
    ) + "\n",
  );
};

function record(value: unknown): asserts value is Record<string, unknown> {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
}

function requireBinding(value: unknown): asserts value is Binding {
  record(value);
  assert.equal(typeof value.lintPatinaSfc, "function");
}

function native(): Binding {
  if (binding) return binding;
  const nativeDir = path.join(root, "npm/native");
  const receiptBytes = fs.readFileSync(nativeHistoryReceipt(nativeDir));
  const receipt: unknown = JSON.parse(receiptBytes.toString());
  validateNativeHistoryBuild(nativeDir, receipt);
  record(receipt);
  record(receipt.source);
  record(receipt.frozen);
  const nativePath = receipt.frozen.path;
  assert.ok(typeof nativePath === "string");
  assert.ok(typeof receipt.frozen.sha256 === "string");
  if (process.env.GITHUB_SHA) assert.equal(receipt.source.head, process.env.GITHUB_SHA);
  nativeBuild = {
    receiptSha256: hash(receiptBytes),
    source: receipt.source,
    frozen: receipt.frozen,
  };
  persist();
  const require = createRequire(import.meta.url);
  const keys = [
    "VIZE_PREFER_WORKSPACE_BINDING",
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
  ] as const;
  const original = keys.map((key) => [key, process.env[key]] as const);
  let loaded: unknown;
  try {
    process.env.VIZE_PREFER_WORKSPACE_BINDING = "1";
    process.env.NAPI_RS_NATIVE_LIBRARY_PATH = nativePath;
    delete process.env.NAPI_RS_FORCE_WASI;
    loaded = require(path.join(nativeDir, "index.js"));
  } finally {
    for (const [key, value] of original) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
  }
  assert.ok(
    require.cache[fs.realpathSync(nativePath)],
    "the public loader must use the authenticated current-source addon",
  );
  requireBinding(loaded);
  binding = loaded;
  return binding;
}

function observe(id: string, row: MacroCase, extra: Record<string, unknown> = {}): Outcome {
  const source = sourceFor(row);
  const options = {
    filename,
    preset: "opinionated",
    enabledRules: [rule],
    typeAware: false,
    includeFixes: true,
    helpLevel: "full",
    ...extra,
  };
  const api = native();
  const optionsBefore = JSON.stringify(options);
  let outcome: Outcome;
  let stack: string | null = null;
  let ownErrorKeys: string[] = [];
  try {
    outcome = { kind: "returned", value: api.lintPatinaSfc(source, options) };
  } catch (error) {
    if (error instanceof Error) {
      stack = error.stack ?? null;
      ownErrorKeys = Object.getOwnPropertyNames(error);
      outcome = {
        kind: "threw",
        error: {
          name: error.name,
          message: error.message,
          code: "code" in error ? error.code : null,
        },
      };
    } else outcome = { kind: "non-error-thrown", value: error };
  }
  const optionsAfter = JSON.stringify(options);
  const packet = {
    id,
    sourceSha256: hash(source),
    source,
    options,
    optionsBefore,
    optionsAfter,
    outcome,
    stack,
    ownErrorKeys,
  };
  packets.push(packet);
  persist();
  console.log("vize-macro-order-config-packet " + JSON.stringify(packet));
  assert.equal(optionsAfter, optionsBefore, id + ": native options must remain unchanged");
  return outcome;
}

const optionsJson = (row: MacroCase) =>
  JSON.stringify(row.options === null ? {} : { [rule]: row.options });
const returned = (value: unknown): Outcome => ({ kind: "returned", value });
const complete = () => {
  completedTests++;
  persist();
};

test("shared macro order JSON options follow six complete public NAPI packets and requery", () => {
  for (const row of corpus.cases) {
    const original = sourceFor(row);
    const expected = returned(nativeExpected(row));
    const json = optionsJson(row);
    assert.deepEqual(observe(row.id + ":json", row, { ruleOptionsJson: json }), expected, row.id);
    assert.deepEqual(
      observe(row.id + ":json-requery", row, { ruleOptionsJson: json }),
      expected,
      row.id,
    );
    const direct = row.options === null ? {} : { defineMacrosOrder: row.options };
    assert.deepEqual(observe(row.id + ":compatibility", row, direct), expected, row.id);
    assert.equal(sourceFor(row), original);
    assert.equal(optionsJson(row), json);
    if (["options-first-configured", "emits-first-reversed"].includes(row.id)) {
      assert.notDeepEqual(observe(row.id + ":unchanged-default-control", row), expected, row.id);
    }
  }
  const defaultRow = corpus.cases[0]!;
  assert.deepEqual(
    observe("default-public-no-fix-field", defaultRow, { includeFixes: false }),
    returned(nativeExpected(defaultRow, false)),
  );
  complete();
});

test("macro order JSON options override compatibility policy and reset to defaults", () => {
  const baseline = corpus.cases[0]!;
  const custom = corpus.cases[1]!;
  assert.equal(custom.id, "options-first-configured");
  assert.deepEqual(
    observe("json-overrides-direct-default", custom, {
      defineMacrosOrder: { order: ["defineProps", "defineEmits"] },
      ruleOptionsJson: optionsJson(custom),
    }),
    returned(nativeExpected(custom)),
  );
  assert.deepEqual(
    observe("json-empty-policy-resets-direct-custom", custom, {
      defineMacrosOrder: custom.options,
      ruleOptionsJson: JSON.stringify({ [rule]: {} }),
    }),
    returned(nativeExpected(baseline)),
  );
  assert.deepEqual(
    observe("absent-json-policy-keeps-direct-custom", custom, {
      defineMacrosOrder: custom.options,
      ruleOptionsJson: "{}",
    }),
    returned(nativeExpected(custom)),
  );
  complete();
});

test("macro order JSON options do not enable an unselected public rule", () => {
  const row = corpus.cases[0]!;
  for (const enabledRules of [[], ["vue/no-deprecated-slot-attribute"]]) {
    assert.deepEqual(
      observe("unselected:" + JSON.stringify(enabledRules), row, {
        preset: "essential",
        enabledRules,
        ruleOptionsJson: JSON.stringify({ [rule]: {} }),
      }),
      returned(cleanExpected()),
    );
  }
  complete();
});

test("macro order JSON options preserve complete invalid-policy errors", () => {
  const row = corpus.cases[0]!;
  const errors = [
    {
      options: { order: ["defineProps", "defineProps"] },
      reason: "macro names in order must be unique",
    },
    { options: { order: [""] }, reason: "macro names in order must not be empty" },
  ];
  // Capture every refusal and the following valid query before comparing any
  // result, so one changed error position cannot hide the remaining controls.
  const observations = errors.map(({ options, reason }) => {
    const json = JSON.stringify({ [rule]: options });
    return { reason, json, outcome: observe("invalid:" + reason, row, { ruleOptionsJson: json }) };
  });
  const json = JSON.stringify({ [rule]: { defineExposeLast: true } });
  const unknown = observe("unexposed-define-expose-last", row, { ruleOptionsJson: json });
  const repair = corpus.cases[1]!;
  const repaired = observe("valid-policy-after-refusals", repair, {
    ruleOptionsJson: optionsJson(repair),
  });
  for (const { reason, json, outcome } of observations) {
    assert.deepEqual(outcome, {
      kind: "threw",
      error: {
        name: "Error",
        code: "InvalidArg",
        message: `Invalid Patina rule options: ${reason} at line 1 column ${json.indexOf("}") + 1}`,
      },
    });
  }
  const column = json.indexOf('"defineExposeLast"') + '"defineExposeLast"'.length;
  assert.deepEqual(unknown, {
    kind: "threw",
    error: {
      name: "Error",
      code: "InvalidArg",
      message: `Invalid Patina rule options: unknown field \`defineExposeLast\`, expected \`order\` at line 1 column ${column}`,
    },
  });
  assert.deepEqual(repaired, returned(nativeExpected(repair)));
  complete();
});
