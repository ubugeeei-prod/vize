import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import test from "node:test";
import {
  BASE_SOURCE, SOURCE_PARENT, SOURCE_GRANDPARENT, CANDIDATE_SOURCE, OPERATIONS, ORDER, PREFIX, SOURCE_PATHS, TOOLING_PATHS,
  checkAncestry, checkFocused, parentFromCommit, sourceFor, validateReport,
} from "./controller.ts";

const digest = (value: unknown): string => createHash("sha256").update(JSON.stringify(value)).digest("hex");
const image = "a".repeat(64);
function fixture(): Record<string, unknown> {
  const range = { start: { line: 1, character: 2 }, end: { line: 1, character: 9 } };
  const uri = "file:///synthetic/App.vue";
  const payload = { authoredUri: uri, completion: { requiredLabel: "message", virtualNameCount: 0 },
    hoverRange: range, definition: [{ uri, range }], references: [{ uri, range }], prepareRenameRange: range,
    rename: { uri, newName: "title", edits: [{ range, newText: "title" }], appliedTextSha256: "b".repeat(64) },
    passedOperations: OPERATIONS };
  const sample = { totalMs: 6, timingsMs: Object.fromEntries(OPERATIONS.map((name) => [name, 1])),
    rss: { totalKiB: 100, processes: 1 }, revisionReceiptSha256: "c".repeat(64),
    oracleReceipt: { ...payload, sha256: digest(payload) } };
  const samples = () => Array.from({ length: 100 }, () => structuredClone(sample));
  const product = () => ({ fresh: samples(), warm: samples(), freshDiagnostics: Array(100).fill({}), warmDiagnostics: [{}] });
  const receipt = { sourceSha: CANDIDATE_SOURCE, vizeBinary: { binarySha256: image,
    relativeBinaryPath: "target/ci/vize", receipt: { recipe: "cargo build --profile ci -p vize", sourceRevision: CANDIDATE_SOURCE, binarySha256: image } } };
  return { schemaVersion: 3, status: "passed", commit: CANDIDATE_SOURCE,
    configuration: { freshSamplesPerProduct: 100, warmSamplesPerProduct: 100,
      schedule: "ABBA by round; both products receive byte-identical primary method/params sequences",
      diagnosticQuietMs: 250, diagnosticDeadlineMs: 60_000, diagnosticBarriers: 2 },
    sourceToolReceipt: { ...receipt, sha256: digest(receipt) }, products: { vize: product(), "vue-language-tools": product() },
    oracleEquivalenceReceipts: { fresh: Array(100).fill({}), warm: Array(100).fill({}) },
    cacheResetReceipt: { rows: Array.from({ length: 200 }, () => ({ absenceVerified: true })) },
    wire: { primaryOperationReceipts: { fresh: Array(100).fill({}), warm: [{}] } }, warmupBoundaryReceipts: [{}, {}] };
}
function focused(role: "B" | "C"): string {
  const names = [`${PREFIX}tests::reactive_destructure_bindings_and_type_property_keys_have_distinct_ranges`];
  if (role === "C") names.push(`${PREFIX}oracle_tests::binding_identifier_visitation_preserves_semantic_range_vectors`);
  return `${names.map((name) => `test ${name} ... ok`).join("\n")}\n\ntest result: ok. ${names.length} passed; 0 failed; 0 ignored; 0 measured; 100 filtered out; finished in 0.01s\n`;
}

