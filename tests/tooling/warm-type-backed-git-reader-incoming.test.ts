import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import {
  prepareLiteralSource,
  runHistoricalIncomingLaw,
} from "../performance/support/warm-type-backed-git-reader-nodeep.ts";
import {
  gitReaderIncomingChanges,
  gitReaderIncomingPairs,
  gitReaderIncomingSource,
  qualifyGitReaderIncomingHarness,
} from "../performance/support/warm-type-backed-git-reader-incoming.ts";

const pairs = new Map(
  gitReaderIncomingPairs.map(([file, before, after]) => [file, { before, after }]),
);
const digest = (side: "before" | "after", file: string) => pairs.get(file)![side];

test("the historical incoming tuple executes its complete raw source law and imports", () => {
  const root = path.resolve(import.meta.dirname, "../..");
  prepareLiteralSource(root, gitReaderIncomingSource);
  assert.deepEqual(runHistoricalIncomingLaw(root), {
    source: "3dda8f9d71fbd7e701f0805187daff6191865190",
    moduleSHA256: "d7b7789b3a993b2434aeede15203fa15a177c4190833e65839a09963928007bd",
    completeBodyPairs: 79,
    additionalImportedBodies: 3,
    passed: 4,
    acceptanceCredit: "none",
  });
});

test("every complete incoming before and after body remains mandatory", () => {
  assert.ok(qualifyGitReaderIncomingHarness(gitReaderIncomingChanges, new Set(), digest));
  for (const [file] of gitReaderIncomingPairs)
    for (const side of ["before", "after"] as const)
      assert.equal(
        qualifyGitReaderIncomingHarness(gitReaderIncomingChanges, new Set(), (candidate, name) =>
          candidate === side && name === file ? "f".repeat(64) : digest(candidate, name),
        ),
        null,
        `${side}:${file}`,
      );
});

test("the incoming family cannot admit unknown, duplicate or omitted owners", () => {
  assert.equal(
    qualifyGitReaderIncomingHarness([...gitReaderIncomingChanges, "unknown.ts"], new Set(), digest),
    null,
  );
  assert.equal(
    qualifyGitReaderIncomingHarness(
      [...gitReaderIncomingChanges, gitReaderIncomingChanges[0]],
      new Set(),
      digest,
    ),
    null,
  );
  for (const omitted of gitReaderIncomingChanges)
    assert.equal(
      qualifyGitReaderIncomingHarness(
        gitReaderIncomingChanges.filter((file) => file !== omitted),
        new Set(),
        digest,
      ),
      null,
      omitted,
    );
  assert.equal(qualifyGitReaderIncomingHarness([], new Set(), digest), null);
  const inherited = new Set(["already-qualified.ts"]);
  assert.ok(
    qualifyGitReaderIncomingHarness([...gitReaderIncomingChanges, ...inherited], inherited, digest),
  );
});

test("a different old whole preset law cannot be relabelled as the incoming body", () => {
  const file = "tests/tooling/warm-type-backed-default-migration-host.test.ts";
  const original = "86ce5a8bcbb25c298e59d211910b6aeca3da3e280f9a2109ce0daee6ace2ca0c";
  for (const side of ["before", "after"] as const)
    assert.equal(
      qualifyGitReaderIncomingHarness(gitReaderIncomingChanges, new Set(), (candidate, name) =>
        candidate === side && name === file ? original : digest(candidate, name),
      ),
      null,
    );
});
