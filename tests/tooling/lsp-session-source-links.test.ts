import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { readPinnedArtifact, sha256 } from "../differential/harness.mjs";
import { root } from "./support/lsp/paths.ts";
import { originalVueConfigFixtureSource } from "./support/vue-config-source-custody.ts";
import {
  CURRENT_SESSION_SOURCE_PIN,
  ORIGINAL_SESSION_CANDIDATES,
  SESSION_CANDIDATES,
  SESSION_SOURCE_PIN,
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
    7,
  );
  for (const candidate of candidates) {
    assert.equal(
      sha256(
        originalVueConfigFixtureSource(
          candidate.current.path,
          fs.readFileSync(path.join(root, candidate.current.path)),
        ),
      ),
      candidate.current.sha256,
    );
  }
  const tampered = catalog.toString().replace(/\t0\t0\n/, "\t1\t0\n");
  assert.throws(() => parseSessionCandidates(Buffer.from(tampered)));
});

test("current catalog succeeds the frozen original without changing inputs, responses or history credit", () => {
  const original = readPinnedArtifact(root, ORIGINAL_SESSION_CANDIDATES);
  const originalRows = original.toString().trimEnd().split("\n");
  const currentRows = catalog.toString().trimEnd().split("\n");
  const originalCandidates = parseSessionCandidates(original);
  assert.equal(
    originalCandidates.filter((row) => row.original.sha256 === row.current.sha256).length,
    8,
  );
  assert.equal(currentRows.length, originalRows.length);
  for (let index = 0; index < originalRows.length; index++) {
    const before = originalRows[index].split("\t");
    const after = currentRows[index].split("\t");
    if (before[0] === "044e518c22a7d72fb6b3ccc3d0c8361f816019a9") {
      assert.equal(before[3], "b46e2badf5fc3b0bb53296d9692e36be027b34eb1869d9c095ebd4cf14fa2140");
      assert.equal(before[5], before[3]);
      assert.notEqual(after[5], before[5]);
      assert.notEqual(after[6], before[6]);
      // Only the complete current-file hash and selected current ranges advance.
      // Every original/input/response/setup/supersession/credit field stays frozen.
      after[5] = before[5];
      after[6] = before[6];
    }
    assert.deepEqual(after, before);
  }
  const linked = linkSessionSourceOrigins(root, []);
  assert.equal(linked.catalog.sourceRevision, CURRENT_SESSION_SOURCE_PIN);
  assert.deepEqual(linked.catalog.originalCatalog, {
    ...ORIGINAL_SESSION_CANDIDATES,
    sourceRevision: SESSION_SOURCE_PIN,
  });
});

test("color successor binds unchanged original callbacks to current spans and excludes the dense performance case", () => {
  const candidate = candidates.find((row) => row.fixCommit.startsWith("044e518c"))!;
  const source = fs.readFileSync(path.join(root, candidate.current.path), "utf8");
  assert.equal(
    candidate.original.sha256,
    "b46e2badf5fc3b0bb53296d9692e36be027b34eb1869d9c095ebd4cf14fa2140",
  );
  assert.equal(
    candidate.current.sha256,
    "c9e909d192afc99e9d1500328efed25ec09a4f29e0199b8a20cf637c3a290166",
  );
  assert.equal(sha256(source), candidate.current.sha256);
  assert.equal(originalVueConfigFixtureSource(candidate.current.path, Buffer.from(source)), source);
  assert.deepEqual(candidate.tests, [
    {
      name: "documentColor reports every supported CSS colour and nothing else",
      startLine: 109,
      endLine: 140,
    },
    { name: "colorPresentation offers hex, rgb, and hsl notation", startLine: 142, endLine: 158 },
  ]);
  const callbackHashes = [
    "8c3f0059217daa278bfeada049181b521a4065cbe739822db31910e9d7117017",
    "2d73e00582c17fdeba065b3994c6935dac1cc29425f2662f179c2ee1330bf076",
  ];
  const lines = source.split("\n");
  for (const [index, selected] of candidate.tests.entries()) {
    // These complete callback hashes also match the original 044e518c source.
    const callback = lines.slice(selected.startLine - 1, selected.endLine).join("\n") + "\n";
    assert.equal(sha256(callback), callbackHashes[index]);
    for (const line of [selected.startLine, selected.endLine]) {
      const linked = linkSessionSourceOrigins(root, [witness(candidate, line)]);
      assert.equal(linked.candidates.length, 1);
      assert.deepEqual(linked.candidates[0].selectedTests, [selected.name]);
      assert.equal(linked.candidates[0].authoredTestFileByteIdentical, false);
      assert.equal(linked.candidates[0].category, "control");
      assert.equal(linked.candidates[0].completeExpectedResponsesFrozen, false);
      assert.equal(linked.candidates[0].dependencyClosureProved, false);
      for (const field of ["wholeFixesClosed", "nativeHandled", "nativeEquivalent"] as const) {
        assert.equal(linked.candidates[0][field], 0);
      }
    }
  }
  // Exclude the shared helper, surrounding blank lines and the entire new dense case.
  for (const line of [
    75,
    108,
    141,
    159,
    ...Array.from({ length: 30 }, (_, index) => 160 + index),
  ]) {
    assert.equal(linkSessionSourceOrigins(root, [witness(candidate, line)]).candidates.length, 0);
  }
  for (const changed of [
    source + "\n",
    source.replace("source = SOURCE", 'source = ""'),
    source.replace("row.repeat(2048)", "row.repeat(2049)"),
    source.replace("red: 1, green: 0, blue: 0, alpha: 1", "red: 0, green: 0, blue: 0, alpha: 1"),
  ]) {
    assert.notEqual(changed, source);
    const changedHash = sha256(
      originalVueConfigFixtureSource(candidate.current.path, Buffer.from(changed)),
    );
    assert.notEqual(changedHash, candidate.current.sha256);
    assert.equal(
      linkSessionSourceOrigins(root, [{ ...witness(candidate), sha256: changedHash }]).candidates
        .length,
      0,
    );
  }
  assert.equal(
    linkSessionSourceOrigins(root, [{ ...witness(candidate), sha256: candidate.original.sha256 }])
      .candidates.length,
    0,
  );
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

test("reviewed configuration carriers preserve whole source pins and reject unrelated source edits", () => {
  const owners = [
    "tests/tooling/lsp-workspace-folder-changes.test.ts",
    "tests/tooling/lsp-initial-diagnostics-version.test.ts",
  ];
  for (const file of owners) {
    const candidate = candidates.find((row) => row.current.path === file)!;
    const bytes = fs.readFileSync(path.join(root, file));
    const original = originalVueConfigFixtureSource(file, bytes);
    assert.equal(sha256(original), candidate.current.sha256);
    assert.notEqual(sha256(bytes), candidate.current.sha256);
    assert.equal(
      sha256(originalVueConfigFixtureSource(file, Buffer.from(original))),
      candidate.current.sha256,
    );
    for (const source of [
      bytes.toString() + "\n",
      bytes.toString().replace("new LspSession()", "new LspSession(undefined)"),
    ]) {
      assert.notEqual(source, bytes.toString());
      assert.notEqual(
        sha256(originalVueConfigFixtureSource(file, Buffer.from(source))),
        candidate.current.sha256,
      );
    }
  }
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
