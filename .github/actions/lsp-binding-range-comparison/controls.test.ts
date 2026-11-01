import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import test from "node:test";
import { execFileSync } from "node:child_process";
import {
  BASE_SOURCE, CANDIDATE_SOURCE, OPERATIONS, ORDER, FOCUSED_TEST, SNAPSHOT_TESTS, OWNERSHIP_TESTS, SOURCE_PATHS, TOOLING_PATHS,
  checkAmbientOverrides, checkAncestry, checkFocused, checkCandidateTests, checkSnapshotRows, parentFromCommit, sourceFor, validateReport,
} from "./controller.ts";
import { adapterIdentity, ORIGINAL_BENCHMARK, SHIPPING_RECIPE, shippingBenchmark, shippingBinary, shippingCargoArtifact } from "./shipping-adapter.ts";

import { DEFINITION_CASES, DEFINITION_TESTS, DEFINITION_FIXTURE_SHA256, DEFINITION_BOUND_PATHS, SDK_CRATES,
  checkDefinitionExecution, definitionEnvironment, definitionPairs, libraryArtifact, sdkArtifacts } from "./definition-parity.ts";

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
  checkAncestry(BASE_SOURCE, CANDIDATE_SOURCE, [...SOURCE_PATHS], [...TOOLING_PATHS]);
  assert.throws(() => checkAncestry("f".repeat(40), CANDIDATE_SOURCE, [...SOURCE_PATHS], [...TOOLING_PATHS]));
  assert.throws(() => checkAncestry(BASE_SOURCE, BASE_SOURCE, [...SOURCE_PATHS], [...TOOLING_PATHS]));
});
test("foreign, omitted and extra source/controller paths refuse authority", () => {
  for (const changed of [[SOURCE_PATHS[0]], [...SOURCE_PATHS, "Cargo.toml"], [SOURCE_PATHS[0], "unreviewed.rs"]])
    assert.throws(() => checkAncestry(BASE_SOURCE, CANDIDATE_SOURCE, changed, [...TOOLING_PATHS]));
  assert.throws(() => checkAncestry(BASE_SOURCE, CANDIDATE_SOURCE, [...SOURCE_PATHS], [...TOOLING_PATHS, "tests/performance/lsp-dual-server-benchmark.ts"]));
});
test("immutable parent parsing ignores message text and refuses missing/merge parents", () => {
  assert.equal(parentFromCommit(`tree ${"a".repeat(40)}\nparent ${BASE_SOURCE}\nauthor synthetic\n\nparent ${BASE_SOURCE}\n`), BASE_SOURCE);
  assert.throws(() => parentFromCommit(`tree ${"a".repeat(40)}\n\nparent ${BASE_SOURCE}\n`));
  assert.throws(() => parentFromCommit(`tree ${"a".repeat(40)}\nparent ${BASE_SOURCE}\nparent ${BASE_SOURCE}\n\nmerge`));
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

const refuse = (action: () => void): void => assert.throws(action, assert.AssertionError);
const encodeDefinition = (rows: unknown[]): string => rows.map(row => "VIZE_PRIVATE_DEFINITION_NATIVE " + JSON.stringify(row)).join("\n");
function definitions(): Record<string, unknown>[] {
  return DEFINITION_CASES.map((item, index) => {
    const start = Buffer.from(item.source).indexOf("const message") + Buffer.byteLength("const ");
    const lines = Buffer.from(item.source).subarray(0, start).toString("utf8").split("\n");
    const position = { line: lines.length - 1, character: lines.at(-1)?.length ?? 0 };
    const packet = item.private ? { uri: `file:///tmp/definition-case-${index}/App.vue`,
      range: { start: position, end: { line: position.line, character: position.character + 7 } } } : null;
    return { ...item, case: index, admitted: item.private, old: packet, actual: packet };
  });
}
function definitionMutation(index: number, values: Record<string, unknown>): string {
  return encodeDefinition(definitions().map((row, i) => i === index ? { ...row, ...values } : row));
}
test("candidate-only definition requires exactly three actual test names and zero ignored", () => {
  checkDefinitionExecution(native(DEFINITION_TESTS));
  for (const stdout of [native([]), native(DEFINITION_TESTS.slice(1)), native([...DEFINITION_TESTS, DEFINITION_TESTS[0]!]),
    native(DEFINITION_TESTS).replace("0 ignored", "1 ignored"), native(DEFINITION_TESTS).replace("3 passed", "0 passed"),
    native(DEFINITION_TESTS).replace(" ... ok", " ... ignored"), native(DEFINITION_TESTS) + native(DEFINITION_TESTS)])
    refuse(() => checkDefinitionExecution(stdout));
});
test("definition requires the exact source roster, 34 rows and six complete nonempty Scalar anchors", () => {
  const rows = definitions(); assert.equal(rows.length, 34);
  assert.equal(rows.filter(row => row.private).length, 6);
  assert.equal(definitionPairs(encodeDefinition(rows)).length, 34);
  for (const changed of [rows.slice(1), [...rows, rows[0]!], [rows[1]!, rows[0]!, ...rows.slice(2)]])
    refuse(() => { definitionPairs(encodeDefinition(changed)); });
  for (const values of [{ case: 1 }, { id: "foreign" }, { source: "changed" }, { offset: 1 }, { private: false },
    { admitted: false }, { old: null, actual: null }, { extra: true }])
    refuse(() => { definitionPairs(definitionMutation(0, values)); });
  const missing = rows.map((row, i) => { if (i !== 0) return row; const { admitted: _admitted, ...rest } = row; return rest; });
  refuse(() => { definitionPairs(encodeDefinition(missing)); });
  refuse(() => { definitionPairs(encodeDefinition(rows) + "\nVIZE_PRIVATE_DEFINITION_NATIVE_DISABLED explicit PR envelope"); });
});
test("positive declaration packet must retain exact range and current authored file URI", () => {
  const original = definitions()[0]!.old as { uri: string; range: unknown };
  for (const packet of [[original], { ...original, uri: "file:///tmp/Other.vue" }, { ...original, range: { start: {}, end: {} } },
    { ...original, extra: true }, { ...original, uri: "https://foreign/App.vue" }])
    refuse(() => { definitionPairs(definitionMutation(0, { old: packet, actual: packet })); });
});
test("negative whole Option arrays and nested unknown fields preserve order and every value", () => {
  const old = [{ targetUri: "file:///external/real.ts", extension: { flags: [true, null, { opaque: "kept" }] } }, { uri: "file:///other.ts", extra: 9 }];
  definitionPairs(definitionMutation(6, { old, actual: old }));
  for (const actual of [[...old].reverse(), old.slice(1), { ...old[0] },
    [{ ...old[0], extension: { flags: [true, null, { opaque: "changed" }] } }, old[1]]])
    refuse(() => { definitionPairs(definitionMutation(6, { old, actual })); });
  refuse(() => { definitionPairs(definitionMutation(6, { old: "scalar", actual: "scalar" })); });
});
test("mandatory native role refuses disable/require conflicts and empty overrides", () => {
  definitionEnvironment({ VIZE_TEST_REQUIRE_TSGO: "1" });
  for (const env of [{}, { VIZE_TEST_REQUIRE_TSGO: "0" }, { VIZE_TEST_REQUIRE_TSGO: "1", VIZE_TEST_DISABLE_TSGO: "1" },
    { VIZE_TEST_REQUIRE_TSGO: "1", VIZE_TEST_DISABLE_TSGO: "" }, { VIZE_TEST_REQUIRE_TSGO: "1", VIZE_TEST_TSGO_PATH: "/foreign" },
    { VIZE_TEST_REQUIRE_TSGO: "1", VIZE_TEST_TSGO_PATH: "" }]) refuse(() => definitionEnvironment(env));
});
function cargoLibrary(): Record<string, unknown>[] {
  const library = { reason: "compiler-artifact", package_id: "path+file:///repo/crates/vize_maestro#0.1.0",
    manifest_path: "/repo/crates/vize_maestro/Cargo.toml", target: { name: "vize_maestro", kind: ["lib"], src_path: "/repo/crates/vize_maestro/src/lib.rs" },
    profile: { test: true }, executable: "/owned/ci/deps/vize_maestro-hash", features: ["native"] };
  return [library, ...SDK_CRATES.map(name => ({ reason: "compiler-artifact", package_id: `registry+https://github.com/rust-lang/crates.io-index#${name}@2.0.0-beta.4`,
    target: { name, kind: ["lib"] }, features: [] })), { reason: "build-finished", success: true }];
}
test("actual library artifact and all seven SDK4 packages are mandatory, profiler remains off", () => {
  const rows = cargoLibrary(); libraryArtifact(rows, "/owned", "/repo"); assert.equal(sdkArtifacts(rows).length, 7);
  for (const values of [rows.slice(1), [...rows, rows[0]!], [{ ...rows[0], executable: "/foreign/lib" }, ...rows.slice(1)],
    [{ ...rows[0], manifest_path: "/foreign/Cargo.toml" }, ...rows.slice(1)], [{ ...rows[0], features: ["native", "profiling"] }, ...rows.slice(1)],
    [rows[0]!, ...rows.slice(2)]]) refuse(() => { libraryArtifact(values, "/owned", "/repo"); });
  for (const name of ["profiling", "micro-profiling"]) {
    const values = rows.map((row, i) => i === 1 ? { ...row, features: [name] } : row);
    refuse(() => { sdkArtifacts(values); });
  }
  refuse(() => { sdkArtifacts(rows.filter((_row, i) => i !== 1)); });
});
test("definition test source and inherited proof dependencies bind exact current Git bytes", () => {
  const fixture = execFileSync("git", ["show", `${CANDIDATE_SOURCE}:${SOURCE_PATHS[1]}`]);
  assert.equal(createHash("sha256").update(fixture).digest("hex"), DEFINITION_FIXTURE_SHA256);
  assert.equal(DEFINITION_BOUND_PATHS.length, 43);
  for (const file of DEFINITION_BOUND_PATHS.filter(file => !SOURCE_PATHS.some(changed => changed === file)))
    assert.equal(execFileSync("git", ["rev-parse", `${BASE_SOURCE}:${file}`], { encoding: "utf8" }),
      execFileSync("git", ["rev-parse", `${CANDIDATE_SOURCE}:${file}`], { encoding: "utf8" }));
});

test("ambient override guard rejects explicit empty aliases and arbitrary LSP_DUAL keys", () => {
  checkAmbientOverrides({}); checkAmbientOverrides({ UNRELATED_KEY: "" });
  const names = ["VIZE_LSP_BIN", "CORSA_PATH", "CORSA_EXECUTABLE", "VIZE_CORSA_PATH", "TSGO_PATH", "TSGO_EXECUTABLE", "VIZE_TSGO_PATH", "TSRS_PATH", "TSRS_EXECUTABLE", "VIZE_TSRS_PATH", "VIZE_TRACE_EDITOR_PREPARATION", "VIZE_TEST_TSGO_PATH", "LSP_DUAL_FRESH", "LSP_DUAL_ANY_FOREIGN_KEY"];
  for (const name of names) for (const value of ["", "false", "/foreign"])
    refuse(() => checkAmbientOverrides({ [name]: value }));
});
