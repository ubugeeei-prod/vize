import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repositoryUrl = "https://github.com/ubugeeei-prod/vize";
const changelogHeader = `# Changelog

All notable changes to this repository are tracked in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Conventional Commits](https://www.conventionalcommits.org/).

See [\`docs/release/support-policy.md\`](./docs/release/support-policy.md) for the
deprecation contract that backs the entries below.
`;

const sectionOrder = [
  "Breaking",
  "Added",
  "Fixed",
  "Performance",
  "Reverted",
  "Documented release contract",
  "Released",
] as const;

export type ChangelogSection = (typeof sectionOrder)[number];

export interface ChangelogCommit {
  readonly id: string;
  readonly message: string;
  readonly section: ChangelogSection;
}

export interface ChangelogRelease {
  readonly version: string;
  readonly date?: string;
  readonly commits: readonly ChangelogCommit[];
}

export interface ChangelogOptions {
  readonly check: boolean;
  readonly latest: boolean;
  readonly output: string;
  readonly stdout: boolean;
  readonly tag?: string;
  readonly unreleased: boolean;
}

interface GitCommit {
  readonly body: string;
  readonly id: string;
  readonly message: string;
}

export function parseChangelogArgs(argv: readonly string[]): ChangelogOptions {
  let check = false;
  let latest = false;
  let output = "CHANGELOG.md";
  let stdout = false;
  let tag: string | undefined;
  let unreleased = false;

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--check") check = true;
    else if (arg === "--latest") latest = true;
    else if (arg === "--stdout") stdout = true;
    else if (arg === "--unreleased") unreleased = true;
    else if (arg === "--output" || arg === "--tag") {
      const value = argv[index + 1];
      if (value == null || value === "" || value.startsWith("--")) {
        throw new Error(`Missing value for ${arg}`);
      }
      if (arg === "--output") output = value;
      else tag = value;
      index += 1;
    } else throw new Error(`Unknown argument: ${arg}`);
  }

  if (tag != null && !/^v?\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(tag)) {
    throw new Error(`Invalid release tag: ${tag}`);
  }
  if (check && stdout) throw new Error("--check and --stdout cannot be combined");
  return { check, latest, output, stdout, tag, unreleased };
}

export function classifyCommit(
  id: string,
  subject: string,
  body = "",
): ChangelogCommit | undefined {
  const match =
    /^(?<type>[a-z][a-z0-9-]*)(?:\((?<scope>[^)]+)\))?(?<breaking>!)?:\s+(?<message>.+)$/i.exec(
      subject.trim(),
    );
  if (match?.groups == null) return undefined;

  const type = match.groups.type?.toLowerCase();
  const scope = match.groups.scope?.toLowerCase();
  const breaking = match.groups.breaking === "!" || /(?:^|\n)BREAKING[ -]CHANGE:\s*/i.test(body);
  let section: ChangelogSection | undefined;
  if (breaking) section = "Breaking";
  else if (type === "feat") section = "Added";
  else if (type === "fix") section = "Fixed";
  else if (type === "perf") section = "Performance";
  else if (type === "revert") section = "Reverted";
  else if (type === "docs" && scope === "release") section = "Documented release contract";
  else if (type === "chore" && scope === "release") section = "Released";
  if (section == null) return undefined;

  const message = match.groups.message?.trim();
  if (message == null || message === "") return undefined;
  return { id, message: `${message[0]?.toUpperCase() ?? ""}${message.slice(1)}`, section };
}

export function renderChangelog(releases: readonly ChangelogRelease[]): string {
  const output = [changelogHeader.trimEnd(), ""];
  for (const release of releases) {
    const version =
      release.version === "Unreleased" ? release.version : release.version.replace(/^v/, "");
    output.push(release.date == null ? `## [${version}]` : `## [${version}] - ${release.date}`, "");
    for (const section of sectionOrder) {
      const commits = release.commits.filter((commit) => commit.section === section);
      if (commits.length === 0) continue;
      output.push(`### ${section}`, "");
      for (const commit of commits) {
        output.push(
          `- ${commit.message} ([${commit.id.slice(0, 7)}](${repositoryUrl}/commit/${commit.id}))`,
        );
      }
      output.push("");
    }
  }
  return `${output.join("\n").trimEnd()}\n`;
}

function git(cwd: string, args: readonly string[]): string {
  return execFileSync("git", args, {
    cwd,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  }).trim();
}

function readCommits(cwd: string, range: string): readonly GitCommit[] {
  const raw = git(cwd, ["log", "--no-merges", "--format=%H%x1f%s%x1f%b%x1e", range]);
  if (raw === "") return [];
  return raw
    .split("\x1e")
    .map((record) => record.trim())
    .filter((record) => record !== "")
    .map((record) => {
      const [id = "", message = "", ...body] = record.split("\x1f");
      return { body: body.join("\x1f"), id, message };
    });
}

function releaseFromRange(
  cwd: string,
  version: string,
  range: string,
  date?: string,
): ChangelogRelease {
  const commits = readCommits(cwd, range)
    .map((commit) => classifyCommit(commit.id, commit.message, commit.body))
    .filter((commit): commit is ChangelogCommit => commit != null);
  return { commits, date, version };
}

export function collectChangelog(
  cwd: string,
  options: ChangelogOptions,
): readonly ChangelogRelease[] {
  const tags = git(cwd, ["tag", "--list", "v[0-9]*", "--sort=-v:refname"])
    .split("\n")
    .filter((tag) => tag !== "");
  const latestTag = tags[0];
  const releases: ChangelogRelease[] = [];
  const pendingRange = latestTag == null ? "HEAD" : `${latestTag}..HEAD`;

  if (options.tag != null) {
    releases.push(
      releaseFromRange(cwd, options.tag, pendingRange, new Date().toISOString().slice(0, 10)),
    );
  } else if (options.unreleased || !options.latest) {
    const pending = releaseFromRange(cwd, "Unreleased", pendingRange);
    if (pending.commits.length > 0 || options.unreleased) releases.push(pending);
  }

  const selectedTags = options.latest ? tags.slice(0, 1) : tags;
  for (const tag of selectedTags) {
    const index = tags.indexOf(tag);
    const olderTag = tags[index + 1];
    const range = olderTag == null ? tag : `${olderTag}..${tag}`;
    const date = git(cwd, ["log", "-1", "--format=%cs", tag]);
    releases.push(releaseFromRange(cwd, tag, range, date));
  }
  return releases;
}

export function regenerateChangelog(cwd: string, options: ChangelogOptions): string {
  const markdown = renderChangelog(collectChangelog(cwd, options));
  const outputPath = path.resolve(cwd, options.output);
  if (options.stdout) process.stdout.write(markdown);
  else if (options.check) {
    if (readFileSync(outputPath, "utf8") !== markdown) {
      throw new Error(`${options.output} is stale; regenerate it without --check`);
    }
  } else writeFileSync(outputPath, markdown);
  return markdown;
}

const scriptPath = fileURLToPath(import.meta.url);
if (process.argv[1] != null && path.resolve(process.argv[1]) === scriptPath) {
  try {
    const root = path.resolve(path.dirname(scriptPath), "../../..");
    regenerateChangelog(root, parseChangelogArgs(process.argv.slice(2)));
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  }
}
