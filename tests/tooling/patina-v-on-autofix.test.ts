import assert from "node:assert/strict";
import { mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import { test } from "node:test";
import {
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
} from "../../npm/native/scripts/formatter-history-build.mts";
import { asRecord, asString, parseJson } from "../../tools/support/json-value.ts";
import {
  apply,
  cases,
  expected,
  flat,
  hash,
  policy,
  provider,
  referenceFor,
  root,
  rule,
  sourceFor,
  type AutofixCase,
} from "./support/patina-v-on-autofix-contract.ts";

interface Binding {
  lintPatinaSfc(source: string, options: Record<string, unknown>): unknown;
}
let binding: Binding | undefined;
let nativeBuild: unknown;
const packets: unknown[] = [];
let completedTests = 0;
function native(): Binding {
  if (binding) return binding;
  const directory = join(root, "npm/native");
  const bytes = readFileSync(nativeHistoryReceipt(directory));
  const receipt = asRecord(parseJson(bytes.toString()));
  validateNativeHistoryBuild(directory, receipt);
  const image = asString(asRecord(receipt.frozen).path);
  if (process.env.GITHUB_SHA) assert.equal(asRecord(receipt.source).head, process.env.GITHUB_SHA);
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
    loaded = require(join(directory, "index.js"));
  } finally {
    for (const [key, value] of previous) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
  }
  assert(require.cache[realpathSync(image)], "actual authenticated source addon required");
  assert.equal(typeof asRecord(loaded).lintPatinaSfc, "function");
  binding = loaded as Binding;
  return binding;
}
function persist() {
  const directory = join(root, "target/differential/patina-v-on-event-autofix-6097");
  mkdirSync(directory, { recursive: true });
  writeFileSync(
    join(directory, "observations.json"),
    JSON.stringify(
      {
        schema: "vize.patina.event-autofix-public.v1",
        complete: completedTests === 3,
        nativeBuild,
        packets,
        limits:
          "Native complete packets and output bytes are compared. The pinned official Unicode identity-edit fixed:true frontier is preserved separately; CLI/content changed flags are not presented as equivalent.",
      },
      null,
      2,
    ) + "\n",
  );
}
function observe(id: string, row: AutofixCase, source: string, extra: Record<string, unknown>) {
  const options: Record<string, unknown> = {
    filename: row.file.replace(/\.txt$/u, ""),
    preset: "opinionated",
    enabledRules: [rule],
    includeFixes: true,
    helpLevel: "full",
    typeAware: false,
    ...extra,
  };
  const before = JSON.stringify(options);
  const keysBefore = Reflect.ownKeys(options);
  const input = () => ({
    own: Object.hasOwn(options, "vOnEventHyphenationAutofix"),
    type: typeof options.vOnEventHyphenationAutofix,
    ...(options.vOnEventHyphenationAutofix === undefined
      ? {}
      : { value: options.vOnEventHyphenationAutofix }),
  });
  const autofixInput = input();
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
    keysBefore,
    keysAfter: Reflect.ownKeys(options),
    autofixInput,
    autofixAfter: input(),
    outcome,
  };
  packets.push(packet);
  persist();
  console.log("vize-v-on-autofix-packet " + JSON.stringify(packet));
  assert.equal(packet.optionsAfter, before);
  assert.deepEqual(packet.keysAfter, keysBefore);
  assert.deepEqual(packet.autofixAfter, autofixInput);
  return asRecord(outcome);
}
const jsonPolicy = (row: AutofixCase) =>
  policy(row) === undefined ? "{}" : JSON.stringify({ [rule]: policy(row) });
function returned(outcome: ReturnType<typeof observe>) {
  assert.equal(outcome.kind, "returned");
  return outcome.value;
}

test("public native JSON and flat policies retain all twenty-two whole authored packets and edit envelopes", () => {
  for (const row of cases) {
    const source = sourceFor(row),
      reference = referenceFor(row);
    for (const [route, options] of [
      ["json", { ruleOptionsJson: jsonPolicy(row) }],
      ["flat", flat(row)],
    ] as const) {
      const actual = returned(observe(row.id + ":" + route, row, source, options));
      assert.deepEqual(actual, expected(row, source, reference.messages), row.id + ":" + route);
    }
  }
  completedTests++;
  persist();
});

