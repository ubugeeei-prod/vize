import { createHash } from "node:crypto";
import { readZipEntry, zipEntries } from "../../../archive/zip.ts";
import { boundedBytes, githubSnapshot } from "./http.ts";
import type { ProducedCandidateProof, ProducerCandidateRecord, ReleaseOwnership } from "./types.ts";

const prefix = "release-producer-source-";
function requireValue(value: unknown, message: string): asserts value {
  if (!value) throw new Error(message);
}
function object(value: unknown): Record<string, unknown> {
  requireValue(
    value !== null && typeof value === "object" && !Array.isArray(value),
    "producer API object required",
  );
  return value as Record<string, unknown>;
}
export class MissingProducerAttestation extends Error {
  readonly code = "VIZE_PRODUCER_ATTESTATION_UNAVAILABLE";
  readonly witness: { runId: string; total: number; pages: unknown[] };
  constructor(message: string, witness: { runId: string; total: number; pages: unknown[] }) {
    super(message);
    this.witness = witness;
  }
}
export function validateProducedRecord(
  raw: Buffer,
  ownership: ReleaseOwnership,
  artifactName: string,
): ProducerCandidateRecord {
  requireValue(
    raw.length > 0 && raw.length <= 16 * 1024,
    "bounded producer candidate record required",
  );
  const record = object(
    JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(raw)) as unknown,
  );
  requireValue(
    record.schema === "vize.releaseProducerCandidate.v1" &&
      record.repository === "ubugeeei-prod/vize",
    "producer record schema/repository differs",
  );
  requireValue(
    record.runId === ownership.runId &&
      record.workflowHead === ownership.workflowHead &&
      record.version === ownership.version,
    "producer run/workflow/version differs",
  );
  requireValue(
    Number.isSafeInteger(record.runAttempt) &&
      Number(record.runAttempt) > 0 &&
      record.runAttempt === Number(artifactName.slice(prefix.length)) &&
      ownership.runAttempt !== undefined &&
      Number(record.runAttempt) <= ownership.runAttempt,
    "producer attempt differs",
  );
  requireValue(
    record.candidateSha === ownership.candidateSha &&
      record.producerSource === ownership.producerSource,
    "original Release produced a different source/candidate",
  );
  requireValue(
    typeof ownership.nativeManifestGitBlob === "string" &&
      typeof ownership.inventoryGitBlob === "string" &&
      record.nativeManifestGitBlob === ownership.nativeManifestGitBlob &&
      record.inventoryGitBlob === ownership.inventoryGitBlob,
    "producer committed manifest/cohort proof differs",
  );
  return record as unknown as ProducerCandidateRecord;
}
async function candidateArtifactIds(
  ownership: ReleaseOwnership,
  fetchImpl: typeof fetch,
): Promise<number[]> {
  const ids: number[] = [],
    seen = new Set<number>(),
    names = new Set<string>();
  let total: number | null = null;
  const pages: unknown[] = [];
  for (let page = 1; page <= 10; page += 1) {
    const snapshot = await githubSnapshot(
      `actions/runs/${ownership.runId}/artifacts?per_page=100&page=${page}`,
      fetchImpl,
    );
    const { value, ...pageWitness } = snapshot;
    pages.push(pageWitness);
    const body = object(value);
    requireValue(
      Number.isSafeInteger(body.total_count) &&
        Number(body.total_count) >= 0 &&
        Array.isArray(body.artifacts) &&
        body.artifacts.length <= 100,
      "producer artifact inventory malformed",
    );
    requireValue(
      total === null || total === body.total_count,
      "producer artifact inventory changed",
    );
    total = Number(body.total_count);
    for (const value of body.artifacts) {
      const artifact = object(value),
        id = Number(artifact.id);
      requireValue(
        Number.isSafeInteger(id) && id > 0 && !seen.has(id),
        "duplicate/invalid producer artifact ID",
      );
      seen.add(id);
      if (typeof artifact.name === "string" && artifact.name.startsWith(prefix)) {
        requireValue(
          /^release-producer-source-[1-9]\d*$/.test(artifact.name) && !names.has(artifact.name),
          "producer artifact attempt/name not unique",
        );
        names.add(artifact.name);
        if (artifact.expired === false) ids.push(id);
      }
    }
    if (seen.size === total) {
      if (names.size === 0)
        throw new MissingProducerAttestation(
          "legacy Release has no genuine producer-source artifact; automatic recovery refused",
          { runId: ownership.runId, total, pages },
        );
      requireValue(
        ids.length > 0 && ids.length <= 10,
        "producer artifacts expired or exceed proof bound",
      );
      return ids;
    }
    requireValue(
      seen.size < total && body.artifacts.length > 0,
      "producer inventory coverage unavailable",
    );
  }
  throw new Error("producer artifact inventory exceeds page bound");
}
async function readProducerZip(
  metadata: Record<string, unknown>,
  fetchImpl: typeof fetch,
): Promise<Buffer> {
  const id = Number(metadata.id),
    size = Number(metadata.size_in_bytes);
  requireValue(
    Number.isSafeInteger(id) &&
      id > 0 &&
      Number.isSafeInteger(size) &&
      size > 0 &&
      size <= 128 * 1024 &&
      typeof metadata.digest === "string" &&
      /^sha256:[a-f0-9]{64}$/.test(metadata.digest),
    "small immutable producer ZIP required",
  );
  const token = process.env.GITHUB_TOKEN;
  const authorization: Record<string, string> = token ? { Authorization: "Bearer " + token } : {};
  const response = await fetchImpl(
    `https://api.github.com/repos/ubugeeei-prod/vize/actions/artifacts/${id}/zip`,
    { redirect: "manual", signal: AbortSignal.timeout(15_000), headers: authorization },
  );
  const location = response.headers.get("location");
  requireValue(
    [301, 302, 303, 307, 308].includes(response.status) && location !== null,
    "producer ZIP redirect required",
  );
  await response.body?.cancel();
  const target = new URL(location);
  requireValue(
    target.protocol === "https:" && !target.username && !target.password,
    "producer ZIP HTTPS location required",
  );
  // Never forward the GitHub token to signed artifact storage.
  const download = await fetchImpl(target, {
    redirect: "error",
    signal: AbortSignal.timeout(15_000),
  });
  requireValue(download.status === 200, "producer ZIP download failed");
  const bytes = await boundedBytes(download, 128 * 1024);
  requireValue(
    bytes.length === size &&
      "sha256:" + createHash("sha256").update(bytes).digest("hex") === metadata.digest,
    "producer ZIP digest/size differs",
  );
  return bytes;
}
/** Authenticate original R/H service metadata, whole tiny ZIP digest, and the actual produced C. */
export async function readProducedCandidate(
  ownership: ReleaseOwnership,
  fetchImpl: typeof fetch = globalThis.fetch,
): Promise<ProducedCandidateProof> {
  const ids = await candidateArtifactIds(ownership, fetchImpl),
    matches: ProducedCandidateProof[] = [];
  for (const id of ids) {
    const snapshot = await githubSnapshot(`actions/artifacts/${id}`, fetchImpl);
    const metadata = object(snapshot.value);
    const run = object(metadata.workflow_run);
    requireValue(
      metadata.id === id &&
        metadata.expired === false &&
        String(run.id) === ownership.runId &&
        run.head_sha === ownership.workflowHead &&
        run.head_branch === "main" &&
        typeof metadata.name === "string" &&
        /^release-producer-source-[1-9]\d*$/.test(metadata.name),
      "producer artifact API workflow binding differs",
    );
    const archive = await readProducerZip(metadata, fetchImpl),
      entries = zipEntries(archive);
    requireValue(entries.length === 1, "producer ZIP must contain exactly its one record");
    const entry = entries[0];
    requireValue(
      entry !== undefined &&
        entry.name === "producer-candidate.json" &&
        !entry.isDirectory &&
        entry.size <= 16 * 1024,
      "producer record member identity/size differs",
    );
    const raw = readZipEntry(archive, entry),
      parsed = object(JSON.parse(raw.toString("utf8")) as unknown);
    if (parsed.candidateSha !== ownership.candidateSha) continue; // A different genuine creation attempt grants no ownership.
    const record = validateProducedRecord(raw, ownership, metadata.name);
    matches.push({
      record,
      artifactId: id,
      artifactName: metadata.name,
      artifactApi: `https://api.github.com/repos/ubugeeei-prod/vize/actions/artifacts/${id}`,
      archiveBytes: archive.length,
      archiveSha256: createHash("sha256").update(archive).digest("hex"),
      memberSha256: createHash("sha256").update(raw).digest("hex"),
      memberBytes: raw.length,
      memberRawBase64: raw.toString("base64"),
      metadata: snapshot,
    });
  }
  requireValue(
    matches.length === 1,
    "original Release produced no unique matching intended candidate",
  );
  return matches[0]!;
}
