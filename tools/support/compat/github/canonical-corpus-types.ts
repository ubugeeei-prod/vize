// Authenticated corpus graph, kernel observation and native receipt data.
export interface CorpusNode {
  path: string;
  mode: string;
  id?: string;
  target?: string;
  repository?: string;
  revision?: string;
}
export type CorpusNodes = Map<string, CorpusNode>;
export type CorpusChildren = Map<string, Set<string>>;
export interface RejectedPath {
  path: string;
  code: string;
  followed: number;
}
export interface Boundary {
  code: string;
  firstRejected: number;
  rejected: RejectedPath[];
}
export interface KernelPrefixWitness {
  path: string;
  followed: number;
  preceding: { path: string; followed: number };
}
export type VueFile = [string, string, number, string];
type Gitlink = { path: string; sha: string };
type GitObject = { id: string; type: string; bytes: string };
export interface CorpusProof {
  schema: string;
  version: number;
  repositories: Array<Gitlink & { objects: GitObject[] }>;
  kernelBoundary?: Boundary;
  traversal?: "finite-physical-edges";
}
interface CollectorDigests {
  sourceSha256: string;
  bodySha256: string;
  vectorSha256: string;
}
export interface NativeWalkIdentity extends CollectorDigests {
  boundarySha256: string;
  originalVectorSha256?: string;
  originalFilesSha256?: string;
}
export interface CorpusIdentity {
  sha: string;
  tree: string;
  runId: number;
  attempt: number;
  gitlinks: Gitlink[];
  nativeWalk?: NativeWalkIdentity;
}
export interface NativeReceipt
  extends CollectorDigests, Pick<CorpusIdentity, "sha" | "tree" | "runId" | "attempt"> {
  schema: string;
  version: number;
  platform: string;
  architecture: string;
  harnessSha256: string;
  binarySha256: string;
  toolchainSha256: string;
  files: number;
  boundary: Boundary;
  traversal?: "finite-physical-edges";
  originalVectorSha256: string;
  originalFilesSha256: string;
}
export type PhysicalEntry = {
  path: string;
  mode: number;
  size: number;
  mtimeMs: number;
  ctimeMs: number;
  targetBase64?: string;
};
export type MetadataEntry = {
  path: string;
  isDirectory?: boolean;
  followed?: number;
  error?: Pick<NodeJS.ErrnoException, "code" | "errno" | "syscall" | "path" | "message">;
};
export interface NativeObservation {
  physical: PhysicalEntry[] | null;
  metadata: MetadataEntry[] | null;
  files: VueFile[];
  rejected: Boundary["rejected"];
  successfulFollows: number;
}
