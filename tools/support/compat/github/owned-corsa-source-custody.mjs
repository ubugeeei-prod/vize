import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { qualifyClippyOwnedCorsaSource } from "./owned-corsa-clippy-source.mjs";

const sha = (body) => createHash("sha256").update(body).digest("hex");
const blob = (body) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
const authorityBytes = (name) => {
  const file = new URL(name, import.meta.url);
  const stat = fs.lstatSync(file);
  assert(stat.isFile() && (stat.mode & 0o777) === 0o644, "regular authority body: " + name);
  return fs.readFileSync(file);
};
const raw = authorityBytes("./owned-corsa-source-manifest.json");
assert.equal(sha(raw), "bffe80f1e770ad10df3af2670e01b5f53a9574f357d42df6948a458f5311040c");
const metadata = JSON.parse(raw);
assert.deepEqual(
  metadata.parts.map(({ file }) => file),
  Array.from(
    { length: 20 },
    (_, index) => `owned-corsa-source-part-${String(index).padStart(2, "0")}.json`,
  ),
);
const rows = metadata.parts.flatMap(({ file, sha256, bodyPairs }) => {
  const body = authorityBytes("./" + file);
  assert.equal(sha(body), sha256, file);
  const values = JSON.parse(body);
  assert.equal(values.length, bodyPairs);
  return values;
});
const freeze = (value) => {
  if (value && typeof value === "object") {
    for (const child of Object.values(value)) freeze(child);
    Object.freeze(value);
  }
  return value;
};
export const ownedCorsaSource = freeze({ ...metadata, rows });
assert.equal(ownedCorsaSource.version, 1);
assert.equal(ownedCorsaSource.oldProductionBodies, 121);
assert.equal(ownedCorsaSource.archiveMemberBodies, 112);
assert.equal(ownedCorsaSource.additiveLicenseBodies, 4);
assert.equal(ownedCorsaSource.completeProductionBodies, 237);
assert.equal(ownedCorsaSource.rows.length, 237);
assert.equal(new Set(ownedCorsaSource.rows.map(({ file }) => file)).size, 237);
assert.equal(ownedCorsaSource.rows.filter(({ archiveMember }) => archiveMember).length, 112);
assert.equal(
  ownedCorsaSource.rows.filter(({ additiveLicenseCopy }) => additiveLicenseCopy).length,
  4,
);

const cargoPaths = [
  "Cargo.lock",
  "Cargo.toml",
  "vendor/corsa_core/Cargo.toml",
  "vendor/corsa_lsp/Cargo.toml",
];
const capsules = new Map();
for (const { file, side, historical } of ownedCorsaSource.historicalCargoBodies) {
  assert(cargoPaths.includes(file));
  assert(["before", "after"].includes(side));
  const key = `${side}:${file}`;
  assert(!capsules.has(key));
  if (historical === null) capsules.set(key, null);
  else {
    const body = Buffer.from(historical.rawBase64, "base64");
    assert.equal(body.toString("base64"), historical.rawBase64);
    assert.equal(historical.mode, "100644");
    assert.equal(body.length, historical.bytes);
    assert.equal(sha(body), historical.sha256);
    assert.equal(blob(body), historical.blob);
    capsules.set(key, { ...historical, body });
  }
}
assert.equal(capsules.size, 8);

/** Reader must bind raw, replacement-disabled Git and physical regular-file identity. */
function sameBody(actual, expected) {
  if (expected === null) return actual === null;
  if (!actual || !Buffer.isBuffer(actual.body)) return false;
  return (
    actual.mode === "100644" &&
    actual.mode === expected.mode &&
    actual.body.length === expected.bytes &&
    sha(actual.body) === expected.sha256 &&
    blob(actual.body) === expected.blob &&
    actual.blob === expected.blob
  );
}

/** Historical projection is available only after the complete actual 237-body pair succeeds. */
function qualifyOriginalOwnedCorsaSource({
  baseline,
  production,
  alreadyQualified,
  reader,
  historicalPairs,
}) {
  if (baseline !== ownedCorsaSource.baseline) return null;
  if (new Set(production).size !== production.length) return null;
  const expected = ownedCorsaSource.rows.map(({ file }) => file);
  const paths = new Set(expected);
  if (production.every((file) => alreadyQualified.has(file))) return null;
  if (production.some((file) => !paths.has(file) && !alreadyQualified.has(file))) return null;
  if (
    JSON.stringify(production.filter((file) => paths.has(file)).toSorted()) !==
    JSON.stringify(expected.toSorted())
  )
    return null;
  if (historicalPairs.length !== 121 || new Set(historicalPairs.map(([file]) => file)).size !== 121)
    return null;
  const historical = new Map(
    historicalPairs.map(([file, before, after]) => [file, { before, after }]),
  );
  for (const row of ownedCorsaSource.rows) {
    if (
      !sameBody(reader("before", row.file), row.before) ||
      !sameBody(reader("after", row.file), row.after)
    )
      return null;
    if (Object.hasOwn(row, "historicalBeforeSHA256")) {
      const original = historical.get(row.file);
      if (
        !original ||
        original.before !== row.historicalBeforeSHA256 ||
        original.after !== row.historicalAfterSHA256
      )
        return null;
      if (cargoPaths.includes(row.file)) {
        for (const side of ["before", "after"])
          if ((capsules.get(`${side}:${row.file}`)?.sha256 ?? null) !== original[side]) return null;
      } else if (
        (row.before?.sha256 ?? null) !== original.before ||
        row.after.sha256 !== original.after
      )
        return null;
    }
  }
  const additions = new Set(
    ownedCorsaSource.rows.filter((row) => !historical.has(row.file)).map(({ file }) => file),
  );
  assert.equal(additions.size, 116);
  return {
    authority: "complete-reviewed-owned-corsa-package-source-pair",
    baseline,
    manifestSHA256: sha(raw),
    completeProductionBodies: 237,
    originalProductionBodies: 121,
    newArchiveMemberBodies: 112,
    additiveLicenseBodies: 4,
    runtimeCredit: false,
    performanceCredit: false,
    files: expected,
    historicalProduction: production.filter((file) => !additions.has(file)),
    historicalReader: (side, file) => {
      const value = cargoPaths.includes(file)
        ? capsules.get(`${side}:${file}`)
        : reader(side, file);
      return value === null ? null : { ...value, body: Buffer.from(value.body) };
    },
  };
}

export function qualifyOwnedCorsaSource(input) {
  return qualifyClippyOwnedCorsaSource(
    input,
    ownedCorsaSource.rows,
    qualifyOriginalOwnedCorsaSource,
  );
}
