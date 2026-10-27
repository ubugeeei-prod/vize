// Inert R=42 mock packets only. Never upload these as a real producer artifact.
import { createHash } from "node:crypto";
import { writeZip } from "../../../tools/support/archive/zip.ts";
import { writeTarGz } from "../../../tools/support/archive/tar.ts";
import type {
  OwnedCrateSourceProof,
  ProducedCandidateProof,
  ProducerCandidateRecord,
  ReleaseOwnership,
} from "../../../tools/support/release/release/quota/types.ts";

const hash = (bytes: Uint8Array | string) => createHash("sha256").update(bytes).digest("hex");
export function mockProduced(owner: ReleaseOwnership) {
  if (owner.runAttempt === undefined || !owner.nativeManifestGitBlob || !owner.inventoryGitBlob)
    throw new Error("complete inert producer facts required");
  const record: ProducerCandidateRecord = {
    schema: "vize.releaseProducerCandidate.v1",
    repository: "ubugeeei-prod/vize",
    runId: owner.runId,
    runAttempt: owner.runAttempt,
    workflowHead: owner.workflowHead,
    version: owner.version,
    candidateSha: owner.candidateSha,
    producerSource: owner.producerSource,
    nativeManifestGitBlob: owner.nativeManifestGitBlob,
    inventoryGitBlob: owner.inventoryGitBlob,
  };
  const raw = Buffer.from(JSON.stringify(record) + "\n");
  const archive = writeZip([{ name: "producer-candidate.json", data: raw, deflate: true }]);
  const name = "release-producer-source-" + owner.runAttempt;
  const metadata = {
    id: 7,
    name,
    expired: false,
    size_in_bytes: archive.length,
    digest: "sha256:" + hash(archive),
    workflow_run: { id: Number(owner.runId), head_sha: owner.workflowHead, head_branch: "main" },
  };
  const proof: ProducedCandidateProof = {
    record,
    artifactId: 7,
    artifactName: name,
    artifactApi: "https://api.github.com/repos/ubugeeei-prod/vize/actions/artifacts/7",
    archiveBytes: archive.length,
    archiveSha256: hash(archive),
    memberSha256: hash(raw),
    memberBytes: raw.length,
    memberRawBase64: raw.toString("base64"),
    metadata,
  };
  return { archive, metadata, proof };
}
export function mockProducerResponse(
  owner: ReleaseOwnership,
  input: string | URL | Request,
  init?: RequestInit,
): Response | null {
  const url = new URL(input instanceof Request ? input.url : input),
    fixture = mockProduced(owner);
  if (url.origin === "https://api.github.com" && url.pathname.endsWith("/artifacts")) {
    return new Response(JSON.stringify({ total_count: 1, artifacts: [fixture.metadata] }));
  }
  if (url.origin === "https://api.github.com" && url.pathname.endsWith("/artifacts/7")) {
    return new Response(JSON.stringify(fixture.metadata));
  }
  if (url.origin === "https://api.github.com" && url.pathname.endsWith("/artifacts/7/zip")) {
    return new Response(null, {
      status: 302,
      headers: { Location: "https://artifact.invalid/proof.zip" },
    });
  }
  if (url.origin === "https://artifact.invalid") {
    if (new Headers(init?.headers).has("Authorization"))
      throw new Error("token leaked to signed storage");
    return new Response(new Uint8Array(fixture.archive));
  }
  return null;
}
export function mockCrate(crate: string, version: string, candidate: string) {
  const vcsPath = crate + "-" + version + "/.cargo_vcs_info.json";
  const raw = Buffer.from(
    JSON.stringify({ git: { sha1: candidate, dirty: true }, path_in_vcs: "crates/" + crate }) +
      "\n",
  );
  const archive = writeTarGz([{ name: vcsPath, data: raw }]);
  const checksum = hash(archive);
  const proof: OwnedCrateSourceProof = {
    candidateSha: candidate,
    archiveUrl: `https://static.crates.io/crates/${crate}/${crate}-${version}.crate`,
    archiveBytes: archive.length,
    archiveSha256: checksum,
    vcsPath,
    vcsBytes: raw.length,
    vcsSha256: hash(raw),
    vcsRawBase64: raw.toString("base64"),
  };
  return { archive, checksum, proof };
}
