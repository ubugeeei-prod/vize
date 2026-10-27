// Internal operational budget, not a guaranteed registry contract.
export const CRATE_QUOTA_WINDOW_MS = 24 * 60 * 60 * 1000;
export const CRATE_QUOTA_MARGIN_MS = 2 * 60 * 1000;
export const CRATE_QUOTA_BUDGET = 19;
export const PAGE_SIZE = 100;
export const MAX_PAGES = 10;
export const MAX_BYTES = 2 * 1024 * 1024;
export const ORIGIN = "https://crates.io";

export interface ReleaseOwnership {
  version: string;
  runId: string;
  workflowHead: string;
  candidateSha: string;
  producerSource: string;
  runAttempt?: number;
  nativeManifestGitBlob?: string;
  inventoryGitBlob?: string;
}
export interface RegistrySnapshot {
  url: string;
  status: number;
  date: string | null;
  age: string | null;
  body: string;
}
export interface HistoryVersion {
  id: number;
  num: string;
  timestamp: number;
  createdAt: string;
  yanked: boolean;
  checksum: string;
  trustpub: unknown;
}
export interface CrateQuotaObservation {
  crate: string;
  totalVersions: number;
  windowCount: number;
  newWrites: number;
  proposedCount: number;
  allowed: boolean;
  nextEligibleAt: string;
  recent: HistoryVersion[];
  pages: Array<Omit<RegistrySnapshot, "body"> & { bytes: number; sha256: string }>;
  priorMissing: null | (Omit<RegistrySnapshot, "body"> & { bytes: number; sha256: string });
  ownVersion: null | {
    checksum: string;
    producerProofSha256: string;
    source: OwnedCrateSourceProof;
    proof: Omit<RegistrySnapshot, "body"> & { bytes: number; sha256: string };
  };
}
export interface RegistryReadReceipt {
  url: string;
  status: number | null;
  date: string | null;
  age: string | null;
  bytes: number | null;
  sha256: string | null;
  completeBody: boolean;
  error?: string;
}
export interface CrateQuotaReport {
  kind: "allowed" | "blocked" | "unavailable";
  checkedAt: string;
  budget: number;
  reserve: number;
  marginMs: number;
  remainingCrateWrites: number;
  nextEligibleAt: string | null;
  observations: CrateQuotaObservation[];
  failures: { crate: string; message: string }[];
  requests: RegistryReadReceipt[];
  produced: ProducedCandidateProof | null;
}

export interface ProducerCandidateRecord {
  schema: "vize.releaseProducerCandidate.v1";
  repository: string;
  runId: string;
  runAttempt: number;
  workflowHead: string;
  version: string;
  candidateSha: string;
  producerSource: string;
  nativeManifestGitBlob: string;
  inventoryGitBlob: string;
}
export interface ProducedCandidateProof {
  record: ProducerCandidateRecord;
  artifactId: number;
  artifactName: string;
  artifactApi: string;
  archiveBytes: number;
  archiveSha256: string;
  memberSha256: string;
  memberBytes: number;
  memberRawBase64: string;
  metadata: unknown;
}
export interface OwnedCrateSourceProof {
  candidateSha: string;
  archiveUrl: string;
  archiveBytes: number;
  archiveSha256: string;
  vcsPath: string;
  vcsBytes: number;
  vcsSha256: string;
  vcsRawBase64: string;
}
