import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { RawGitRepository } from "../installed-alias-replay/raw-git.ts";
import {
  loadVueFixtureAuthority,
  originalHead,
  originalFiles,
  stockGraph,
} from "../installed-alias-replay/vue.ts";

export const eventSourceHead = "9fe63a3f20b7a56349bcca281e5f7fd98aca4adc";
const eventLockSha256 = "d2f8dbbde00a2ef67e286d4f07381ed512b312a5ad115868a0e3d235fc583179";
const importerSha256 = "29a214d7d20208d81f98f47bb987daa1fd62d4ab92abaedf15cf9560ddf0fa0b";
const lockedGraphSha256 = "273bbf4839841bb079c521014337f987f7004de89e475a5f12ebda5de80eead8";
const digest = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");

/** Preserve the literal whole Playground importer instead of reconstructing selected dependencies. */
export function playgroundImporter(lock: string): string {
  const document = lock.split("\n---\n").at(-1)!;
  const importers = document.split("\nimporters:\n")[1]?.split("\npackages:\n")[0];
  assert.equal(typeof importers, "string", "original lock importer section required");
  const matches = [...importers.matchAll(/(?:^|\n)  playground:\n/gu)];
  assert.equal(matches.length, 1, "unique literal Playground importer required");
  const start = matches[0].index! + (matches[0][0].startsWith("\n") ? 1 : 0);
  const tail = importers.slice(start),
    next = /\n  \S[^\n]*:\n/u.exec(tail);
  return next ? tail.slice(0, next.index + 1) : tail;
}

export function verifyEventFixtureSource(sourceRoot: string) {
  const repository = new RawGitRepository(sourceRoot);
  const sources = (head: string) =>
    Object.fromEntries(
      Object.keys(originalFiles).map((file) => [file, repository.file(head, file)]),
    );
  return compareEventFixtureSources(sources(originalHead), sources(eventSourceHead));
}

/** Pure comparison uses the same complete pinned bytes; it never supplies a runtime source fallback. */
export function compareEventFixtureSources(
  originalSources: Record<string, Buffer>,
  eventSources: Record<string, Buffer>,
) {
  for (const sources of [originalSources, eventSources])
    assert.deepEqual(Object.keys(sources).toSorted(), Object.keys(originalFiles).toSorted());
  const original = originalSources["pnpm-lock.yaml"],
    event = eventSources["pnpm-lock.yaml"];
  assert.equal(digest(original), originalFiles["pnpm-lock.yaml"]);
  assert.equal(digest(event), eventLockSha256);
  const importer = playgroundImporter(event.toString("utf8"));
  assert.equal(importer, playgroundImporter(original.toString("utf8")));
  assert.equal(digest(importer), importerSha256);
  const originalGraph = stockGraph(original.toString("utf8"));
  const eventGraph = stockGraph(event.toString("utf8"));
  assert.deepEqual(eventGraph, originalGraph);
  assert.equal(digest(JSON.stringify(eventGraph)), lockedGraphSha256);
  const files = Object.fromEntries(
    Object.entries(originalFiles)
      .filter(([name]) => name !== "pnpm-lock.yaml")
      .map(([name, sha]) => {
        const bytes = eventSources[name];
        assert.equal(digest(bytes), sha);
        assert.equal(digest(originalSources[name]), sha);
        assert.deepEqual(bytes, originalSources[name]);
        return [name, { sha256: sha, bytesEqual: true }];
      }),
  );
  return {
    schema: "vize-original-event-stock-source-comparison-v1",
    originalHead,
    eventSourceHead,
    lockfiles: {
      originalSha256: originalFiles["pnpm-lock.yaml"],
      eventSha256: eventLockSha256,
      wholeBytesEqual: false,
    },
    playgroundImporter: {
      wholeBytesEqual: true,
      sha256: importerSha256,
      algorithm: "SHA256 of literal UTF8 whole Playground importer block",
    },
    lockedGraph: {
      wholeRecordsEqual: true,
      packages: 26,
      sha256: lockedGraphSha256,
      algorithm:
        "SHA256(UTF8(JSON.stringify(stockGraph(lock)))) with sorted26{name,version,key,integrity,dependencies} records",
      records: eventGraph,
    },
    files,
  };
}

/** The c2bc installed receipt keeps its identity; a separate raw-source equality receipt binds 9fe. */
export function loadEventFixtureVueAuthority(options: {
  receiptPath: string;
  receiptSha256: string;
  sourceRoot: string;
}) {
  const sourceComparison = verifyEventFixtureSource(options.sourceRoot);
  const vue = loadVueFixtureAuthority({ ...options, sourceHead: originalHead });
  assert.deepEqual(vue.receipt.lockedPackages, sourceComparison.lockedGraph.records);
  const recheck = () => {
    assert.deepEqual(verifyEventFixtureSource(options.sourceRoot), sourceComparison);
    vue.recheck();
  };
  return { ...vue, sourceComparison, recheck };
}
