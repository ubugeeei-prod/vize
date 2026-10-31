import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import test from "node:test";
import { execFileSync } from "node:child_process";
import {
  BASE_SOURCE, SOURCE_PARENT, CANDIDATE_SOURCE, OPERATIONS, ORDER, FOCUSED_TEST, SNAPSHOT_TESTS, OWNERSHIP_TESTS, SOURCE_PATHS, TOOLING_PATHS,
  checkAncestry, checkFocused, checkCandidateTests, checkSnapshotRows, parentFromCommit, sourceFor, validateReport,
} from "./controller.ts";
import { adapterIdentity, ORIGINAL_BENCHMARK, SHIPPING_RECIPE, shippingBenchmark, shippingBinary, shippingCargoArtifact } from "./shipping-adapter.ts";

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
    relativeBinaryPath: "target/ci/vize", receipt: { recipe: SHIPPING_RECIPE, binaryPath: "target/ci/vize", sourceRevision: CANDIDATE_SOURCE, binarySha256: image } } };
  return { schemaVersion: 3, status: "passed", commit: CANDIDATE_SOURCE,
    configuration: { freshSamplesPerProduct: 100, warmSamplesPerProduct: 100,
      schedule: "ABBA by round; both products receive byte-identical primary method/params sequences",
      diagnosticQuietMs: 250, diagnosticDeadlineMs: 60_000, diagnosticBarriers: 2 },
    sourceToolReceipt: { ...receipt, sha256: digest(receipt) }, products: { vize: product(), "vue-language-tools": product() },
    oracleEquivalenceReceipts: { fresh: Array(100).fill({}), warm: Array(100).fill({}) },
    cacheResetReceipt: { rows: Array.from({ length: 200 }, () => ({ absenceVerified: true })) },
    wire: { primaryOperationReceipts: { fresh: Array(100).fill({}), warm: [{}] } }, warmupBoundaryReceipts: [{}, {}] };
}
function native(names: readonly string[]): string {
  return `running ${names.length} tests\n${names.map((name) => `test ${name} ... ok`).join("\n")}\n\ntest result: ok. ${names.length} passed; 0 failed; 0 ignored; 0 measured; 100 filtered out; finished in 0.01s\n`;
}
function packetRows(): string[] {
  const rows: string[] = [];
  const range = { start: { line: 1, character: 0 }, end: { line: 1, character: 5 } };
  const reference = { uri: "file:///workspace/Packet.vue", range };
  for (let caseIndex = 0; caseIndex < 12; caseIndex++) for (const newline of ["\n", "\r\n"])
    for (let slot = 0; slot < 4; slot++) for (let operation = 0; operation < 7; operation++) {
      const method = operation < 2 ? "references" : operation < 6 ? "rename" : "prepare";
      const detail = operation < 2 ? { include: operation === 1 } : operation < 6 ? { name: ["renamed", "beta", "outside", "雪"][operation - 2] } : {};
      const result = caseIndex === 10 ? null : method === "prepare" ? range : method === "references" ? [reference] : [{ range, newText: "renamed" }];
      rows.push("structural-snapshot-oracle " + JSON.stringify({ case: caseIndex, newline, slot, method, ...detail,
        original: result, candidate: result, warm: result, warmHit: caseIndex !== 10 }));
    }
  return rows;
}
function mutate(index: number, change: (row: Record<string, unknown>) => void): string {
  const rows = packetRows(); const prefix = "structural-snapshot-oracle ";
  const row: Record<string, unknown> = JSON.parse(rows[index]!.slice(prefix.length)); change(row);
  rows[index] = prefix + JSON.stringify(row); return rows.join("\n");
}

