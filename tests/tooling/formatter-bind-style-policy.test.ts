import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import {
  FORMATTER_BIND_POLICY_ID,
  FORMATTER_BIND_POLICY_FIXED,
  loadFormatterBindPolicy,
} from "../differential/lsp-formatter-bind-policy.ts";
import { inspectLspObservation } from "../differential/lsp-report.ts";
import { referenceObservation, rewriteFrames } from "./support/lsp/differential-reference.ts";
import { asRecord } from "../../tools/support/json-value.ts";

const root = new URL("../../", import.meta.url);
const read = (path: string) => readFileSync(new URL(path, root));
const linterRoot = "tests/_fixtures/differential/linter/formatter-bind-style-policy-6098/";
const originalRoot = "tests/_fixtures/differential/linter/v-bind-prop-longform-6097/";
const loaded = loadLspManifest(
  fileURLToPath(new URL("tests/_fixtures/differential/lsp/manifest.json", root)),
);
const fixture = loaded.cases.find((row) => row.id === FORMATTER_BIND_POLICY_ID);
assert(fixture);

test("host interaction inputs retain the original complete lint input and fixed reference bytes", () => {
  const contract = asRecord(
    JSON.parse(read(linterRoot + "cases.json").toString("utf8")) as unknown,
  );
  const source = asRecord(contract.source);
  const original = read(originalRoot + "Original.vue.txt");
  const referenceBytes = read(originalRoot + "cases.json");
  assert.equal(source.revision, "36d76a54ebdef00cfa83f8788224d0c69dad59a9");
  assert.equal(source.originalSha256, createHash("sha256").update(original).digest("hex"));
  assert.equal(source.referenceSha256, createHash("sha256").update(referenceBytes).digest("hex"));
  assert.deepEqual(read(linterRoot + "Original.vue.txt"), original);
  const corpus = JSON.parse(referenceBytes.toString("utf8")) as {
    cases: { id: string; reference: { fixResult: { output: string } } }[];
  };
  const fixed = corpus.cases.find((row) => row.id === "original-longform")!.reference.fixResult
    .output;
  assert.equal(fixed, FORMATTER_BIND_POLICY_FIXED);
  assert.equal(read(linterRoot + "Fixed.vue.txt").toString("utf8"), fixed);
  assert.equal(fixture.files[0]!.bytes.toString("utf8"), fixed);
  assert.deepEqual(contract.config, {
    formatter: { normalizeDirectiveShorthands: true },
    linter: {
      preset: "incremental",
      ruleOptions: { "vue/v-bind-style": "longform" },
      rules: { "vue/v-bind-style": "error" },
    },
  });
});

test("the new exact-ID host contract rejects changed methods, missing nulls and configuration", () => {
  const expected: unknown = JSON.parse(
    read(
      "tests/_fixtures/differential/lsp/formatter-bind-style-policy-6098/response.expected.json",
    ).toString("utf8"),
  );
  assert.equal(loadFormatterBindPolicy(fixture.data, fixture.files, expected).length, 3);
  for (const mutation of ["id", "method", "initializer", "options", "closed-document"]) {
    const data = structuredClone(fixture.data);
    if (mutation === "id") data.id = "lsp/regression/unregistered";
    else if (mutation === "method") data.method = "textDocument/hover";
    else if (mutation === "initializer") data.initializationOptions!.typecheck = true;
    else if (mutation === "options") data.options!.tabSize = 8;
    else data.openDocument = false;
    assert.throws(() => loadFormatterBindPolicy(data, fixture.files, expected), mutation);
  }
  for (const mutation of ["short-array", "result", "wrong-method", "extra-field"]) {
    const value = structuredClone(expected) as Record<string, unknown>[];
    if (mutation === "short-array") value.pop();
    else if (mutation === "result") value[0]!.result = [];
    else if (mutation === "wrong-method") value[1]!.method = "textDocument/formatting";
    else value[2]!.unexpected = true;
    assert.throws(() => loadFormatterBindPolicy(fixture.data, fixture.files, value), mutation);
  }
  for (const mutation of ["config", "source", "extra-file"]) {
    const files = fixture.files.map((file) => ({ ...file, bytes: Buffer.from(file.bytes) }));
    if (mutation === "config")
      files[1]!.bytes = Buffer.from("export default { vue: { formatter: {} } };\n");
    else if (mutation === "source") files[0]!.bytes = Buffer.from("<template />\n");
    else files.push({ runtimePath: "extra.mjs", bytes: Buffer.from("unexpected") });
    assert.throws(() => loadFormatterBindPolicy(fixture.data, files, expected), mutation);
  }
});

test("whole wire falsifications distinguish null refusals from edits without claiming product execution", () => {
  const expected = [2, 3, 4].map((requestId) => ({ requestId, state: "equal" }));
  const reference = referenceObservation(fixture);
  assert.deepEqual(inspectLspObservation(fixture, reference), expected);
  const changed = referenceObservation(fixture);
  rewriteFrames(changed, "server", (rows) => {
    rows.find((row) => row.id === 3)!.result = [];
  });
  assert.notDeepEqual(inspectLspObservation(fixture, changed), expected);
  const sort = loaded.cases.find((row) => row.id === "lsp/regression/editor-import-sorting-6098")!;
  assert.equal(sort.requests.length, 1);
  assert(Array.isArray(sort.requests[0]!.result) && sort.requests[0]!.result.length === 1);
});
