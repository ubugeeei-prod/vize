import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { writeTarGz } from "../../../tools/support/archive/tar.ts";
import { asArray } from "../../../tools/support/json-value.ts";
import {
  afterReference,
  assertOriginalVectors,
  expectedMapper,
  reference,
} from "../../../tools/support/release/constructor_original27_public/oracle.ts";
import {
  afterRelease,
  requiredMerge,
  sdkFiles,
} from "../../../tools/support/release/constructor_original27_public/authority.ts";
import { shippedImages } from "../../../tools/support/release/constructor_original27_public/native-prerequisite.ts";

const version = "0.650.0";
test("independent pre-native intent removes exactly two whole synthetic records and preserves both authored orders", () => {
  const current = afterReference(),
    historical = reference();
  assert.equal(current.candidate, "9f0324a09c86b5bf0aeed8b4ac10d711fbfa92b4");
  assert.deepEqual(current.cases, historical.caseIds);
  assert.deepEqual(current.authoredCases, historical.authoredCases);
  const rows = structuredClone(asArray(historical.mapper));
  const removed = asArray(current.removedExpectedSyntheticRows);
  assert.equal(removed.length, 2);
  for (const row of removed) {
    const matches = rows.flatMap((value, index) =>
      JSON.stringify(value) === JSON.stringify(row) ? [index] : [],
    );
    assert.equal(matches.length, 1);
    rows.splice(matches[0], 1);
  }
  assert.deepEqual(current.expectedMapper, rows);
  assert.deepEqual(current.expectedCli, historical.cli);
  assert.equal(rows.length, 14);
  assert.equal(asArray(current.cases).length - asArray(current.authoredCases).length, 13);
});
test("every complete authored negative remains required and either historical generated row is refused", () => {
  const current = afterReference(),
    mapper = expectedMapper(version),
    cli = asArray(current.expectedCli);
  assertOriginalVectors(version, mapper, cli);
  for (let index = 0; index < 14; index++) {
    assert.throws(() =>
      assertOriginalVectors(
        version,
        mapper.filter((_row, i) => i !== index),
        cli,
      ),
    );
    assert.throws(() =>
      assertOriginalVectors(
        version,
        mapper,
        cli.filter((_row, i) => i !== index),
      ),
    );
  }
  for (const debt of asArray(current.removedExpectedSyntheticRows))
    assert.throws(() => assertOriginalVectors(version, [...mapper, debt], cli));
  const changed = structuredClone(mapper);
  changed[0].continuation = ["  unknown continuation"];
  assert.throws(() => assertOriginalVectors(version, changed, cli));
});
test("AFTER role binds containing merge, producer source and run while the complete SDK closure has ten members", () => {
  assert.equal(requiredMerge, "1d1bb26a7f7dcbcb9adf75075bd7ad301c0bb957");
  assert.deepEqual(afterRelease, {
    version,
    run: "41507479394",
    source: "85ab7600097035afa060ca03f8a77108cdc5e337",
    candidate: "3ec8f999df04a50b37aea7f12a5fa85161d43d8d",
  });
  assert.equal(new Set(sdkFiles).size, 10);
  assert(sdkFiles.includes("virtual-ts-slots.d.ts"));
});
test("actual shipped-byte authentication rejects changed payload, installed mutation and symlink before any loader runs", () => {
  const root = mkdtempSync(join(tmpdir(), "original27-shipped-"));
  try {
    const body = Buffer.from("inert public fixture; never executed\n");
    const payload = writeTarGz([
      { name: "package/package.json", data: Buffer.from("{}\n") },
      { name: "package/index.js", data: body },
    ]);
    const integrity = "sha512-" + createHash("sha512").update(payload).digest("base64");
    writeFileSync(join(root, "package.json"), "{}\n");
    writeFileSync(join(root, "index.js"), body);
    assert.equal(shippedImages(root, payload, integrity).images.length, 2);
    const changed = Buffer.from(payload);
    changed[changed.length - 1] ^= 1;
    assert.throws(() => shippedImages(root, changed, integrity));
    writeFileSync(join(root, "index.js"), "mutated");
    assert.throws(() => shippedImages(root, payload, integrity));
    rmSync(join(root, "index.js"));
    symlinkSync(join(root, "package.json"), join(root, "index.js"));
    assert.throws(() => shippedImages(root, payload, integrity));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
