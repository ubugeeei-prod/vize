import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import {
  FORMATTER_SLOT_POLICY_ID,
  FORMATTER_SLOT_POLICY_FIXED,
  loadFormatterSlotPolicy,
} from "../differential/lsp-formatter-slot-policy.ts";
import { inspectLspObservation } from "../differential/lsp-report.ts";
import { referenceObservation, rewriteFrames } from "./support/lsp/differential-reference.ts";
import { asRecord } from "../../tools/support/json-value.ts";

const root = new URL("../../", import.meta.url);
const read = (path: string) => readFileSync(new URL(path, root));
const linterRoot = "tests/_fixtures/differential/linter/formatter-slot-style-policy-6098/";
const lspRoot = "tests/_fixtures/differential/lsp/formatter-slot-style-policy-6098/";
const loaded = loadLspManifest(
  fileURLToPath(new URL("tests/_fixtures/differential/lsp/manifest.json", root)),
);
const fixture = loaded.cases.find((row) => row.id === FORMATTER_SLOT_POLICY_ID);
assert(fixture);

test("slot policy retains the original configured SFC and labels its new whole fix honestly", () => {
  const contract = asRecord(JSON.parse(read(linterRoot + "cases.json").toString()) as unknown);
  const source = asRecord(contract.source),
    reference = asRecord(contract.reference);
  const original = read(linterRoot + "Original.vue.txt");
  assert.equal(source.originalProducer, "699c512b2e8df3c69c2b3f957549a2287a33f372");
  assert.equal(source.originalSha256, createHash("sha256").update(original).digest("hex"));
  // This is the unchanged authored option consumer, not a fabricated issue report.
  const consumer = read("crates/vize/tests/lint_rule_options_casing_cli.rs").toString();
  const body = consumer
    .split("fn lint_rule_options_configure_v_slot_style_by_position()")[1]!
    .split("#[test]")[0]!;
  assert(body.includes('r#"' + original.toString() + '"#'));
  assert.equal(read(linterRoot + "Fixed.vue.txt").toString(), FORMATTER_SLOT_POLICY_FIXED);
  assert.equal(
    reference.fixedSha256,
    createHash("sha256").update(FORMATTER_SLOT_POLICY_FIXED).digest("hex"),
  );
  assert.equal(
    reference.kind,
    "Existing #10241 whole configured inputs; new independent lint-fix verification by ordinary Actions",
  );
  assert.equal(fixture.files[0]!.bytes.toString(), FORMATTER_SLOT_POLICY_FIXED);
  assert.deepEqual(contract.config, {
    formatter: { normalizeDirectiveShorthands: true },
    linter: {
      preset: "incremental",
      ruleOptions: {
        "vue/v-slot-style": { atComponent: "longform", default: "v-slot", named: "longform" },
      },
      rules: { "vue/v-slot-style": "error" },
    },
  });
});

test("only the exact authored three-method null/config contract is admitted", () => {
  const expected: unknown = JSON.parse(read(lspRoot + "response.expected.json").toString());
  assert.equal(loadFormatterSlotPolicy(fixture.data, fixture.files, expected).length, 3);
  for (const mutation of ["id", "method", "initializer", "options", "closed-document"]) {
    const data = structuredClone(fixture.data);
    if (mutation === "id") data.id = "lsp/regression/unregistered";
    else if (mutation === "method") data.method = "textDocument/hover";
    else if (mutation === "initializer") data.initializationOptions!.typecheck = true;
    else if (mutation === "options") data.options!.tabSize = 8;
    else data.openDocument = false;
    assert.throws(() => loadFormatterSlotPolicy(data, fixture.files, expected), mutation);
  }
  for (const mutation of ["short-array", "result", "wrong-method", "extra-field", "position"]) {
    const value = structuredClone(expected) as Record<string, unknown>[];
    if (mutation === "short-array") value.pop();
    else if (mutation === "result") value[0]!.result = [];
    else if (mutation === "wrong-method") value[1]!.method = "textDocument/formatting";
    else if (mutation === "position") asRecord(asRecord(value[2]!.params).position).line = 0;
    else value[2]!.unexpected = true;
    assert.throws(() => loadFormatterSlotPolicy(fixture.data, fixture.files, value), mutation);
  }
  for (const mutation of ["config", "source", "extra-file"]) {
    const files = fixture.files.map((file) => ({ ...file, bytes: Buffer.from(file.bytes) }));
    if (mutation === "config") files[1]!.bytes = Buffer.from("export default { vue: {} };\n");
    else if (mutation === "source") files[0]!.bytes = Buffer.from("<template />\n");
    else files.push({ runtimePath: "extra.mjs", bytes: Buffer.from("unexpected") });
    assert.throws(() => loadFormatterSlotPolicy(fixture.data, files, expected), mutation);
  }
});

test("wire falsifications preserve old bind and sort records without claiming transport execution", () => {
  const expected = [2, 3, 4].map((requestId) => ({ requestId, state: "equal" }));
  assert.deepEqual(inspectLspObservation(fixture, referenceObservation(fixture)), expected);
  const changed = referenceObservation(fixture);
  rewriteFrames(changed, "server", (rows) => {
    rows.find((row) => row.id === 3)!.result = [];
  });
  assert.notDeepEqual(inspectLspObservation(fixture, changed), expected);
  const bind = loaded.cases.find(
    (row) => row.id === "lsp/regression/formatter-bind-style-policy-6098",
  )!;
  assert.deepEqual(inspectLspObservation(bind, referenceObservation(bind)), expected);
  const sort = loaded.cases.find((row) => row.id === "lsp/regression/editor-import-sorting-6098")!;
  assert.equal(sort.requests.length, 1);
  assert(Array.isArray(sort.requests[0]!.result) && sort.requests[0]!.result.length === 1);
});