test("finite sources and outer ABBA preserve the original product ABBA", () => {
  assert.equal(sourceFor("B"), BASE_SOURCE); assert.equal(sourceFor("C"), CANDIDATE_SOURCE);
  assert.deepEqual(ORDER, ["B", "C", "C", "B"]);
  checkAncestry(SOURCE_PARENT, BASE_SOURCE, CANDIDATE_SOURCE, [...SOURCE_PATHS], [...TOOLING_PATHS]);
  assert.throws(() => checkAncestry("f".repeat(40), BASE_SOURCE, CANDIDATE_SOURCE, [...SOURCE_PATHS], [...TOOLING_PATHS]));
  assert.throws(() => checkAncestry(SOURCE_PARENT, "f".repeat(40), CANDIDATE_SOURCE, [...SOURCE_PATHS], [...TOOLING_PATHS]));
  assert.throws(() => checkAncestry(SOURCE_PARENT, BASE_SOURCE, BASE_SOURCE, [...SOURCE_PATHS], [...TOOLING_PATHS]));
});
test("foreign, omitted and extra source/controller paths refuse authority", () => {
  for (const changed of [[SOURCE_PATHS[0]], [...SOURCE_PATHS, "Cargo.toml"], [SOURCE_PATHS[0], "unreviewed.rs"]])
    assert.throws(() => checkAncestry(SOURCE_PARENT, BASE_SOURCE, CANDIDATE_SOURCE, changed, [...TOOLING_PATHS]));
  assert.throws(() => checkAncestry(SOURCE_PARENT, BASE_SOURCE, CANDIDATE_SOURCE, [...SOURCE_PATHS], [...TOOLING_PATHS, "tests/performance/lsp-dual-server-benchmark.ts"]));
});
test("immutable parent parsing ignores message text and refuses missing/merge parents", () => {
  assert.equal(parentFromCommit(`tree ${"a".repeat(40)}\nparent ${SOURCE_PARENT}\nauthor synthetic\n\nparent ${BASE_SOURCE}\n`), SOURCE_PARENT);
  assert.throws(() => parentFromCommit(`tree ${"a".repeat(40)}\n\nparent ${SOURCE_PARENT}\n`));
  assert.throws(() => parentFromCommit(`tree ${"a".repeat(40)}\nparent ${SOURCE_PARENT}\nparent ${BASE_SOURCE}\n\nmerge`));
  assert.throws(() => parentFromCommit("tree synthetic\nparent not-a-sha\n\nmessage"));
});
test("both roles require the exact existing focused test with one pass and zero ignores", () => {
  checkFocused(native([FOCUSED_TEST]), "B"); checkFocused(native([FOCUSED_TEST]), "C");
  for (const role of ["B", "C"] as const) {
    assert.throws(() => checkFocused(native([]), role));
    assert.throws(() => checkFocused(native([FOCUSED_TEST]).replace("0 ignored", "1 ignored"), role));
    assert.throws(() => checkFocused(native([FOCUSED_TEST + "_foreign"]), role));
  }
});
test("both roles require all thirteen snapshot and three ownership test names", () => {
  checkCandidateTests(native(SNAPSHOT_TESTS), "snapshot"); checkCandidateTests(native(OWNERSHIP_TESTS), "ownership");
  assert.throws(() => checkCandidateTests(native(SNAPSHOT_TESTS.slice(1)), "snapshot"));
  assert.throws(() => checkCandidateTests(native([...OWNERSHIP_TESTS, OWNERSHIP_TESTS[0]!]), "ownership"));
  assert.throws(() => checkCandidateTests(native(SNAPSHOT_TESTS).replace("13 passed", "12 passed"), "snapshot"));
  assert.throws(() => checkCandidateTests(native(OWNERSHIP_TESTS).replace("0 failed", "1 failed"), "ownership"));
});
test("interleaved nocapture announcements remain exact and summaries cannot be forged", () => {
  const stdout = native(SNAPSHOT_TESTS).replace(`test ${SNAPSHOT_TESTS[0]} ... ok`, `test ${SNAPSHOT_TESTS[0]} ... ${packetRows()[0]}\nok`);
  checkCandidateTests(stdout, "snapshot");
  const focused = native([FOCUSED_TEST]);
  assert.throws(() => checkFocused(focused.replace(`test ${FOCUSED_TEST} ... ok\n`, ""), "B"));
  assert.throws(() => checkFocused(`test ${FOCUSED_TEST} ... ok\n${focused}`, "B"));
  assert.throws(() => checkFocused(focused.replace("0 measured", "1 measured"), "B"));
  assert.throws(() => checkFocused(focused.replace(/test result:[^\n]*/, ""), "B"));
  assert.throws(() => checkFocused(`${focused}test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured;\n`, "B"));
});
test("all672 ordered cold and warm packets include honest nonempty hits", () => {
  const rows = packetRows();
  const stdout = `test ${SNAPSHOT_TESTS[8]} ... ${rows.join("\n")}\nok\n`;
  const proof = checkSnapshotRows(stdout);
  assert.equal(proof.rows, 672); assert.equal(proof.hits, 616); assert.equal(proof.misses, 56);
  assert.deepEqual(proof.positiveWarm, { references: 176, rename: 352, prepare: 88 });
});
test("omitted, duplicated, reordered or malformed row identities refuse", () => {
  const rows = packetRows();
  assert.throws(() => checkSnapshotRows(rows.slice(1).join("\n")));
  assert.throws(() => checkSnapshotRows([...rows, rows[0]!].join("\n")));
  [rows[0], rows[1]] = [rows[1]!, rows[0]!]; assert.throws(() => checkSnapshotRows(rows.join("\n")));
  for (const [key, value] of [["case", 99], ["newline", "\r"], ["slot", 4], ["include", true]] as const)
    assert.throws(() => checkSnapshotRows(mutate(0, (row) => { row[key] = value; })));
  assert.throws(() => checkSnapshotRows(mutate(0, (row) => { row.extra = true; })));
  assert.throws(() => checkSnapshotRows(packetRows().join("\n").replace(/structural-snapshot-oracle [^\n]*/, "structural-snapshot-oracle not-json")));
});
test("whole packet differences, invalid ranges and nonboolean or absent hit proof refuse", () => {
  assert.throws(() => checkSnapshotRows(mutate(0, (row) => { row.warm = []; })));
  assert.throws(() => checkSnapshotRows(mutate(2, (row) => { row.candidate = null; })));
  assert.throws(() => checkSnapshotRows(mutate(6, (row) => { row.original = row.candidate = row.warm = { start: { line: -1, character: 0 }, end: { line: 0, character: 1 } }; })));
  assert.throws(() => checkSnapshotRows(mutate(0, (row) => { row.warmHit = "true"; })));
  assert.throws(() => checkSnapshotRows(mutate(0, (row) => { delete row.warmHit; })));
  const noHits = packetRows().map((line) => line.replace(/"warmHit":true/g, '"warmHit":false')).join("\n");
  assert.throws(() => checkSnapshotRows(noHits));
  const nulls = packetRows().map((line) => {
    const prefix = "structural-snapshot-oracle "; const row: Record<string, unknown> = JSON.parse(line.slice(prefix.length));
    row.original = row.candidate = row.warm = null; return prefix + JSON.stringify(row);
  }).join("\n");
  assert.throws(() => checkSnapshotRows(nulls));
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

test("shipping adapter changes only two custody tokens in the identical immutable B/C benchmark", () => {
  const original = execFileSync("git", ["show", `${BASE_SOURCE}:${ORIGINAL_BENCHMARK}`], { encoding: "utf8" });
  assert.equal(original, execFileSync("git", ["show", `${CANDIDATE_SOURCE}:${ORIGINAL_BENCHMARK}`], { encoding: "utf8" }));
  const adapted = shippingBenchmark(original);
  const identity = adapterIdentity(original, adapted);
  assert.equal(identity.changedCustodyTokens, 2);
  assert.equal(adapted.replace("  SHIPPING_BUILD_RECIPE,\n", "  BUILD_RECIPE,\n")
    .replace("      ? SHIPPING_BUILD_RECIPE\n", "      ? BUILD_RECIPE\n"), original);
  assert.throws(() => shippingBenchmark(original + "\n// drift\n"));
  assert.throws(() => adapterIdentity(original, adapted.replace('"--fresh"', '"--changed"')));
});

test("shipping staging refuses CI/profile/unknown recipes, other paths, source and image drift", () => {
  const receipt = { recipe: SHIPPING_RECIPE, sourceRevision: CANDIDATE_SOURCE, binaryPath: "target/ci/vize", binarySha256: image };
  const binary = { relativeBinaryPath: "target/ci/vize", binarySha256: image, receipt };
  shippingBinary(binary, CANDIDATE_SOURCE, image);
  for (const recipe of ["cargo build --profile ci -p vize", "cargo build --profile ci-opt -p vize --features profiling", "unknown"])
    assert.throws(() => shippingBinary({ ...binary, receipt: { ...receipt, recipe } }, CANDIDATE_SOURCE, image));
  for (const relativeBinaryPath of ["target/release/vize", "target/ci-opt/vize", "../foreign/vize"])
    assert.throws(() => shippingBinary({ ...binary, relativeBinaryPath }, CANDIDATE_SOURCE, image));
  assert.throws(() => shippingBinary(binary, BASE_SOURCE, image));
  assert.throws(() => shippingBinary(binary, CANDIDATE_SOURCE, "f".repeat(64)));
  assert.throws(() => shippingBinary({ ...binary, receipt: { ...receipt, binaryPath: "target/release/vize" } }, CANDIDATE_SOURCE, image));
});

test("actual Cargo release custody requires opt3, no debug assertions and exact default features", () => {
  const profile = { test: false, opt_level: "3", debug_assertions: false };
  const artifact = { target: { name: "vize", kind: ["bin"] }, profile, features: ["default", "glyph", "maestro"] };
  shippingCargoArtifact(artifact);
  for (const change of [{ opt_level: "0" }, { debug_assertions: true }, { test: true }])
    assert.throws(() => shippingCargoArtifact({ ...artifact, profile: { ...profile, ...change } }));
  for (const feature of ["profiling", "micro-profiling", "legacy", "cache-custody"])
    assert.throws(() => shippingCargoArtifact({ ...artifact, features: [...artifact.features, feature] }));
  assert.throws(() => shippingCargoArtifact({ ...artifact, features: ["glyph", "maestro"] }));
});
