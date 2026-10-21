export interface ReleaseRunFacts {
  repository: string;
  workflowPath: string;
  event: string;
  branch: string;
  status: string;
  conclusion: string | null;
  headSha: string;
}

export interface ResumePolicyInput {
  repository: string;
  run: ReleaseRunFacts;
  tagParent: string;
  artifactNames: readonly string[];
  requiredArtifacts: readonly string[];
  targetVersion: string;
  higherVersions: readonly string[];
  incompleteHigherPackages: readonly string[];
}

const stableVersion = /^(\d+)\.(\d+)\.(\d+)$/;

export function compareStableVersions(left: string, right: string): number | null {
  const a = stableVersion.exec(left);
  const b = stableVersion.exec(right);
  if (a === null || b === null) return null;
  for (let index = 1; index <= 3; index++) {
    const difference = Number(a[index]) - Number(b[index]);
    if (difference !== 0) return difference;
  }
  return 0;
}

export function higherStableVersions(target: string, versions: readonly string[]): string[] {
  if (stableVersion.exec(target) === null) return [];
  return versions
    .filter((version) => (compareStableVersions(version, target) ?? -1) > 0)
    .toSorted((left, right) => compareStableVersions(left, right) ?? 0);
}

export function backfillTag(version: string): string {
  return `backfill-${version.replaceAll(".", "-")}`;
}

export function validateResumePolicy(input: ResumePolicyInput): string[] {
  const errors: string[] = [];
  if (stableVersion.exec(input.targetVersion) === null)
    errors.push("version must be stable semver");
  if (input.run.repository !== input.repository) errors.push("run belongs to another repository");
  if (input.run.workflowPath !== ".github/workflows/release.yml")
    errors.push("run did not use release.yml");
  if (input.run.event !== "workflow_dispatch") errors.push("run was not manually dispatched");
  if (input.run.branch !== "main") errors.push("run was not dispatched on main");
  if (input.run.status !== "completed") errors.push("run is not complete");
  if (!["failure", "cancelled"].includes(input.run.conclusion ?? ""))
    errors.push("only failed or cancelled runs can be resumed");
  if (input.run.headSha !== input.tagParent) errors.push("tag does not belong to the selected run");
  const artifacts = new Set(input.artifactNames);
  for (const required of input.requiredArtifacts) {
    if (!artifacts.has(required)) errors.push(`missing release artifact: ${required}`);
  }
  if (input.incompleteHigherPackages.length > 0) {
    errors.push(
      `newer releases are incomplete: ${input.incompleteHigherPackages.toSorted().join(", ")}`,
    );
  }
  return errors;
}
