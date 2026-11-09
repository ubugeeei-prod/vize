// The finalizer re-proves process-sharded observer artifacts: the same
// receipts, logs and manifests as single-process observers, plus the pinned
// shard vectors that must exactly cover files.json.
import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { validateArtifactsConcurrently } from "../../tools/support/compat/github/canonical-corpus-artifact-batch.ts";
import {
  forcedCheckoutArgs,
  validateHydration,
} from "../../tools/support/compat/github/canonical-corpus-hydration.mjs";
import { observers, sha256 } from "../../tools/support/compat/github/canonical-corpus-identity.mjs";
import {
  observerLogs,
  validateObserverLog,
} from "../../tools/support/compat/github/canonical-corpus-observer.mjs";
import { verifyCanonicalArtifacts } from "../../tools/support/compat/github/canonical-corpus-workers.mjs";
import {
  canonicalWorkerInventory,
  canonicalWorkerLog,
} from "./support/canonical-corpus-worker-inventory.mjs";
import { shardedObserverEvidence } from "./support/canonical-corpus-shard-logs.ts";

type Gitlink = { path: string; sha: string };
type Inventory = { gitlinks: Gitlink[]; files: [string, string, number, string][]; proof: unknown };

const sha = "a".repeat(40);
const context = {
  repositoryId: 99,
  providerRepositoryId: 99,
  providerSha: sha,
  repository: "ubugeeei-prod/vize",
  runId: 12,
  attempt: 1,
  sha,
  event: "merge_group",
};
const selection = {
  schema: "vize.canonical-worker-selection",
  version: 1,
  ...context,
  workers: (observers as string[]).map((observer) => ({
    observer,
    artifactName: `canonical-corpus-${observer}-12-1-${sha}`,
    executionAttempt: 1,
  })),
};
const write = (path: string, value: unknown) => writeFileSync(path, `${JSON.stringify(value)}\n`);

function writeArtifacts(root: string, inventory: Inventory, identity: Record<string, unknown>) {
  const { gitlinks, files, proof } = inventory;
  const selectedText = `${gitlinks.map((row) => row.path).join("\n")}\n`;
  const statusText = `${gitlinks.map((row) => ` ${row.sha} ${row.path}`).join("\n")}\n`;
  for (const worker of selection.workers) {
    const path = join(root, worker.artifactName);
    mkdirSync(path);
    const sharded =
      worker.observer === "dom"
        ? null
        : shardedObserverEvidence(
            worker.observer,
            files.map(([name]) => name),
            gitlinks.length,
          );
    const bytes: Buffer =
      sharded?.log ?? canonicalWorkerLog(worker.observer, files.length, gitlinks.length);
    write(join(path, "identity.json"), identity);
    write(join(path, "files.json"), files);
    write(join(path, "committed-corpus.json"), proof);
    writeFileSync(join(path, "selected-gitlinks.txt"), selectedText);
    writeFileSync(join(path, "submodule-status.txt"), statusText);
    write(join(path, "hydration.json"), {
      ...identity,
      schema: "vize.canonical-corpus-hydration",
      args: forcedCheckoutArgs(gitlinks),
      status: 0,
      statusStatus: 0,
      stdoutSha256: sha256(""),
      stderrSha256: sha256(""),
      statusStderrSha256: sha256(""),
    });
    for (const name of [
      "hydration-stdout.log",
      "hydration-stderr.log",
      "hydration-status-stderr.log",
    ])
      writeFileSync(join(path, name), "");
    writeFileSync(
      join(path, (observerLogs as Record<string, string>)[worker.observer] as string),
      bytes,
    );
    for (const [name, vector] of sharded?.vectors ?? []) writeFileSync(join(path, name), vector);
    const vectors = sharded
      ? Object.fromEntries([...sharded.vectors].map(([name, vector]) => [name, sha256(vector)]))
      : undefined;
    write(join(path, "observer.json"), {
      schema: "vize.canonical-corpus-observer",
      version: 1,
      identity,
      hydrationSha256: validateHydration(path, identity),
      observer: worker.observer,
      outcome: "success",
      logSha256: sha256(bytes),
      selectedSha256: sha256(selectedText),
      statusSha256: sha256(statusText),
      counters: validateObserverLog(worker.observer, bytes, identity, {
        files,
        readVector: (name: string) => readFileSync(join(path, name)),
      }),
      ...(vectors ? { vectorSha256: vectors } : {}),
    });
  }
}

await test("finalization re-proves process-sharded observers and their exact shard vectors", () => {
  const root = mkdtempSync(join(tmpdir(), "canonical-sharded-artifacts-"));
  const inventory = canonicalWorkerInventory(3001, 149) as Inventory;
  const { gitlinks, files, proof } = inventory;
  const identity = {
    schema: "vize.canonical-corpus-identity",
    version: 1,
    ...context,
    tree: "f".repeat(40),
    gitlinks,
    gitlinksSha256: sha256(JSON.stringify(gitlinks)),
    modulesSha256: "c".repeat(64),
    files: files.length,
    filesSha256: sha256(JSON.stringify(files)),
    committedSha256: sha256(JSON.stringify(proof)),
  };
  const verify = (concurrent = false) =>
    concurrent
      ? verifyCanonicalArtifacts(selection, root, identity, validateArtifactsConcurrently)
      : verifyCanonicalArtifacts(selection, root, identity);
  try {
    writeArtifacts(root, inventory, identity);
    assert.deepEqual(verify(), identity);
    assert.deepEqual(verify(true), identity);
    const reach = join(root, selection.workers[2]!.artifactName);
    const vector = join(reach, "reach-davinci_production_reach-shard-05-of-16.nul");
    const original = readFileSync(vector);
    const paths = original.toString("utf8").split("\0").filter(Boolean);
    writeFileSync(vector, `${[...paths].reverse().join("\0")}\0`);
    assert.throws(() => verify(), /shard vectors were replaced/);
    assert.throws(() => verify(true), /shard vectors were replaced/);
    writeFileSync(vector, original);
    const receiptPath = join(reach, "observer.json");
    const receipt = readFileSync(receiptPath);
    const { vectorSha256: _, ...unpinned } = JSON.parse(receipt.toString("utf8")) as Record<
      string,
      unknown
    >;
    write(receiptPath, unpinned);
    assert.throws(() => verify(), /shard vectors were replaced/);
    writeFileSync(receiptPath, receipt);
    rmSync(vector);
    assert.throws(() => verify(), /ENOENT/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
