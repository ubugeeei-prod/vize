import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { readPinnedArtifact, sha256 } from "../differential/harness.mjs";
import { root } from "./support/lsp/paths.ts";
import {
  SESSION_CANDIDATES,
  parseSessionCandidates,
  linkSessionSourceOrigins,
  type SourceWitness,
} from "./support/lsp/session-source-links.ts";

const catalog = readPinnedArtifact(root, SESSION_CANDIDATES);
const candidates = parseSessionCandidates(catalog);
const witness = (
  candidate: (typeof candidates)[number],
  line = candidate.tests[0].startLine,
): SourceWitness => ({
  path: candidate.current.path,
  sha256: candidate.current.sha256,
  line,
  column: 1,
});

test("source-origin catalog retains eight response candidates and two safety controls, never wire expectations", () => {
  assert.equal(
    candidates.filter((candidate) => candidate.category === "required-response").length,
    8,
  );
  assert.equal(candidates.filter((candidate) => candidate.category === "control").length, 2);
  assert.equal(
    candidates.filter((candidate) => candidate.original.sha256 === candidate.current.sha256).length,
    8,
  );
  for (const candidate of candidates) {
    assert.equal(
      sha256(fs.readFileSync(path.join(root, candidate.current.path))),
      candidate.current.sha256,
    );
  }
  const tampered = catalog.toString().replace(/\t0\t0\n/, "\t1\t0\n");
  assert.throws(() => parseSessionCandidates(Buffer.from(tampered)));
});

test("actual caller hash and selected test line are both required for historical source association", () => {
  const candidate = candidates.find((row) => row.fixCommit.startsWith("c1545ef"))!;
  const selected = witness(candidate);
  const linked = linkSessionSourceOrigins(root, [selected, selected]);
  assert.equal(linked.candidates.length, 1);
  assert.deepEqual(linked.candidates[0].selectedTests, [candidate.tests[0].name]);
  for (const field of ["wholeFixesClosed", "nativeHandled", "nativeEquivalent"] as const) {
    assert.equal(linked.candidates[0][field], 0);
  }
  assert.equal(linked.candidates[0].completeExpectedResponsesFrozen, false);
  assert.equal(linked.candidates[0].dependencyClosureProved, false);
  assert.equal(
    linkSessionSourceOrigins(root, [{ ...selected, sha256: "0".repeat(64) }]).candidates.length,
    0,
  );
  assert.equal(linkSessionSourceOrigins(root, [{ ...selected, line: 1 }]).candidates.length, 0);
  // This actual same-file line belongs to the unrelated reactive inlay-hint test.
  assert.equal(linkSessionSourceOrigins(root, [{ ...selected, line: 50 }]).candidates.length, 0);
  assert.equal(
    linkSessionSourceOrigins(root, [{ ...selected, path: "tests/tooling/other.test.ts" }])
      .candidates.length,
    0,
  );
});

test("one initial-diagnostic execution can associate two source histories without duplicate launch credit", () => {
  const candidate = candidates.find((row) => row.fixCommit.startsWith("215ca5706"))!;
  const linked = linkSessionSourceOrigins(root, [witness(candidate)]);
  assert.deepEqual(
    linked.candidates.map((row) => row.fixCommit),
    ["215ca5706f402ed831a3bac0166e96cbc21da286", "c5eea8eed6cf316aaa6ff6daa9e2dabd47a13165"],
  );
  assert.ok(linked.candidates.every((row) => row.wholeFixesClosed === 0));
});

test("missing source catalog stays unassociated and corrupt pinned catalogs fail closed", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-session-source-origin-law-"));
  try {
    const missing = linkSessionSourceOrigins(temporary, [witness(candidates[0])]);
    assert.equal(missing.catalog.present, false);
    assert.deepEqual(missing.candidates, []);
    const destination = path.join(temporary, SESSION_CANDIDATES.path);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, Buffer.concat([catalog, Buffer.from("corrupt\n")]));
    assert.throws(() => linkSessionSourceOrigins(temporary, []), /SHA256 mismatch/);
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
