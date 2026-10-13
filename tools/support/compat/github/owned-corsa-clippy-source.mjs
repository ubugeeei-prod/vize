import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { projectOwnedCorsaTestSources } from "./owned-corsa-test-source.mjs";

const sha = (body) => createHash("sha256").update(body).digest("hex");
const blob = (body) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
const url = new URL("./owned-corsa-clippy-source.json", import.meta.url);
const stat = fs.lstatSync(url);
assert(stat.isFile() && (stat.mode & 0o777) === 0o644);
const bytes = fs.readFileSync(url);
assert.equal(sha(bytes), "9b9212acd2ae6a56c4c4a8fb86abec5e550a2a55347ae030a896785e77f8c96f");
const data = JSON.parse(bytes);
assert.equal(data.schema, 1);
assert.equal(
  data.retainedGraphManifestSHA256,
  "bffe80f1e770ad10df3af2670e01b5f53a9574f357d42df6948a458f5311040c",
);
assert.equal(data.retainedOldAfterSource, "b2caebeab6fb5bc817b4e31a65d5d560449b691b");
const files = [
  "corsa_core",
  "corsa_jsonrpc",
  "corsa_client",
  "corsa_lsp",
  "corsa_orchestrator",
  "corsa",
].map((name) => "vendor/" + name + "/Cargo.toml");
assert.deepEqual(
  data.entries.map(({ file }) => file),
  files,
);
const capsules = new Map();
for (const row of data.entries) {
  const values = {};
  for (const side of ["retainedOldAfter", "currentAfter"]) {
    const cell = row[side];
    const body = Buffer.from(cell.rawBase64, "base64");
    assert.equal(body.toString("base64"), cell.rawBase64);
    assert.equal(cell.mode, "100644");
    assert.equal(body.length, cell.bytes);
    assert.equal(sha(body), cell.sha256);
    assert.equal(blob(body), cell.blob);
    values[side] = { ...cell, body };
  }
  assert.deepEqual(
    Buffer.concat([values.retainedOldAfter.body, Buffer.from(data.appendix)]),
    values.currentAfter.body,
    "whole six-manifest inverse: " + row.file,
  );
  capsules.set(row.file, values);
}
const sourceFiles = Object.freeze([
  "vendor/corsa_client/src/api/handles.rs",
  "vendor/corsa/src/bin/bench_real_corsa/args.rs",
]);
assert.deepEqual(
  data.sourceCorrections.map(({ file }) => file),
  sourceFiles,
);
for (const row of data.sourceCorrections) {
  const values = {};
  for (const side of ["retainedOldAfter", "currentAfter"]) {
    const cell = row[side];
    const body = Buffer.from(cell.rawBase64, "base64");
    assert.equal(body.toString("base64"), cell.rawBase64);
    assert.equal(cell.mode, "100644");
    assert.equal(body.length, cell.bytes);
    assert.equal(sha(body), cell.sha256);
    assert.equal(blob(body), cell.blob);
    values[side] = { ...cell, body };
  }
  assert.equal(row.clauses.length, row.file === sourceFiles[0] ? 4 : 1);
  let inverse = values.currentAfter.body.toString();
  for (const [before, after] of [...row.clauses].reverse()) {
    assert.equal(inverse.split(after).length - 1, 1);
    inverse = inverse.replace(after, before);
  }
  assert.deepEqual(
    Buffer.from(inverse),
    values.retainedOldAfter.body,
    "whole named source inverse: " + row.file,
  );
  capsules.set(row.file, values);
}
function same(actual, expected) {
  return (
    !!actual &&
    Buffer.isBuffer(actual.body) &&
    actual.mode === "100644" &&
    actual.blob === expected.blob &&
    actual.body.length === expected.bytes &&
    sha(actual.body) === expected.sha256 &&
    blob(actual.body) === expected.blob
  );
}

