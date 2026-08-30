import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");

export function normalizeRepoText(content: string): string {
  return content.replace(/\r\n?/g, "\n");
}

export function readRepoFile(...segments: string[]): string {
  return normalizeRepoText(fs.readFileSync(path.join(root, ...segments), "utf8"));
}

export function readGithubYamlFiles(): Array<{ relativePath: string; content: string }> {
  const files: Array<{ relativePath: string; content: string }> = [];
  const visit = (directory: string) => {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const fullPath = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        visit(fullPath);
        continue;
      }
      if (!/\.(ya?ml)$/.test(entry.name)) {
        continue;
      }
      files.push({
        relativePath: path.relative(root, fullPath),
        content: normalizeRepoText(fs.readFileSync(fullPath, "utf8")),
      });
    }
  };
  visit(path.join(root, ".github"));
  return files.sort((left, right) => left.relativePath.localeCompare(right.relativePath));
}

export function workflowJobBody(workflow: string, jobName: string): string {
  const jobStart = workflow.indexOf(`\n  ${jobName}:\n`);
  assert.notEqual(jobStart, -1, `missing job ${jobName}`);
  const remaining = workflow.slice(jobStart + 1);
  const nextJobMatch = /\n  [a-z0-9-]+:\n/g.exec(remaining.slice(1));
  return remaining.slice(0, nextJobMatch ? nextJobMatch.index + 1 : undefined);
}

/**
 * A top-level job key's value, including any trailing comment. Scoping to the
 * owning job keeps assertions from being satisfied by a different job that
 * happens to carry the expected value.
 */
export function workflowJobField(
  workflow: string,
  jobName: string,
  field: string,
): string | undefined {
  const prefix = `    ${field}:`;
  const line = workflowJobBody(workflow, jobName)
    .split("\n")
    .find((candidate) => candidate.startsWith(prefix));
  return line?.slice(prefix.length).trim();
}

export function workflowJobRunsOn(workflow: string, jobName: string): string | undefined {
  return workflowJobField(workflow, jobName, "runs-on");
}

export function assertReleasePlatformCommandsUseRefEnv(
  workflow: string,
  jobNames: readonly string[],
): void {
  assert.doesNotMatch(
    workflow,
    /release-platforms\.rs (?:github-output|apply-cadence) "\$\{\{ github\.ref_name \}\}"/,
  );
  for (const jobName of jobNames) {
    const job = workflowJobBody(workflow, jobName);
    assert.match(job, /RELEASE_REF_NAME:\s*\$\{\{\s*github\.ref_name\s*\}\}/, jobName);
    assert.match(
      job,
      /release-platforms\.rs (?:github-output|apply-cadence) "\$RELEASE_REF_NAME"/,
      jobName,
    );
  }
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

// A runner label is a complete YAML scalar, so every pattern below ends at a
// label boundary. Without it the pattern only matches a prefix, and an
// assertion for `ubuntu-24.04` would be satisfied by `ubuntu-24.04-arm` (a
// different architecture) whenever the caller has no closing delimiter of its
// own.
const LABEL_BOUNDARY = "(?![\\w.-])";

// Runner pairs (hosted GitHub label, equivalent Blacksmith label) accepted by
// the workflow-shape tests. Letting either form match keeps the cross-platform
// coverage assertion strict on _which_ platforms are present without pinning the
// runner-pool vendor; switching back to GitHub-hosted shouldn't break the test
// and vice versa.
export function hostedOrBlacksmith(hostedLabel: string): string {
  if (hostedLabel === "ubuntu-22.04") {
    return `ubuntu-22\\.04${LABEL_BOUNDARY}`;
  }
  if (hostedLabel === "ubuntu-22.04-arm") {
    return `ubuntu-22\\.04-arm${LABEL_BOUNDARY}`;
  }
  if (hostedLabel === "ubuntu-24.04") {
    return `(?:ubuntu-24\\.04|blacksmith-\\d+vcpu-ubuntu-2404)${LABEL_BOUNDARY}`;
  }
  if (hostedLabel === "ubuntu-24.04-arm") {
    return `(?:ubuntu-24\\.04-arm|blacksmith-\\d+vcpu-ubuntu-2404-arm)${LABEL_BOUNDARY}`;
  }
  if (hostedLabel === "macos-15") {
    return `(?:macos-15|blacksmith-(?:6|12)vcpu-macos-15)${LABEL_BOUNDARY}`;
  }
  if (hostedLabel === "windows-2025") {
    return `(?:windows-2025|blacksmith-\\d+vcpu-windows-2025)${LABEL_BOUNDARY}`;
  }
  return `${escapeRegExp(hostedLabel)}${LABEL_BOUNDARY}`;
}

/** Match a full runner label, for callers that compare an extracted scalar. */
export function hostedOrBlacksmithExact(hostedLabel: string): RegExp {
  return new RegExp(`^${hostedOrBlacksmith(hostedLabel)}$`);
}
