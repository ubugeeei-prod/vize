export interface FileFacts {
  path: string;
  realPath: string;
  kind: "file" | "directory" | "symlink" | "other";
  body: Buffer | null;
}
export interface PublicPackageFacts {
  name: string;
  version: string;
  directory: string;
  manifest: Buffer;
  /** Null only for an archive-only reader; never a fabricated tarball. */
  archive: Buffer | null;
  integrity: string;
  resolved: string;
}
export interface HumanBinFacts {
  role: "runtime-prop-null-11252" | "imports-vue-ts-11253";
  host: { platform: "darwin" | "linux"; arch: "arm64" | "x64" };
  root: string;
  configuration: import("./bin-configuration.ts").BinConfiguration;
  parent: PublicPackageFacts;
  owner: PublicPackageFacts;
  /** Every dependency/optional/peer declared by the authenticated parent,
   * including public owner manifests that declare no executable. */
  dependencyOwners: PublicPackageFacts[];
  graph: {
    parentDirectory: string;
    dependencyName: string;
    version: string;
    directory: string;
  }[];
  lock: Buffer;
  manager: null | {
    reportedVersion: string;
    versionStdout: Buffer;
    versionStatus: number | null;
    versionSignal: string | null;
    versionError: string | null;
    versionStderr: Buffer;
    originalArgv: string[];
    originalImage: { path: string; bytes: number; sha256: string };
    currentImage: FileFacts;
  };
  /** Comes from the independently authenticated reviewed-controller file row.
   * Never a hash copied from a capture or an observed shim body. */
  expectedInputPolicyGitBlob: string | null;
  inputPolicyRaw: Buffer | null;
  settings: {
    userConfig: Buffer;
    globalConfig: Buffer;
    rootNpmrc: Buffer | null;
    rootWorkspace: Buffer;
    rootPackage: Buffer;
  };
  targets: FileFacts[];
  inventory: FileFacts[];
}
export interface DerivedBin {
  name: string;
  target: string;
  publicTargetSha256: string;
  runtime: "node" | null;
  shimPath: string;
  nodePath: string[];
  relocatableRoot: string | null;
}