/** All237 current after bodies precede any old after projection; six variants are atomic. */
export function authenticateOwnedCorsaAfterContext(rows, reader, onMismatch) {
  const actualReader = reader;
  const tests = projectOwnedCorsaTestSources(rows, reader);
  if (!tests) return null;
  reader = tests.reader;
  assert.equal(rows.length, 237);
  assert.equal(new Set(rows.map(({ file }) => file)).size, 237);
  for (const file of [...files, ...sourceFiles]) {
    const old = capsules.get(file).retainedOldAfter;
    const row = rows.find((row) => row.file === file);
    assert(
      row &&
        row.after.mode === old.mode &&
        row.after.blob === old.blob &&
        row.after.bytes === old.bytes &&
        row.after.sha256 === old.sha256,
    );
  }
  for (const correction of data.sourceCorrections) {
    const row = rows.find(({ file }) => file === correction.file);
    assert.equal(row.archiveSHA256, correction.retainedArchiveSHA256);
    assert.equal(row.archiveMemberSHA256, correction.retainedArchiveMemberSHA256);
  }
  const old = files.every((file) => same(reader(file), capsules.get(file).retainedOldAfter));
  const current = files.every((file) => same(reader(file), capsules.get(file).currentAfter));
  if (!old && !current) return null;
  // Only ordered prefixes admit old, handles-only, or handles+benchmark whole bodies.
  const sourceClass = [0, 1, 2].find((count) =>
    sourceFiles.every((file, index) =>
      same(reader(file), capsules.get(file)[index < count ? "currentAfter" : "retainedOldAfter"]),
    ),
  );
  if (sourceClass === undefined || (!current && sourceClass !== 0)) return null;
  const correctedSourceFiles = sourceFiles.slice(0, sourceClass);
  const mismatched = [];
  for (const row of rows) {
    const expected =
      current && (files.includes(row.file) || correctedSourceFiles.includes(row.file))
        ? capsules.get(row.file).currentAfter
        : row.after;
    if (!same(reader(row.file), expected)) {
      if (!onMismatch) return null;
      mismatched.push(row.file);
    }
  }
  if (mismatched.length) {
    onMismatch(Object.freeze(mismatched));
    return null;
  }
  return {
    context: current
      ? {
          authority: data.authority,
          sourceContextSHA256: sha(bytes),
          wholeCurrentBodies: 237,
          wholeManifestBodies: 6,
          wholeCorrectedRustBodies: sourceClass,
          correctedSourceFiles,
          ...(tests.context ? { testSourceContext: tests.context } : {}),
          retainedOldAfterSource: data.retainedOldAfterSource,
          runtimeCredit: false,
          performanceCredit: false,
          clippyExecutionCredit: false,
        }
      : null,
    historicalAfter: (file) => {
      const value =
        current && capsules.has(file) ? capsules.get(file).retainedOldAfter : reader(file);
      return value ? { ...value, body: Buffer.from(value.body) } : null;
    },
    actualAfter: (file) => {
      const value = actualReader(file);
      return value ? { ...value, body: Buffer.from(value.body) } : null;
    },
  };
}

/** Report exact failed cells for a known complete class without lending source authority. */
export function assertOwnedCorsaAfterContext(rows, reader) {
  let mismatched = [];
  const value = authenticateOwnedCorsaAfterContext(rows, reader, (files) => {
    mismatched = files;
  });
  assert(
    value,
    "complete actual237 provider bodies and atomic manifest policy: " + mismatched.join(", "),
  );
  return value;
}
export function qualifyClippyOwnedCorsaSource(input, rows, original) {
  const retained = original(input);
  if (retained) return retained;
  const after = authenticateOwnedCorsaAfterContext(rows, (file) => input.reader("after", file));
  if (!after?.context) return null;
  const projected = original({
    ...input,
    reader: (side, file) =>
      side === "after" ? after.historicalAfter(file) : input.reader(side, file),
  });
  if (!projected) return null;
  return {
    ...projected,
    sourceContext: after.context,
    actualReader: (side, file) =>
      side === "after" ? after.actualAfter(file) : input.reader(side, file),
  };
}
