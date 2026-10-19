// Version parsing and synchronized manifest updates for release candidates.
import { run } from "../command_io.ts";
import { moonTrim, parseMoonInt, replaceEvery, replaceFirst } from "../command_runtime.ts";

/**
 * Parsed semantic version components used by the release bump logic.
 * The record stores only the fields the repository needs today: major / minor /
 * patch plus an optional prerelease label and numeric suffix.
 */
interface Version {
  major: number;
  minor: number;
  patch: number;
  prerelease: string | null;
  prereleaseNum: number | null;
}

export async function isGitIgnored(target: string): Promise<boolean> {
  return (await run("git", ["check-ignore", "-q", target])) === 0;
}

/**
 * Parses the repository's release version format into structured fields.
 * Supported inputs are:
 * - `major.minor.patch`
 * - `major.minor.patch-label`
 * - `major.minor.patch-label.number`
 * The parser is intentionally strict so release automation never guesses when
 * the current version format has drifted unexpectedly.
 */
function parseVersion(value: string): Version | null {
  const parts = value.split("-");
  if (parts.length < 1 || parts.length > 2) return null;
  const baseParts = parts[0].split(".");
  if (baseParts.length !== 3) return null;
  const major = parseMoonInt(baseParts[0]);
  if (major === null) return null;
  const minor = parseMoonInt(baseParts[1]);
  if (minor === null) return null;
  const patch = parseMoonInt(baseParts[2]);
  if (patch === null) return null;

  let prerelease: string | null = null;
  let prereleaseNum: number | null = null;
  if (parts.length === 2) {
    const prereleaseParts = parts[1].split(".");
    if (prereleaseParts.length < 1 || prereleaseParts.length > 2) return null;
    if (prereleaseParts[0] === "") return null;
    prerelease = prereleaseParts[0];
    if (prereleaseParts.length === 2) {
      prereleaseNum = parseMoonInt(prereleaseParts[1]);
      if (prereleaseNum === null) return null;
    }
  }
  return { major, minor, patch, prerelease, prereleaseNum };
}

/** Formats a structured version back into the repository's release string form. */
function formatVersion(
  major: number,
  minor: number,
  patch: number,
  prerelease: string | null,
  prereleaseNum: number | null,
): string {
  const base = `${major}.${minor}.${patch}`;
  if (prerelease === null) return base;
  return prereleaseNum === null
    ? `${base}-${prerelease}`
    : `${base}-${prerelease}.${prereleaseNum}`;
}

/** Adds one with 32-bit wrapping, matching the original integer arithmetic. */
const increment = (value: number): number => (value + 1) | 0;

/**
 * Computes the next release version for a given bump strategy.
 * Stable bumps (`patch`, `minor`, `major`) reset the numeric suffix, while
 * prerelease bumps preserve the existing base version and increment the suffix
 * only when the current version already uses the same prerelease channel.
 */
export function bumpVersion(current: string, bump: string): string | null {
  const version = parseVersion(current);
  if (version === null) return null;
  switch (bump) {
    case "patch":
      return formatVersion(version.major, version.minor, increment(version.patch), null, null);
    case "minor":
      return formatVersion(version.major, increment(version.minor), 0, null, null);
    case "major":
      return formatVersion(increment(version.major), 0, 0, null, null);
    case "alpha":
      return version.prerelease === "alpha"
        ? formatVersion(
            version.major,
            version.minor,
            version.patch,
            "alpha",
            increment(version.prereleaseNum ?? 1),
          )
        : formatVersion(version.major, version.minor, version.patch, "alpha", null);
    case "beta":
      return formatVersion(version.major, version.minor, version.patch, "beta", null);
    case "rc":
      return formatVersion(version.major, version.minor, version.patch, "rc", null);
    case "release":
      return formatVersion(version.major, version.minor, version.patch, null, null);
    default:
      return null;
  }
}

/** Extracts the current workspace version from the root `Cargo.toml`. */
export function workspaceVersionFromContent(content: string): string | null {
  let section = "";
  for (const line of content.split("\n")) {
    const trimmed = moonTrim(line);
    if (trimmed.startsWith("[") && trimmed.endsWith("]")) {
      section = trimmed;
      continue;
    }
    if (
      section === "[workspace.package]" &&
      trimmed.startsWith('version = "') &&
      trimmed.endsWith('"')
    ) {
      return replaceEvery(replaceFirst(trimmed, 'version = "', ""), '"', "");
    }
  }
  return null;
}

/**
 * Rewrites all workspace-level Rust version pins that must stay aligned.
 * This updates both `[workspace.package].version` and the exact-version
 * `vize_*` dependencies inside `[workspace.dependencies]`.
 */
export function updateWorkspaceManifestVersions(
  content: string,
  currentVersion: string,
  newVersion: string,
): string {
  let section = "";
  const updated: string[] = [];
  for (const line of content.split("\n")) {
    const trimmed = moonTrim(line);
    if (trimmed.startsWith("[") && trimmed.endsWith("]")) {
      section = trimmed;
      updated.push(line);
      continue;
    }
    if (section === "[workspace.package]" && trimmed === `version = "${currentVersion}"`) {
      updated.push(replaceFirst(line, currentVersion, newVersion));
      continue;
    }
    if (
      section === "[workspace.dependencies]" &&
      (trimmed.startsWith("vize_") || line.includes('package = "vize_')) &&
      line.includes(`version = "=${currentVersion}"`)
    ) {
      updated.push(
        replaceEvery(line, `version = "=${currentVersion}"`, `version = "=${newVersion}"`),
      );
      continue;
    }
    updated.push(line);
  }
  return updated.join("\n");
}

/** Orders stable `major.minor.patch` versions numerically. */
function compareStable(left: Version, right: Version): number {
  return left.major - right.major || left.minor - right.minor || left.patch - right.patch;
}

function parseClaimed(value: string): Version | null {
  const version = parseVersion(value.startsWith("v") ? value.slice(1) : value);
  return version === null || version.prerelease !== null ? null : version;
}

/**
 * Every release bumps the minor of the highest version already claimed
 * anywhere (workspace manifest, git tags, npm), so a burned or partial release
 * never collides with the next one.
 */
export function nextReleaseVersion(claimed: readonly string[]): string | null {
  let highest: Version | null = null;
  for (const value of claimed) {
    const version = parseClaimed(value);
    if (version !== null && (highest === null || compareStable(version, highest) > 0)) {
      highest = version;
    }
  }
  return highest === null
    ? null
    : formatVersion(highest.major, increment(highest.minor), 0, null, null);
}

/** True when `candidate` is a stable version above every claimed one. */
export function isUnclaimedVersion(candidate: string, claimed: readonly string[]): boolean {
  const version = parseClaimed(candidate);
  if (version === null || candidate.startsWith("v")) return false;
  return claimed.every((value) => {
    const other = parseClaimed(value);
    return other === null || compareStable(version, other) > 0;
  });
}
