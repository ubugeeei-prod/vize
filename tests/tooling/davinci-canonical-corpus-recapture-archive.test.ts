import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { validateObserverArtifact } from "../../tools/support/compat/github/canonical-corpus-observer.mjs";
import { cycleOwner } from "../../tools/support/compat/github/canonical-corpus-logical-walk.ts";
import { test } from "node:test";
import { validateRecordRecapture } from "../../tools/support/compat/github/canonical-corpus-recapture-archive.mjs";

await test("record relation cannot replace strict source/run/head/phase custody", () => {
  // These controls must refuse before any archive I/O. A relation is an
  // explicit later-phase claim, never permission on an ordinary corpus proof.
  const identity = {
    schema: "vize.canonical-corpus",
    version: 1,
    repository: "ubugeeei-prod/vize",
    sha: "a".repeat(40),
    tree: "b".repeat(40),
    runId: 12,
    attempt: 1,
    event: "merge_group",
    providerSha: "a".repeat(40),
    repositoryId: 1,
    providerRepositoryId: 1,
    gitlinksSha256: "c".repeat(64),
    modulesSha256: "d".repeat(64),
    files: 1,
    filesSha256: "e".repeat(64),
    committedSha256: "f".repeat(64),
    nativeWalk: { vectorSha256: "0".repeat(64) },
    gitlinks: [],
  };
  const relation = {
    schema: "vize.canonical-record-recapture",
    version: 1,
    phase: "snapshot-to-record",
    nativeAcceptanceCredit: false,
    baselineIdentity: identity,
  };
  const validate = (value: unknown) =>
    validateRecordRecapture("/no-archive-read-authorized", identity, [], {}, value);
  assert.throws(() => validate(undefined), /Missing snapshot-to-record/);
  for (const [key, value] of [
    ["schema", "foreign"],
    ["version", 2],
    ["phase", "snapshot"],
    ["nativeAcceptanceCredit", true],
  ])
    assert.throws(
      () => validate({ ...relation, [key]: value }),
      /recapture|native acceptance/u,
      String(key),
    );
  for (const key of [
    "schema",
    "version",
    "repository",
    "sha",
    "tree",
    "runId",
    "attempt",
    "event",
    "providerSha",
    "repositoryId",
    "providerRepositoryId",
    "gitlinksSha256",
    "modulesSha256",
    "files",
    "filesSha256",
    "committedSha256",
    "nativeWalk",
    "gitlinks",
  ]) {
    assert.throws(
      () => validate({ ...relation, baselineIdentity: { ...identity, [key]: "foreign" } }),
      /Canonical corpus|gitlink ownership/u,
      key,
    );
  }
});

await test("final observer validation derives mandatory relation from owned gitlinks even if markers are removed", () => {
  const directory = mkdtempSync(join(tmpdir(), "canonical-recapture-downgrade-"));
  const write = (name: string, value: unknown) =>
    writeFileSync(join(directory, name), JSON.stringify(value));
  try {
    for (const variant of ["both removed", "identity removed", "relation removed"]) {
      const identity = {
        gitlinks: [{ path: cycleOwner.repository, sha: cycleOwner.revision }],
        ...(variant === "relation removed" ? { nativeWalk: {} } : {}),
      };
      write("identity.json", identity);
      write("observer.json", {
        schema: "vize.canonical-corpus-observer",
        version: 1,
        observer: "reach",
        outcome: "success",
        identity,
        ...(variant === "identity removed" ? { recapture: {} } : {}),
      });
      // Both receipt and archive identity carry the same downgrade, which the
      // ordinary equality comparator alone cannot detect. This must fail in
      // the actual final validator before hydration/archive reads can begin.
      assert.throws(
        () => validateObserverArtifact("reach", directory),
        /Missing native snapshot identity|Missing native snapshot-to-record relation/u,
        variant,
      );
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
