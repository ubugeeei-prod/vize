export const projectionKinds = ["baseline", "incident"] as const;
export type ProjectionKind = (typeof projectionKinds)[number];

export const projectionFamilies = ["dual-scripts-emits", "define-model"] as const;
export type ProjectionFamily = (typeof projectionFamilies)[number];

export type ProjectionBytes = string | Uint8Array;

export interface HistoricalEdit {
  beforeStart: number;
  beforeEnd: number;
  afterStart: number;
  afterEnd: number;
}

export interface ArtifactIdentity {
  sourceSha: string;
  driverSha: string;
  runId: number;
  jobId: number;
  artifactId: number;
  artifactBytes: number;
  createdAt: string;
  expiresAt: string;
  archiveBytes: number;
  archiveSha256: string;
  manifestSha256: string;
  receiptVerifierSha256?: string;
  qualification: string;
}

export interface ArtifactCustody {
  baseline: ArtifactIdentity;
  incident: ArtifactIdentity;
}

export interface RetainedPacketRow {
  path: string;
  bytes: number;
  mode: number;
  sha256: string;
  representation?: "inline" | "empty" | "same-as" | "derived-import-probes";
  source?: string;
}

export interface RetainedPacketManifest {
  files: RetainedPacketRow[];
}

export interface ProjectionEditScript {
  id: string;
  family: ProjectionFamily;
  surface: "canon" | "content-mapper" | "maestro";
  path: string;
  beforeBytes: number;
  beforeSha256: string;
  afterBytes: number;
  afterSha256: string;
  edits: HistoricalEdit[];
}

export interface CoordinateSurfaceProof {
  path: string;
  baselineRows: number;
  incidentRows: number;
  matchedRows: number;
  addedRows: number;
  rejectedRows: number;
  missingRows: number;
  matchedSha256: string;
  addedSha256: string;
  rejectedSha256: string;
  missingSha256: string;
}

export interface MaestroDocumentProof {
  family: ProjectionFamily;
  templateUri: string;
  documents: number;
  normalizedManifestSha256: string;
}

export interface ProjectionCoordinateProof {
  schema: number;
  surfaces: CoordinateSurfaceProof[];
  documents: MaestroDocumentProof[];
  sha256: string;
}

export interface RetainedPayloadManifest {
  file: string;
  compression: "gzip";
  bytes: number;
  sha256: string;
  uncompressedBytes: number;
  uncompressedSha256: string;
  blobCount: number;
}

export interface RetainedProjectionManifest {
  schema: number;
  kind: "vize.template-emit-projection-history";
  families: readonly ProjectionFamily[];
  artifacts: ArtifactCustody;
  packets: Record<ProjectionKind, RetainedPacketManifest>;
  editScripts: ProjectionEditScript[];
  coordinateProof: ProjectionCoordinateProof;
  payload: RetainedPayloadManifest;
}

export type RetainedProjectionPackets = Record<ProjectionKind, Map<string, Buffer>>;

export interface RetainedProjectionFixture {
  manifest: RetainedProjectionManifest;
  packets: RetainedProjectionPackets;
}

export interface BuildRetainedProjectionFixtureOptions {
  baselineRoot: string;
  incidentRoot: string;
  baselineArtifact: ArtifactIdentity;
  incidentArtifact: ArtifactIdentity;
  manifestPath: string;
  payloadPath: string;
}

export interface RetainedInitializerEdit {
  text: string;
  cut: number;
  end: number;
  delta: number;
}

export interface FrozenCanonProjection {
  text: string;
  "pre-rewrite-text": string;
  mappings: string;
  "semantic-links": string;
  "import-source-map": string;
  "import-source-map-probes": string;
}

export interface FrozenMapperProjection {
  text: string;
  mappings: string;
  "semantic-links": string;
  diagnostics: string;
  "authored-hits": string;
}

export interface FrozenMaestroProjection {
  text: string;
  mappings: string;
  "authored-hits": string;
}

export interface FrozenProjectionFamily {
  "source-sha256": string;
  canon: FrozenCanonProjection;
  "content-mapper": FrozenMapperProjection;
  maestro: FrozenMaestroProjection;
}

export type FrozenProjectionBaseline = Record<ProjectionFamily, FrozenProjectionFamily>;
