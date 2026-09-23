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
 * What "the drift does not affect the release" means.
 *
 * The preflight waits 30-40 minutes for eight workflows to report success at
 * the exact release SHA, and the repository's merge automation lands PRs
 * throughout that window. #3540 stopped demanding an exact `main` tip and
 * settled for first-parent ancestry, which removed the failure mode but left
 * the acceptance rule saying nothing: any commit ever on `main` was accepted.
 *
 * Ordinary drift genuinely cannot affect a release. Everything the release
 * publishes is built from the tagged tree; the tag is immutable and verified to
 * still resolve to `sha`; every required gate is evaluated at `sha`, not at
 * `main`; and release-blocking issues are evaluated live. A fix merged after
 * the tag simply ships in the next version.
 *
 * One kind of drift is different: another release. A second `chore: release`
 * commit means `main` has moved the version line on, and finishing the older
 * release then publishes a lower version *after* a higher one -- npm's `latest`
 * dist-tag, the crates.io release order and the "latest" GitHub Release all end
 * up describing the superseded build. v0.316.0 and v0.317.0 were tagged seven
 * minutes apart and were in flight together, so this is the live case, not a
 * hypothetical.
 *
 * The workspace version at `main`'s tip is what makes that decidable: only the
 * release tool writes it, and it is exactly "which release owns `main` now".
 * Equal means the drift is ordinary work and the release still owns its
 * version; different means a newer release has taken over and this one must not
 * publish. Checking it before the gate wait also stops the superseded release
 * in seconds instead of after 35 minutes of CI.
 */
export function assertReleaseVersionStillOwnsMain({
  tag,
  sha,
  mainSha,
  releaseVersion,
  mainVersion,
}) {
  if (mainVersion === releaseVersion) return;
  throw new Error(
    `Release ${tag} (${sha}) is superseded: origin/main ${mainSha} is at workspace version ${mainVersion}, not ${releaseVersion}. Publishing it now would ship an older version after a newer one; cut the next release instead.`,
  );
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

/**
 * Whether a release commit changed nothing but version metadata.
 *
 * The release script writes the same version into 27 tracked files and
 * regenerates the two lockfiles; it never touches source. Yet the preflight
 * used to re-dispatch every gate at the tag SHA, so a commit of the shape
 * `26 files changed, 88 insertions(+), 88 deletions(-)` paid for a fresh Real
 * Project Matrix — measured at 123, 125, 126, 303 and 317 minutes across recent
 * runs, against 8-11 minutes for Check. That is the entire release wait, spent
 * re-proving code that did not change.
 *
 * When this returns true the caller may accept the release commit's first
 * parent as evidence for the gates that prove the *code*. Gates that prove the
 * *artifacts* — Native Smoke installs what the tag builds — must still run at
 * the tag, so the decision is per workflow rather than global.
 *
 * The allowlist is paths, not diff content, which is safe only because the
 * caller pairs it with the checks that already exist: the commit must be the
 * single-parent release commit on main's first-parent chain, its tag must point
 * at it, and `assertReleaseMetadata` must agree the version it carries is the
 * tag's. A hand-written commit touching `Cargo.toml` cannot reach here without
 * also faking all of those.
 */
const versionMetadataBasenames = new Set([
  "Cargo.lock",
  "Cargo.toml",
  "CHANGELOG.md",
  "extension.toml",
  "package.json",
  "pnpm-lock.yaml",
  "pnpm-workspace.yaml",
]);

export function isVersionMetadataOnlyRelease(changedPaths) {
  if (!Array.isArray(changedPaths) || changedPaths.length === 0) return false;
  return changedPaths.every((changedPath) => {
    if (typeof changedPath !== "string" || changedPath.length === 0) return false;
    const basename = changedPath.split("/").pop() ?? "";
    if (versionMetadataBasenames.has(basename)) return true;
    return basename.startsWith("README") && basename.endsWith(".md");
  });
}
