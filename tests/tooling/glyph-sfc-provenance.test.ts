import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { cargoEvidenceCommand } from "../../legacy-tools/fixtures/glyph-corpus.mjs";
import {
  evidenceSourceCommit,
  formatterEvidence,
} from "../../legacy-tools/fixtures/glyph-sfc-evidence.mjs";

test("formatter and commit provenance are bound to exact local bytes", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-provenance-"));
  const binary = path.join(tempDir, "vize");
  try {
    fs.writeFileSync(binary, Buffer.from([0, 1, 2, 255]));
    assert.deepEqual(formatterEvidence(binary, " vize 0.346.0 \n"), {
      version: "vize 0.346.0",
      binarySha256: createHash("sha256").update(fs.readFileSync(binary)).digest("hex"),
    });
    assert.throws(
      () => formatterEvidence(path.join(tempDir, "missing"), "vize"),
      /binary is missing/,
    );
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }

  assert.equal(evidenceSourceCommit({ GITHUB_SHA: "a".repeat(40) }), "a".repeat(40));
  assert.throws(() => evidenceSourceCommit({ GITHUB_SHA: "main" }), /must be an exact commit/);
  assert.equal(
    evidenceSourceCommit({}, () => ({ status: 0, stdout: `${"b".repeat(40)}\n`, stderr: "" })),
    "b".repeat(40),
  );
  assert.throws(
    () => evidenceSourceCommit({}, () => ({ status: 128, stdout: "", stderr: "no repository\n" })),
    /git rev-parse HEAD failed: no repository/,
  );
  assert.equal(
    cargoEvidenceCommand(JSON.stringify({ target_directory: "/tmp/custom-target" }), "linux"),
    "/tmp/custom-target/debug/vize",
  );
  assert.equal(
    cargoEvidenceCommand(JSON.stringify({ target_directory: "C:\\target" }), "win32"),
    path.join("C:\\target", "debug", "vize.exe"),
  );
  assert.throws(
    () => cargoEvidenceCommand(JSON.stringify({}), "linux"),
    /omitted target_directory/,
  );
});