test("finite source map and outer ABBA do not replace original product ABBA", () => {
  assert.equal(sourceFor("B"), BASE_SOURCE); assert.equal(sourceFor("C"), CANDIDATE_SOURCE);
  assert.deepEqual(ORDER, ["B", "C", "C", "B"]);
  checkAncestry(SOURCE_PARENT, SOURCE_GRANDPARENT, BASE_SOURCE, CANDIDATE_SOURCE, [...SOURCE_PATHS], [...TOOLING_PATHS]);
  assert.throws(() => checkAncestry("f".repeat(40), SOURCE_GRANDPARENT, BASE_SOURCE, CANDIDATE_SOURCE, [...SOURCE_PATHS], [...TOOLING_PATHS]));
  assert.throws(() => checkAncestry(SOURCE_PARENT, "f".repeat(40), BASE_SOURCE, CANDIDATE_SOURCE, [...SOURCE_PATHS], [...TOOLING_PATHS]));
  assert.throws(() => checkAncestry(SOURCE_PARENT, SOURCE_GRANDPARENT, "f".repeat(40), CANDIDATE_SOURCE, [...SOURCE_PATHS], [...TOOLING_PATHS]));
  assert.throws(() => checkAncestry(SOURCE_PARENT, SOURCE_GRANDPARENT, BASE_SOURCE, BASE_SOURCE, [...SOURCE_PATHS], [...TOOLING_PATHS]));
});
test("foreign, omitted and extra source/controller paths refuse authority", () => {
  for (const changed of [[SOURCE_PATHS[0]], [...SOURCE_PATHS, "Cargo.toml"], [SOURCE_PATHS[0], "unreviewed.rs"]])
    assert.throws(() => checkAncestry(SOURCE_PARENT, SOURCE_GRANDPARENT, BASE_SOURCE, CANDIDATE_SOURCE, changed, [...TOOLING_PATHS]));
  assert.throws(() => checkAncestry(SOURCE_PARENT, SOURCE_GRANDPARENT, BASE_SOURCE, CANDIDATE_SOURCE, [...SOURCE_PATHS], [...TOOLING_PATHS, "tests/performance/lsp-dual-server-benchmark.ts"]));
});
test("immutable parent parsing ignores message text and refuses missing/merge parents", () => {
  assert.equal(parentFromCommit(`tree ${"a".repeat(40)}\nparent ${SOURCE_PARENT}\nauthor synthetic\n\nparent ${BASE_SOURCE}\n`), SOURCE_PARENT);
  assert.throws(() => parentFromCommit(`tree ${"a".repeat(40)}\n\nparent ${SOURCE_PARENT}\n`));
  assert.throws(() => parentFromCommit(`tree ${"a".repeat(40)}\nparent ${SOURCE_PARENT}\nparent ${BASE_SOURCE}\n\nmerge`));
  assert.throws(() => parentFromCommit("tree synthetic\nparent not-a-sha\n\nmessage"));
});
test("focused B1/C2 exact FQNs require real pass counts and zero ignores", () => {
  checkFocused(focused("B"), "B"); checkFocused(focused("C"), "C");
  assert.throws(() => checkFocused(focused("B"), "C"));
  assert.throws(() => checkFocused(focused("C").replace("0 ignored", "1 ignored"), "C"));
  assert.throws(() => checkFocused(focused("C").replace("binding_identifier_visitation", "unrelated_visitation"), "C"));
  assert.throws(() => checkFocused("test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured;", "B"));
});
test("complete synthetic original report validates all 400 six-operation packets", () => {
  validateReport(fixture(), CANDIDATE_SOURCE, image);
});
test("failed, foreign-source and foreign-image reports cannot become fast samples", () => {
  const failed = fixture(); failed.status = "failed"; assert.throws(() => validateReport(failed, CANDIDATE_SOURCE, image));
  assert.throws(() => validateReport(fixture(), BASE_SOURCE, image));
  assert.throws(() => validateReport(fixture(), CANDIDATE_SOURCE, "d".repeat(64)));
});
test("smoke counts, shortened complete arrays and RSS omissions refuse admission", () => {
  const smoke = fixture(); (smoke.configuration as Record<string, unknown>).freshSamplesPerProduct = 1;
  assert.throws(() => validateReport(smoke, CANDIDATE_SOURCE, image));
  const short = fixture(); const products = short.products as Record<string, { warm: unknown[] }>;
  products.vize!.warm.pop(); assert.throws(() => validateReport(short, CANDIDATE_SOURCE, image));
  const missing = fixture(); const p = missing.products as Record<string, { fresh: { rss: unknown }[] }>;
  p.vize!.fresh[0]!.rss = {}; assert.throws(() => validateReport(missing, CANDIDATE_SOURCE, image));
});
test("whole-vector differences and packet hash tampering both refuse", () => {
  const different = fixture(); const p = different.products as Record<string, { fresh: { oracleReceipt: { rename: unknown } }[] }>;
  p.vize!.fresh[0]!.oracleReceipt.rename = { edits: [] };
  assert.throws(() => validateReport(different, CANDIDATE_SOURCE, image));
  const tampered = fixture(); const t = tampered.products as Record<string, { fresh: { oracleReceipt: { sha256: string } }[] }>;
  for (const product of Object.values(t)) product.fresh[0]!.oracleReceipt.sha256 = "0".repeat(64);
  assert.throws(() => validateReport(tampered, CANDIDATE_SOURCE, image));
});
test("missing operations, cache absence and original diagnostic barriers refuse", () => {
  const changed = fixture(); const p = changed.products as Record<string, { warm: { timingsMs: Record<string, number> }[] }>;
  delete p.vize!.warm[0]!.timingsMs.rename;
  assert.throws(() => validateReport(changed, CANDIDATE_SOURCE, image));
  const cache = fixture(); (cache.cacheResetReceipt as { rows: { absenceVerified: boolean }[] }).rows[0]!.absenceVerified = false;
  assert.throws(() => validateReport(cache, CANDIDATE_SOURCE, image));
  const barrier = fixture(); (barrier.configuration as Record<string, unknown>).diagnosticBarriers = 1;
  assert.throws(() => validateReport(barrier, CANDIDATE_SOURCE, image));
});
