import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { test } from "node:test";
import {
  nativeHistoryDirectory,
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
} from "../../npm/native/scripts/formatter-history-build.mts";
import { asRecord, asString, parseJson } from "../../tools/support/json-value.ts";
import {
  clean,
  expected,
  hash,
  literalRows,
  root,
  rowFor,
  rule,
  type ReferenceTagRow,
} from "./support/patina-attribute-ignore-tags-contract.ts";

interface Binding {
  lintPatinaSfc(source: string, options: Record<string, unknown>): unknown;
}
let binding: Binding | undefined;
let custody: unknown;
const packets: unknown[] = [];
const nativeDir = path.join(root, "npm/native");
function native() {
  if (binding) return binding;
  const bytes = fs.readFileSync(nativeHistoryReceipt(nativeDir));
  const receipt = asRecord(parseJson(bytes.toString()));
  validateNativeHistoryBuild(nativeDir, receipt);
  if (process.env.GITHUB_SHA) assert.equal(asRecord(receipt.source).head, process.env.GITHUB_SHA);
  const image = fs.realpathSync(asString(asRecord(receipt.frozen).path));
  const require = createRequire(import.meta.url);
  const keys = [
    "VIZE_PREFER_WORKSPACE_BINDING",
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
  ] as const;
  const previous = keys.map((key) => [key, process.env[key]] as const);
  try {
    process.env.VIZE_PREFER_WORKSPACE_BINDING = "1";
    process.env.NAPI_RS_NATIVE_LIBRARY_PATH = image;
    delete process.env.NAPI_RS_FORCE_WASI;
    const value = asRecord(require(path.join(nativeDir, "index.js")));
    assert(require.cache[image], "load the genuine authenticated physical source addon");
    assert.equal(typeof value.lintPatinaSfc, "function");
    binding = value as unknown as Binding;
    custody = { source: receipt.source, frozen: receipt.frozen, receiptSha256: hash(bytes) };
  } finally {
    for (const [key, value] of previous) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
  }
  return binding;
}
function persist(complete = false) {
  fs.writeFileSync(
    path.join(nativeHistoryDirectory(nativeDir), "attribute-ignore-tags-native.json"),
    JSON.stringify(
      { schema: "vize.attribute-ignore-tags.native.v1", complete, custody, packets },
      null,
      2,
    ) + "\n",
  );
}
function observe(
  id: string,
  row: ReferenceTagRow,
  extra: Record<string, unknown>,
  source = row.source,
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
  console.log("vize-attribute-ignore-tags-native " + JSON.stringify(packet));
  assert.equal(packet.optionsAfter, before);
  return outcome;
}
const returned = (value: unknown) => ({ kind: "returned", value });
const policy = (value: unknown) => ({ ruleOptionsJson: JSON.stringify({ [rule]: value }) });

await test("sixteen literal tag laws preserve complete native DTOs, actual edits and both fixed-point queries", () => {
  const observations = literalRows
    .flatMap((row) => [
      { row, extra: policy({ style: row.style, ignoreTags: row.ignoreTags }) },
      {
        row,
        extra: { attributeHyphenation: row.style, attributeHyphenationIgnoreTags: row.ignoreTags },
      },
    ])
    .map((row, index) => ({
      ...row,
      actual: observe(row.row.id + ":" + index, row.row, row.extra),
    }));
  assert.equal(observations.length, 32);
  for (const { row, extra, actual } of observations) {
    assert.deepEqual(actual, returned(expected(row)), row.id);
    const diagnostics = asRecord(asRecord(actual).value).diagnostics;
    assert(Array.isArray(diagnostics));
    const edits = diagnostics
      .flatMap((item) => {
        const edits = asRecord(asRecord(item).fix).edits;
        assert(Array.isArray(edits));
        return edits.map((value) => {
          const edit = asRecord(value);
          assert.equal(typeof edit.start, "number");
          assert.equal(typeof edit.end, "number");
          assert.equal(typeof edit.newText, "string");
          return edit as { start: number; end: number; newText: string };
        });
      })
      .sort((a, b) => b.start - a.start);
    let fixed = Buffer.from(row.source),
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
    assert.equal(fixed.toString(), row.fixedSource, row.id);
    for (let query = 0; query < 2; query++)
      assert.deepEqual(
        observe(row.id + ":fixed:" + query, row, extra, fixed.toString()),
        returned(clean(row)),
      );
  }
});

await test("tag policies preserve whole-policy resets, canonical precedence, selection and explicit unsupported refusals", () => {
  const original = rowFor("camel-empty"),
    ignored = rowFor("camel-exact");
  const controls = [
    {
      id: "canonical-over-flat",
      options: {
        attributeHyphenation: "never",
        attributeHyphenationIgnoreTags: ["MyWidget"],
        ...policy({ ignoreTags: ["mywidget"] }),
      },
      want: returned(expected(original)),
    },
    {
      id: "empty-object-reset",
      options: {
        attributeHyphenationIgnore: ["foo", "active", "page"],
        attributeHyphenationIgnoreTags: ["MyWidget"],
        ...policy({}),
      },
      want: returned(expected(original)),
    },
    {
      id: "empty-list-reset",
      options: { attributeHyphenationIgnoreTags: ["MyWidget"], ...policy({ ignoreTags: [] }) },
      want: returned(expected(original)),
    },
    {
      id: "legacy-string-reset",
      options: { attributeHyphenationIgnoreTags: ["MyWidget"], ...policy("always") },
      want: returned(expected(original)),
    },
    {
      id: "absent-retains-flat",
      options: { attributeHyphenationIgnoreTags: ["MyWidget"], ruleOptionsJson: "{}" },
      want: returned(expected(ignored)),
    },
    {
      id: "canonical-skips-invalid-flat",
      options: { attributeHyphenationIgnoreTags: ["/^My/"], ...policy({}) },
      want: returned(expected(original)),
    },
    {
      id: "off",
      options: { enabledRules: [], ...policy({ ignoreTags: [] }) },
      want: returned(clean(original)),
    },
    {
      id: "unselected-canonical",
      options: {
        preset: "incremental",
        enabledRules: undefined,
        ...policy({ ignoreTags: ["MyWidget"] }),
      },
      want: returned(clean(original)),
    },
    {
      id: "unselected-flat",
      options: {
        preset: "incremental",
        enabledRules: undefined,
        attributeHyphenationIgnoreTags: ["MyWidget"],
      },
      want: returned(clean(original)),
    },
    {
      id: "without-fixes",
      options: { includeFixes: false, attributeHyphenationIgnoreTags: [] },
      want: returned(expected(original, false)),
    },
  ].map((control) => ({ ...control, actual: observe(control.id, original, control.options) }));
  const refusals = ["/^My/", "/MyWidget/i", "/x/y", "/[/"].map((tag, index) => ({
    actual: observe("unsupported-regexp:" + index, original, {
      attributeHyphenationIgnoreTags: [tag],
    }),
    message:
      "ignoreTags supports literal tag names only; regular-expression policies are unsupported",
  }));
  refusals.push({
    actual: observe("duplicate", original, {
      attributeHyphenationIgnoreTags: ["MyWidget", "MyWidget"],
    }),
    message: "ignoreTags entries must be unique",
  });
  const repair = observe("valid-after-refusals", original, policy({}));
  for (const control of controls) assert.deepEqual(control.actual, control.want, control.id);
  for (const refusal of refusals)
    assert.deepEqual(refusal.actual, {
      kind: "threw",
      error: { name: "Error", code: "InvalidArg", message: refusal.message },
    });
  assert.deepEqual(repair, returned(expected(original)));
  persist(true);
});
