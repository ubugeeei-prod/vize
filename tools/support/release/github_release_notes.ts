// Pinned release tags are sibling commits, so GitHub's ancestry-based default
// can select an old release. Select the preceding published version explicitly.
import { execFileSync } from "node:child_process";
import { closeSync, mkdtempSync, openSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export interface PublishedRelease {
  tag_name: string;
  draft: boolean;
  prerelease: boolean;
  published_at: string | null;
}

interface Version {
  core: bigint[];
  prerelease: string[];
}

function version(tag: string): Version {
  const match =
    /^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([\da-zA-Z-]+(?:\.[\da-zA-Z-]+)*))?(?:\+[\da-zA-Z-]+(?:\.[\da-zA-Z-]+)*)?$/.exec(
      tag,
    );
  if (!match) throw new Error(`Invalid release version: ${tag}`);
  const prerelease = match[4]?.split(".") ?? [];
  if (prerelease.some((part) => /^0\d+$/.test(part))) {
    throw new Error(`Invalid numeric prerelease version: ${tag}`);
  }
  return { core: match.slice(1, 4).map((part) => BigInt(part)), prerelease };
}

function compare(left: Version, right: Version): number {
  for (let index = 0; index < 3; index++) {
    if (left.core[index] !== right.core[index]) {
      return left.core[index] < right.core[index] ? -1 : 1;
    }
  }
  if (!left.prerelease.length || !right.prerelease.length) {
    return Number(!left.prerelease.length) - Number(!right.prerelease.length);
  }
  for (let index = 0; index < Math.max(left.prerelease.length, right.prerelease.length); index++) {
    const a = left.prerelease[index];
    const b = right.prerelease[index];
    if (a === b) continue;
    if (a === undefined || b === undefined) return a === undefined ? -1 : 1;
    const numericA = /^\d+$/.test(a);
    const numericB = /^\d+$/.test(b);
    if (numericA && numericB) return BigInt(a) < BigInt(b) ? -1 : 1;
    if (numericA !== numericB) return numericA ? -1 : 1;
    return a < b ? -1 : 1;
  }
  return 0;
}

/** Publication order and Git ancestry do not define a release's version range. */
export function previousPublishedTag(
  currentTag: string,
  releases: PublishedRelease[],
): string | null {
  const current = version(currentTag);
  let previous: { tag: string; version: Version } | undefined;
  for (const release of releases) {
    if (release.draft || !release.published_at || release.tag_name === currentTag) continue;
    if (current.prerelease.length === 0 && release.prerelease) continue;
    // Non-versioned releases (for example standalone assets) are not predecessors.
    if (!/^v\d+\.\d+\.\d+(?:[-+]|$)/.test(release.tag_name)) continue;
    const candidate = version(release.tag_name);
    if (current.prerelease.length === 0 && candidate.prerelease.length > 0) continue;
    if (compare(candidate, current) >= 0) continue;
    if (!previous || compare(candidate, previous.version) > 0) {
      previous = { tag: release.tag_name, version: candidate };
    }
  }
  return previous?.tag ?? null;
}

type GitHubApi = (path: string, body?: object) => unknown;

/** Enumerate every page: high release frequency can put the predecessor past page one. */
export function generateReleaseNotes(
  api: GitHubApi,
  repository: string,
  tag: string,
  expectedSha: string,
): { body: string; previousTag: string } {
  if (!/^[\w.-]+\/[\w.-]+$/.test(repository)) throw new Error("Invalid release repository");
  if (!/^[a-f0-9]{40}$/.test(expectedSha))
    throw new Error("Release notes require the exact source SHA");
  version(tag);
  const releases: PublishedRelease[] = [];
  for (let page = 1; ; page++) {
    const batch = api(`repos/${repository}/releases?per_page=100&page=${page}`);
    if (!Array.isArray(batch)) throw new Error("Invalid GitHub releases response");
    releases.push(...batch);
    if (batch.length < 100) break;
  }
  const previousTag = previousPublishedTag(tag, releases);
  // The API infers a predecessor when this field is omitted, including old
  // unpublished tags. A first release needs an explicitly authored bootstrap
  // body; this existing-release workflow must never guess its history range.
  if (!previousTag) throw new Error(`No preceding published version found for ${tag}`);
  const response = api(`repos/${repository}/releases/generate-notes`, {
    tag_name: tag,
    target_commitish: expectedSha,
    previous_tag_name: previousTag,
  });
  if (
    !response ||
    typeof response !== "object" ||
    !("body" in response) ||
    typeof response.body !== "string"
  ) {
    throw new Error("Invalid generated release notes response");
  }
  return { body: response.body, previousTag };
}

function main(): void {
  const [tag, expectedSha, output] = process.argv.slice(2);
  if (!tag || !expectedSha || !output)
    throw new Error("Usage: github_release_notes.ts TAG SHA OUTPUT");
  const actualSha = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  if (actualSha !== expectedSha)
    throw new Error("Release notes checkout differs from the pinned source");
  const repository = process.env.GITHUB_REPOSITORY ?? "";
  const api: GitHubApi = (path, body) => {
    const directory = mkdtempSync(join(tmpdir(), "vize-release-notes-api-"));
    const output = join(directory, "response.json");
    const fd = openSync(output, "wx", 0o600);
    try {
      // Release-list bodies/assets are unrelated to predecessor selection and
      // can exceed a megabyte on one page. Project metadata in gh, then capture
      // through a file descriptor so the generated notes also remain complete.
      const args = body
        ? ["--method", "POST", "--input", "-"]
        : ["--jq", "map({tag_name, draft, prerelease, published_at})"];
      execFileSync("gh", ["api", path, ...args], {
        input: body ? JSON.stringify(body) : undefined,
        stdio: ["pipe", fd, "inherit"],
      });
      return JSON.parse(readFileSync(output, "utf8"));
    } finally {
      closeSync(fd);
      rmSync(directory, { recursive: true, force: true });
    }
  };
  const result = generateReleaseNotes(api, repository, tag, expectedSha);
  const scope = readFileSync(
    new URL("../../../.github/release-notes/drop-in-scope.md", import.meta.url),
    "utf8",
  );
  writeFileSync(output, `${scope.trim()}\n\n${result.body.trim()}\n`);
  process.stdout.write(`Release notes: ${result.previousTag}...${tag} @ ${expectedSha}\n`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
