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

// The published tag receipt is the existing release identity, not a PR title or
// label. Historical receipts outside this exact normal-cut format are left alone.
function publishedIntegration(api: GitHubApi, repository: string, tag: string): string | null {
  try {
    const prefix = `repos/${repository}`;
    const reference = api(`${prefix}/git/ref/tags/${tag}`) as {
      ref: string;
      object: { type: string; sha: string };
    };
    if (
      reference.ref !== `refs/tags/${tag}` ||
      reference.object.type !== "tag" ||
      !/^[a-f0-9]{40}$/.test(reference.object.sha)
    )
      return null;
    const annotated = api(`${prefix}/git/tags/${reference.object.sha}`) as {
      sha: string;
      tag: string;
      object: { type: string; sha: string };
      message: string;
    };
    if (
      annotated.sha !== reference.object.sha ||
      annotated.tag !== tag ||
      annotated.object.type !== "commit"
    )
      return null;
    const escaped = tag.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const sha = "([a-f0-9]{40})";
    const match = new RegExp(
      `^Release ${escaped}\\n\\nRelease-Mode: pinned\\nRelease-PR: #([1-9]\\d*)` +
        `\\nValidated-run: ([1-9]\\d*)\\nValidated-head: ${sha}\\nValidated-base: ${sha}` +
        `\\nIntegration-PR: #([1-9]\\d*)\\nIntegration-head: ${sha}\\nIntegration-merge: ${sha}` +
        `\\nIntegration-parent: ${sha}\\nProtected-candidate: ${sha}$`,
    ).exec(annotated.message.trimEnd());
    if (!match) return null;
    const [, source, , head, cut, integration, integrationHead, merge, parent] = match;
    if (annotated.object.sha !== head || parent !== cut) return null;
    const pr = api(`${prefix}/pulls/${integration}`) as {
      number: number;
      merged: boolean;
      state: string;
      merged_at: string | null;
      merge_commit_sha: string;
      body: string;
      base: { ref: string; repo: { full_name: string } };
      head: { sha: string; repo: { full_name: string } };
    };
    if (
      String(pr.number) !== integration ||
      pr.merged !== true ||
      pr.state !== "closed" ||
      !pr.merged_at ||
      pr.merge_commit_sha !== merge ||
      pr.base.ref !== "main" ||
      pr.base.repo.full_name !== repository ||
      pr.head.repo.full_name !== repository ||
      pr.head.sha !== integrationHead
    )
      return null;
    for (const [key, value] of Object.entries({ source, cut, head, tag })) {
      const marker = `<!-- vize-release-pin-${key}: `;
      const lines = pr.body.split("\n").filter((line) => line.startsWith(marker));
      if (lines.length !== 1 || lines[0] !== `${marker}${value} -->`) return null;
    }
    interface Commit {
      sha: string;
      parents: { sha: string }[];
      commit: { tree: { sha: string }; verification: { verified: boolean; reason: string } };
    }
    const h = api(`${prefix}/commits/${head}`) as Commit;
    const m = api(`${prefix}/commits/${merge}`) as Commit;
    if (
      h.sha !== head ||
      m.sha !== merge ||
      h.parents.length !== 1 ||
      h.parents[0].sha !== cut ||
      m.parents.length !== 1 ||
      m.parents[0].sha !== cut ||
      m.commit.verification.verified !== true ||
      m.commit.verification.reason !== "valid" ||
      !/^[a-f0-9]{40}$/.test(h.commit.tree.sha) ||
      h.commit.tree.sha !== m.commit.tree.sha
    )
      return null;
    // Whole-tree equality proves all of I's content was already published in H.
    // It does not reclassify the delta as version-only or qualify shipping gates.
    return `https://github.com/${repository}/pull/${integration}`;
  } catch {
    // Cosmetic cleanup must not suppress rows when old identity is unavailable,
    // malformed, or from a protocol outside this conservative historical scope.
    return null;
  }
}

function omitPublishedIntegration(body: string, url: string | null): string {
  if (!url || body.split(url).length !== 2) return body;
  const escaped = url.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  // Recognize exactly one complete GitHub-generated PR bullet. Unknown layouts,
  // duplicates, URL suffixes, and other mentions retain the entire original body.
  return body.replace(
    new RegExp(`^\\* [^\\r\\n]+ by @[\\w-]+ in ${escaped}(?:\\r?\\n|$)`, "m"),
    "",
  );
}

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
  const integration = response.body.includes(`https://github.com/${repository}/pull/`)
    ? publishedIntegration(api, repository, previousTag)
    : null;
  return { body: omitPublishedIntegration(response.body, integration), previousTag };
}

export interface ReleaseCommit {
  message: string;
  /** GitHub login of the commit author, when GitHub matched one. */
  author: string | null;
}

/**
 * Credits the people who co-authored a fix without authoring its commit:
 * third-party reporters are added as `Co-authored-by` on their fix, but the
 * generated notes only name each PR's author.
 */
export function coAuthorCredits(commits: readonly ReleaseCommit[]): string {
  const credits = new Map<string, Set<string>>();
  for (const { message, author } of commits) {
    const pr = /\(#(\d+)\)$/m.exec(message.split("\n")[0] ?? "")?.[1];
    for (const [, name, email] of message.matchAll(/^Co-authored-by:\s*(.+?)\s*<([^>]+)>\s*$/gim)) {
      const login = /^(?:\d+\+)?([\w-]+)@users\.noreply\.github\.com$/i.exec(email)?.[1];
      if (login?.toLowerCase() === author?.toLowerCase()) continue;
      if (/\[bot\]|@anthropic\.com$/i.test(`${login ?? name} ${email}`)) continue;
      const who = login ? `@${login}` : name;
      const prs = credits.get(who) ?? new Set<string>();
      if (pr) prs.add(`#${pr}`);
      credits.set(who, prs);
    }
  }
  if (credits.size === 0) return "";
  const rows = [...credits].map(([who, prs]) =>
    prs.size ? `${who} (${[...prs].join(", ")})` : who,
  );
  return `## Thanks\n\nReported and co-authored by ${rows.join(", ")}.\n`;
}

function compareCommits(api: GitHubApi, repository: string, base: string, head: string) {
  const commits: ReleaseCommit[] = [];
  for (let page = 1; ; page++) {
    const response = api(
      `repos/${repository}/compare/${base}...${head}?per_page=100&page=${page}`,
    ) as {
      commits?: { commit: { message: string }; author: { login: string } | null }[];
    };
    const batch = response.commits ?? [];
    for (const entry of batch) {
      commits.push({ message: entry.commit.message, author: entry.author?.login ?? null });
    }
    if (batch.length < 100) return commits;
  }
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
        : path.includes("/releases?")
          ? ["--jq", "map({tag_name, draft, prerelease, published_at})"]
          : path.includes("/compare/")
            ? ["--jq", "{commits: [.commits[] | {commit: {message: .commit.message}, author}]}"]
            : [];
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
  let credits = "";
  try {
    credits = coAuthorCredits(compareCommits(api, repository, result.previousTag, expectedSha));
  } catch (error) {
    // Credits are cosmetic: never block the GitHub Release on them.
    process.stdout.write(`::warning::Skipped co-author credits: ${String(error)}\n`);
  }
  const sections = [scope.trim(), result.body.trim(), credits.trim()].filter(Boolean);
  writeFileSync(output, `${sections.join("\n\n")}\n`);
  process.stdout.write(`Release notes: ${result.previousTag}...${tag} @ ${expectedSha}\n`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
