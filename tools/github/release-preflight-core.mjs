import { parseReleaseVersion } from "./release-platforms.mjs";

const releaseBlockingLabels = new Set(["priority:p0", "priority:p1"]);

export function workspaceVersionFromCargoToml(content) {
  let inWorkspacePackage = false;
  for (const line of content.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (trimmed.startsWith("[") && trimmed.endsWith("]")) {
      inWorkspacePackage = trimmed === "[workspace.package]";
      continue;
    }
    if (!inWorkspacePackage) continue;
    const match = /^version\s*=\s*"([^"]+)"$/.exec(trimmed);
    if (match) return match[1];
  }
  throw new Error("Cargo.toml is missing [workspace.package].version");
}

export function assertReleaseMetadata({ tag, sha, cargoToml, packageManifests }) {
  if (!/^[0-9a-f]{40}$/.test(sha)) {
    throw new Error(`Release SHA must be a full commit SHA, got ${sha}`);
  }
  parseReleaseVersion(tag);
  const version = workspaceVersionFromCargoToml(cargoToml);
  if (tag !== `v${version}`) {
    throw new Error(`Release tag ${tag} does not match workspace version ${version}`);
  }

  const mismatches = [];
  for (const manifest of packageManifests) {
    let packageJson;
    try {
      packageJson = JSON.parse(manifest.content);
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      throw new Error(`Failed to parse release package manifest ${manifest.path}: ${detail}`, {
        cause: error,
      });
    }
    if (packageJson.private === true) {
      mismatches.push(`${manifest.path} is private`);
      continue;
    }
    if (packageJson.version !== version) {
      mismatches.push(`${manifest.path}=${String(packageJson.version)}`);
    }
  }
  if (mismatches.length > 0) {
    throw new Error(
      `Release package versions must all equal ${version}:\n${mismatches
        .map((value) => `- ${value}`)
        .join("\n")}`,
    );
  }
  return version;
}

export function assertReleaseCommitIsOnMainFirstParent(sha, mainSha, isOnFirstParent) {
  if (!isOnFirstParent) {
    throw new Error(
      `Release commit ${sha} is not on the first-parent history of current origin/main ${mainSha}`,
    );
  }
}

/**
 * Issues that must be closed before `tag` may ship.
 *
 * A `fix(fuzz):` reproducer blocks every release: it is a live defect, not a
 * planning marker.
 *
 * The readiness labels block v1 and later only, because that is what they say
 * they are — `priority:p0` is described as "Release blocker for **v1 alpha**
 * production readiness" and `priority:p1` as "High priority before **v1
 * alpha** production readiness". Applying them to `0.x` made every pre-1.0
 * release wait on multi-release campaign umbrellas that are not meant to close
 * before v1, which left the project unable to ship at all. Omitting `tag`
 * keeps the strict behaviour.
 */
export function findReleaseBlockers(issues, tag) {
  const readinessLabelsBlock = tag == null || parseReleaseVersion(tag).major >= 1;
  return issues.filter((issue) => {
    if (issue.pull_request != null) return false;
    if (/^fix\(fuzz\):/i.test(issue.title ?? "")) return true;
    if (!readinessLabelsBlock) return false;
    const labels = (issue.labels ?? []).map((label) =>
      (typeof label === "string" ? label : (label.name ?? "")).toLowerCase(),
    );
    return labels.some((label) => releaseBlockingLabels.has(label));
  });
}

export function remoteTagCommit(output, tag) {
  const refs = new Map(
    output
      .trim()
      .split(/\r?\n/)
      .filter(Boolean)
      .map((line) => line.trim().split(/\s+/, 2).reverse()),
  );
  return refs.get(`refs/tags/${tag}^{}`) ?? refs.get(`refs/tags/${tag}`);
}
