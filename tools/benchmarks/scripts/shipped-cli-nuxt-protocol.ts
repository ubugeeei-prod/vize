export const labels = ["base", "head", "base-twin", "head-twin"] as const;
export type Label = (typeof labels)[number];
export type Output = {
  fileCount: number;
  errorCount: number;
  warningCount: number;
  files: unknown[];
  programs: Array<{ files: string[] }>;
};
export type Snapshot = {
  files: Record<string, string>;
  expectedOutput: Output;
  programInputSha256: Record<string, string>;
};
export type Receipt = {
  scope: string;
  originalProject: string;
  originalDependencies: string;
  snapshotSha256: string;
  snapshotJsonSha256: string;
  lockSha256: string;
  packageSha256: string;
  fixtureFileCount: number;
  reportedFileCount: number;
  programInputCount: number;
  runtime: Record<string, string>;
  producer: {
    repository: string;
    runId: number;
    driverSha: string;
    workflow: string;
    artifactId: number;
    artifactName: string;
    artifactSha256: string;
    baseSha: string;
    headSha: string;
    sourceDateEpoch: string;
    target: string;
    cli: Record<"base" | "head", { sha256: string; sizeBytes: number }>;
  };
};
export type RecordRow = {
  index: number;
  label: Label;
  elapsed: number;
  user: number;
  system: number;
  maxRssKiB: number;
  outputSha256: string;
};

export function balancedOrder(): Label[] {
  return Array.from(
    { length: 4 },
    () =>
      [
        "base",
        "head",
        "head-twin",
        "base-twin",
        "head-twin",
        "base-twin",
        "base",
        "head",
      ] as Label[],
  ).flat();
}