test("actual native edits match whole independent fixed outputs and retain the Unicode no-op diagnostic frontier", () => {
  const reference = provider();
  for (const row of cases) {
    const original = sourceFor(row),
      pinned = referenceFor(row),
      filename = row.file.replace(/\.txt$/u, "");
    let source = original;
    for (let pass = 0; pass < 10; pass++) {
      const findings = reference.linter.verify(source, reference.config(row), { filename });
      const actual = returned(
        observe(row.id + ":fix-pass" + pass, row, source, { ruleOptionsJson: jsonPolicy(row) }),
      );
      assert.deepEqual(actual, expected(row, source, findings), row.id);
      const next = apply(source, actual);
      if (next === source) break;
      source = next;
    }
    assert.equal(source, asRecord(pinned.fixResult).output, row.id);
    const final = returned(
      observe(row.id + ":final", row, source, { ruleOptionsJson: jsonPolicy(row) }),
    );
    assert.deepEqual(final, expected(row, source, pinned.requery), row.id);
    assert.equal(apply(source, final), source, row.id);
    assert.deepEqual(
      final,
      expected(row, source, asRecord(pinned.secondFixResult).messages),
      row.id,
    );
  }
  completedTests++;
  persist();
});

test("public option precedence, omissions, actual marshalling and invalid canonical values remain distinct", () => {
  const row = cases.find((row) => row.id === "LocaleMenu.vue/always-true")!;
  const baseline = cases.find((row) => row.id === "LocaleMenu.vue/default")!;
  const source = sourceFor(row),
    positive = referenceFor(row),
    negative = referenceFor(baseline);
  for (const [id, options, findings] of [
    [
      "canonical-empty-over-flat",
      { vOnEventHyphenationAutofix: true, ruleOptionsJson: JSON.stringify({ [rule]: {} }) },
      negative.messages,
    ],
    [
      "canonical-false-over-flat",
      {
        vOnEventHyphenationAutofix: true,
        ruleOptionsJson: JSON.stringify({ [rule]: { autofix: false } }),
      },
      negative.messages,
    ],
    [
      "canonical-true-over-flat",
      { vOnEventHyphenationAutofix: false, ruleOptionsJson: jsonPolicy(row) },
      positive.messages,
    ],
    [
      "flat-retained-without-rule",
      { vOnEventHyphenationAutofix: true, ruleOptionsJson: "{}" },
      positive.messages,
    ],
    ["flat-undefined", { vOnEventHyphenationAutofix: undefined }, negative.messages],
    // ?bool uses napi-rs's optional-value marshaller; record the real null boundary.
    ["flat-null-is-optional-unset", { vOnEventHyphenationAutofix: null }, negative.messages],
  ] as const)
    assert.deepEqual(
      returned(observe(id, row, source, options)),
      expected(row, source, findings),
      id,
    );
  const hidden = returned(
    observe("include-fixes-false", row, source, {
      ruleOptionsJson: jsonPolicy(row),
      includeFixes: false,
    }),
  );
  assert.deepEqual(hidden, expected(row, source, positive.messages, false));
  const noHelp = returned(
    observe("help-none", row, source, { ruleOptionsJson: jsonPolicy(row), helpLevel: "none" }),
  );
  assert.deepEqual(noHelp, expected(row, source, positive.messages, true, "none"));
  assert.deepEqual(
    returned(
      observe("disabled-stays-disabled", row, source, {
        ruleOptionsJson: jsonPolicy(row),
        enabledRules: [],
      }),
    ),
    { filename: "LocaleMenu.vue", errorCount: 0, warningCount: 0, diagnostics: [] },
  );
  for (const value of [null, "true", 1, [], {}]) {
    const outcome = observe("canonical-invalid-" + JSON.stringify(value), row, source, {
      ruleOptionsJson: JSON.stringify({ [rule]: { autofix: value } }),
    });
    assert.equal(outcome.kind, "threw");
    assert.equal(asRecord(outcome.error).code, "InvalidArg");
  }
  for (const value of ["true", 1, [], {}]) {
    const outcome = observe("flat-invalid-" + JSON.stringify(value), row, source, {
      vOnEventHyphenationAutofix: value,
    });
    assert.equal(outcome.kind, "threw");
    assert.equal(asRecord(outcome.error).code, "InvalidArg");
  }
  const unknown = observe("canonical-unknown-key", row, source, {
    ruleOptionsJson: JSON.stringify({ [rule]: { autofixTypo: true } }),
  });
  assert.equal(unknown.kind, "threw");
  assert.equal(asRecord(unknown.error).code, "InvalidArg");
  completedTests++;
  persist();
});
