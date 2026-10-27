import { createHash } from "node:crypto";
import { Buffer } from "node:buffer";
import type { ReleaseOwnership, ProducedCandidateProof } from "./quota/types.ts";
import { readProducedCandidate } from "./quota/producer.ts";
import { CRATE_INVENTORY_PATH, decodeCrateInventoryFile } from "./quota/inventory.ts";

type GithubRead = (path: string) => Promise<unknown>;
function requireValue(value: unknown, message: string): asserts value {
  if (!value) throw new Error(message);
}
function object(value: unknown): Record<string, unknown> {
  requireValue(
    value !== null && typeof value === "object" && !Array.isArray(value),
    "GitHub object required",
  );
  return value as Record<string, unknown>;
}
function sha(value: unknown): string {
  requireValue(
    typeof value === "string" && /^[a-f0-9]{40}$/.test(value),
    "GitHub commit SHA required",
  );
  return value;
}

/** Bind recovery to the immutable tag and the genuine original Release workflow. */
export async function readReleaseIdentity(
  version: string,
  candidate: string,
  originalRun: string,
  github: GithubRead,
): Promise<ReleaseOwnership> {
  requireValue(/^\d+\.\d+\.\d+$/.test(version), "stable release version required");
  sha(candidate);
  requireValue(/^[1-9]\d*$/.test(originalRun), "original Release run ID required");
  const [runValue, refValue, commitValue, manifestValue] = await Promise.all([
    github(`actions/runs/${originalRun}`),
    github(`git/ref/tags/v${version}`),
    github(`git/commits/${candidate}`),
    github(`contents/npm/native/package.json?ref=${candidate}`),
  ]);
  const run = object(runValue),
    repository = object(run.repository),
    ref = object(object(refValue).object);
  requireValue(
    String(run.id) === originalRun && repository.full_name === "ubugeeei-prod/vize",
    "original run identity differs",
  );
  requireValue(
    run.path === ".github/workflows/release.yml" &&
      run.event === "workflow_dispatch" &&
      run.head_branch === "main",
    "original run is not the main Release workflow",
  );
  const workflowHead = sha(run.head_sha);
  requireValue(
    Number.isSafeInteger(run.run_attempt) && Number(run.run_attempt) > 0,
    "original run attempt required",
  );
  requireValue(ref.type === "commit" && ref.sha === candidate, "immutable release tag differs");
  const commit = object(commitValue);
  requireValue(
    commit.sha === candidate && Array.isArray(commit.parents) && commit.parents.length === 1,
    "prepared release commit identity/parent differs",
  );
  const producerSource = sha(object(commit.parents[0]).sha);
  const manifest = object(manifestValue);
  requireValue(
    manifest.encoding === "base64" && typeof manifest.content === "string",
    "prepared package manifest required",
  );
  const manifestBytes = Buffer.from(manifest.content, "base64");
  const nativeManifestGitBlob = createHash("sha1")
    .update("blob " + manifestBytes.length + "\0")
    .update(manifestBytes)
    .digest("hex");
  requireValue(manifest.sha === nativeManifestGitBlob, "prepared manifest Git blob differs");
  const packageJson = object(
    JSON.parse(Buffer.from(manifest.content, "base64").toString("utf8")) as unknown,
  );
  requireValue(
    packageJson.name === "@vizejs/native" && packageJson.version === version,
    "prepared manifest version differs",
  );
  const compared = object(await github(`compare/${producerSource}...${workflowHead}`));
  requireValue(
    ["ahead", "identical"].includes(String(compared.status)) &&
      object(compared.base_commit).sha === producerSource,
    "producer is not an ancestor of the original main workflow head",
  );
  return {
    version,
    runId: originalRun,
    workflowHead,
    candidateSha: candidate,
    producerSource,
    runAttempt: Number(run.run_attempt),
    nativeManifestGitBlob,
  };
}

export async function readReleaseOwnership(
  version: string,
  candidate: string,
  originalRun: string,
  github: GithubRead,
  fetchImpl: typeof fetch = globalThis.fetch,
): Promise<ReleaseOwnership & { produced: ProducedCandidateProof }> {
  const identity = await readReleaseIdentity(version, candidate, originalRun, github);
  const inventory = decodeCrateInventoryFile(
    await github(`contents/${CRATE_INVENTORY_PATH}?ref=${candidate}`),
  );
  const owned = { ...identity, inventoryGitBlob: inventory.gitBlob };
  return { ...owned, produced: await readProducedCandidate(owned, fetchImpl) };
}
