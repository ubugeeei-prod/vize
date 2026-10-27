import { execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { CRATE_INVENTORY_PATH, releaseCrateInventory } from "./quota/inventory.ts";
import type { ProducerCandidateRecord } from "./quota/types.ts";

const validSha = (value: string) => /^[a-f0-9]{40}$/.test(value);
export function producerCandidateRecord(
  facts: ProducerCandidateRecord,
  physicalHead: string,
  rawCommit: string,
  nativeManifest: string,
): ProducerCandidateRecord {
  if (
    facts.schema !== "vize.releaseProducerCandidate.v1" ||
    facts.repository !== "ubugeeei-prod/vize" ||
    !/^[1-9]\d*$/.test(facts.runId) ||
    !Number.isSafeInteger(facts.runAttempt) ||
    facts.runAttempt < 1 ||
    !/^\d+\.\d+\.\d+$/.test(facts.version) ||
    ![
      facts.candidateSha,
      facts.producerSource,
      facts.workflowHead,
      facts.nativeManifestGitBlob,
      facts.inventoryGitBlob,
    ].every(validSha) ||
    physicalHead !== facts.candidateSha
  )
    throw new Error("actual release producer facts required");
  const headers = rawCommit.split("\n\n", 1)[0] ?? "";
  const parents = headers
    .split("\n")
    .filter((line) => line.startsWith("parent "))
    .map((line) => line.slice(7));
  if (parents.length !== 1 || parents[0] !== facts.producerSource)
    throw new Error("producer candidate must have its exact single source parent");
  const manifest: unknown = JSON.parse(nativeManifest);
  if (
    !manifest ||
    typeof manifest !== "object" ||
    Array.isArray(manifest) ||
    (manifest as Record<string, unknown>).name !== "@vizejs/native" ||
    (manifest as Record<string, unknown>).version !== facts.version
  )
    throw new Error("committed native release version differs");
  return facts;
}
export function captureProducerCandidate(version: string, candidate: string, output: string): void {
  if (
    process.env.GITHUB_ACTIONS !== "true" ||
    process.env.GITHUB_REPOSITORY !== "ubugeeei-prod/vize" ||
    process.env.GITHUB_REF !== "refs/heads/main" ||
    process.env.GITHUB_WORKFLOW_REF !==
      "ubugeeei-prod/vize/.github/workflows/release.yml@refs/heads/main"
  )
    throw new Error(
      "producer attestation is emitted only by actual main Release candidate creation",
    );
  if (!validSha(candidate) || !/^\d+\.\d+\.\d+$/.test(version))
    throw new Error("stable produced version and full candidate required");
  const temp = process.env.RUNNER_TEMP;
  if (
    !temp ||
    !path.resolve(output).startsWith(path.resolve(temp) + path.sep) ||
    path.basename(output) !== "producer-candidate.json"
  )
    throw new Error("owned producer receipt output required");
  const git = (...args: string[]) =>
    execFileSync("git", ["--no-replace-objects", ...args], {
      encoding: "utf8",
      maxBuffer: 2 * 1024 * 1024,
    });
  const raw = git("cat-file", "commit", candidate),
    headers = raw.split("\n\n", 1)[0] ?? "";
  const source =
    headers
      .split("\n")
      .find((line) => line.startsWith("parent "))
      ?.slice(7) ?? "";
  const manifest = git("cat-file", "blob", candidate + ":npm/native/package.json");
  releaseCrateInventory(git("cat-file", "blob", candidate + ":" + CRATE_INVENTORY_PATH));
  const record = producerCandidateRecord(
    {
      schema: "vize.releaseProducerCandidate.v1",
      repository: "ubugeeei-prod/vize",
      runId: process.env.GITHUB_RUN_ID ?? "",
      runAttempt: Number(process.env.GITHUB_RUN_ATTEMPT),
      workflowHead: process.env.GITHUB_SHA ?? "",
      version,
      candidateSha: candidate,
      producerSource: source,
      nativeManifestGitBlob: git("rev-parse", candidate + ":npm/native/package.json").trim(),
      inventoryGitBlob: git("rev-parse", candidate + ":" + CRATE_INVENTORY_PATH).trim(),
    },
    git("rev-parse", "HEAD").trim(),
    raw,
    manifest,
  );
  mkdirSync(path.dirname(output), { recursive: true });
  writeFileSync(output, JSON.stringify(record, null, 2) + "\n");
}
if (import.meta.main) {
  captureProducerCandidate(process.argv[2] ?? "", process.argv[3] ?? "", process.argv[4] ?? "");
}
