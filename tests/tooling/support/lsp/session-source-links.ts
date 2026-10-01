import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { readPinnedArtifact } from "../../../differential/harness.mjs";

export const SESSION_SOURCE_PIN = "cc87bb5960ea9e49e82672205df919de58bb4b24";
export const SESSION_CANDIDATES = {
  path: "docs/davinci/plan/lsp-retained-session-candidates.tsv",
  sha256: "0bc776ff5fc1e1d7bde32bfae865b1b87e8c84e99a6bcf5c8e2cfb4652daf4ba",
};
export type SourceWitness = { path: string; line: number; column: number; sha256: string };
type TestRange = { name: string; startLine: number; endLine: number };
type Candidate = {
  fixCommit: string;
  category: "required-response" | "control";
  original: { path: string; sha256: string };
  current: { path: string; sha256: string };
  tests: TestRange[];
};
const COLUMNS = [
  "fix_commit",
  "category",
  "original_path",
  "original_sha256",
  "current_path",
  "current_sha256",
  "current_test_ranges",
  "authored_input_hashes",
  "local_setup",
  "initialization_options",
  "requests",
  "expected_distinctions",
  "superseded",
  "unresolved",
  "whole_fix_credit",
  "native_credit",
];

export function parseSessionCandidates(bytes: Buffer): Candidate[] {
  const [header, ...rows] = bytes.toString("utf8").trimEnd().split("\n");
  assert.equal(header, COLUMNS.join("\t"));
  assert.equal(rows.length, 10, "the bounded source-origin catalog has ten candidates");
  const seen = new Set<string>();
  return rows.map((row) => {
    const fields = row.split("\t");
    assert.equal(fields.length, COLUMNS.length);
    const value = Object.fromEntries(COLUMNS.map((column, index) => [column, fields[index]]));
    assert.match(value.fix_commit, /^[a-f0-9]{40}$/);
    assert.ok(!seen.has(value.fix_commit), "duplicate historical source origin");
    seen.add(value.fix_commit);
    assert.ok(value.category === "required-response" || value.category === "control");
    for (const field of ["original_path", "current_path"]) {
      assert.match(value[field], /^tests\/tooling\/[^/]+\.test\.ts$/);
    }
    for (const field of ["original_sha256", "current_sha256"]) {
      assert.match(value[field], /^[a-f0-9]{64}$/);
    }
    assert.equal(value.whole_fix_credit, "0");
    assert.equal(value.native_credit, "0");
    assert.equal(JSON.parse(value.expected_distinctions).wholeWire, false);
    for (const field of [
      "authored_input_hashes",
      "initialization_options",
      "requests",
      "superseded",
      "unresolved",
    ]) {
      JSON.parse(value[field]);
    }
    const tests = JSON.parse(value.current_test_ranges) as TestRange[];
    assert.ok(Array.isArray(tests) && tests.length > 0);
    for (const test of tests) {
      assert.equal(typeof test.name, "string");
      assert.ok(test.name.length > 0);
      assert.ok(Number.isSafeInteger(test.startLine) && test.startLine > 0);
      assert.ok(Number.isSafeInteger(test.endLine) && test.endLine >= test.startLine);
    }
    return {
      fixCommit: value.fix_commit,
      category: value.category,
      original: { path: value.original_path, sha256: value.original_sha256 },
      current: { path: value.current_path, sha256: value.current_sha256 },
      tests,
    };
  });
}

/** Associate only selected authored call sites; source association never admits a result. */
export function linkSessionSourceOrigins(repoRoot: string, witnesses: SourceWitness[]) {
  const present = fs.existsSync(path.join(repoRoot, SESSION_CANDIDATES.path));
  const candidates = present
    ? parseSessionCandidates(readPinnedArtifact(repoRoot, SESSION_CANDIDATES))
    : [];
  return {
    state: "pending-original-input-and-session-reconciliation",
    catalog: { ...SESSION_CANDIDATES, sourceRevision: SESSION_SOURCE_PIN, present },
    candidates: candidates.flatMap((candidate) => {
      const selectedTests = candidate.tests
        .filter((test) =>
          witnesses.some(
            (witness) =>
              witness.path === candidate.current.path &&
              witness.sha256 === candidate.current.sha256 &&
              witness.line >= test.startLine &&
              witness.line <= test.endLine,
          ),
        )
        .map((test) => test.name);
      if (selectedTests.length === 0) return [];
      return [
        {
          fixCommit: candidate.fixCommit,
          category: candidate.category,
          purpose:
            candidate.category === "control" ? "safety control opportunity" : "response candidate",
          evidenceClaim: "selected test source existed at the recorded fix",
          original: candidate.original,
          current: candidate.current,
          selectedTests,
          authoredTestFileByteIdentical: candidate.original.sha256 === candidate.current.sha256,
          completeExpectedResponsesFrozen: false,
          dependencyClosureProved: false,
          wholeFixesClosed: 0,
          nativeHandled: 0,
          nativeEquivalent: 0,
        },
      ];
    }),
  };
}
