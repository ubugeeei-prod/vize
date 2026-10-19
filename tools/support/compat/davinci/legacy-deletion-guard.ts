import { execFileSync } from "node:child_process";
import { pathToFileURL } from "node:url";

const legacyManifest =
  /^crates\/(?:vize_armature|vize_relief|vize_croquis(?:_cf)?|vize_atelier_[^/]+)\/Cargo\.toml$/;
const auditWorkflow = ".github/workflows/level-deletion-readiness.yml";
const shaPattern = /^[0-9a-f]{40}$/;

export function removedLegacyManifests(nameStatus) {
  const removed = [];
  for (const line of nameStatus.trim().split("\n")) {
    if (!line) continue;
    const [status, source] = line.split("\t");
    if ((status === "D" || status.startsWith("R")) && legacyManifest.test(source)) {
      removed.push(source);
    }
  }
  return removed;
}

export function hasExactSuccessfulAudit(response, sha) {
  return (
    Number.isSafeInteger(response?.total_count) &&
    Array.isArray(response.workflow_runs) &&
    response.workflow_runs.some(
      (run) =>
        run.head_sha === sha &&
        run.path === auditWorkflow &&
        run.event === "workflow_dispatch" &&
        run.conclusion === "success",
    )
  );
}

function command(binary, args) {
  return execFileSync(binary, args, { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
}

function ensureCommit(sha) {
  try {
    command("git", ["cat-file", "-e", `${sha}^{commit}`]);
  } catch {
    command("git", ["fetch", "--no-tags", "--depth=1", "origin", sha]);
  }
}

function successfulAuditAt(repository, sha) {
  const query =
    `repos/${repository}/actions/workflows/level-deletion-readiness.yml/runs` +
    `?event=workflow_dispatch&head_sha=${sha}&status=success&per_page=100`;
  const response = JSON.parse(command("gh", ["api", query]));
  return hasExactSuccessfulAudit(response, sha);
}

export function guardLegacyDeletion({ baseSha, candidateSha, repository }) {
  if (!shaPattern.test(baseSha) || !shaPattern.test(candidateSha)) {
    throw new Error("Missing exact base or candidate SHA for legacy deletion guard");
  }
  if (!/^[\w.-]+\/[\w.-]+$/.test(repository ?? "")) {
    throw new Error("Missing GitHub repository for legacy deletion guard");
  }
  ensureCommit(baseSha);
  ensureCommit(candidateSha);
  const changes = command("git", [
    "diff",
    "--name-status",
    "-M",
    baseSha,
    candidateSha,
    "--",
    "crates",
  ]);
  const removed = removedLegacyManifests(changes);
  if (removed.length === 0) {
    process.stdout.write("legacy deletion guard: no legacy crate manifest removed\n");
    return;
  }
  for (const sha of new Set([baseSha, candidateSha])) {
    if (!successfulAuditAt(repository, sha)) {
      throw new Error(
        `Legacy crate removal (${removed.join(", ")}) requires a successful ${auditWorkflow} ` +
          `workflow_dispatch audit at exact SHA ${sha}; missing or stale audit blocks deletion`,
      );
    }
  }
  process.stdout.write(
    `legacy deletion guard: exact base and candidate audits verified for ${removed.join(", ")}\n`,
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    guardLegacyDeletion({
      baseSha: process.env.BASE_SHA,
      candidateSha: process.env.CANDIDATE_SHA,
      repository: process.env.GITHUB_REPOSITORY,
    });
  } catch (error) {
    process.stderr.write(`legacy deletion blocked: ${error.message}\n`);
    process.exitCode = 1;
  }
}
