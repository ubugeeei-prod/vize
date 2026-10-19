import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const scope = "crates/vize_patina";
const objectId = /^(?:[0-9a-f]{40}|[0-9a-f]{64})$/;
const diffOptions = [
  "--raw",
  "--no-abbrev",
  "--no-ext-diff",
  "--no-textconv",
  "--no-renames",
  "--no-color",
  "-z",
];

interface SnapshotReference {
  readonly path: string;
  readonly blobBeforeCommit: string | null;
  readonly blobAtCommit: string | null;
  readonly blobAtRevision: string | null;
  readonly status: "candidate-only";
}

interface Delta {
  readonly parent: string | null;
  readonly scopedDiffSha256: string;
  readonly scopedDiffBytes: number;
  readonly touchedFiles: string[];
  readonly snapshotReferenceCandidates: SnapshotReference[];
}

interface CommitMetadata {
  readonly sha: string;
  readonly parents: string[];
  readonly subject: string;
}

export interface HistoryCommit extends CommitMetadata, Delta {
  readonly titleFixCandidate: boolean;
  readonly reviewState: "unreviewed";
}

export interface HistoryMerge extends CommitMetadata {
  readonly reviewState: "unreviewed";
  readonly parentDeltas: Delta[];
}

export interface LinterHistoryInventory {
  readonly schema: "vize.linterFixHistoryInventory";
  readonly version: 1;
  readonly revision: string;
  readonly scope: string;
  readonly diffProtocol: { readonly command: string; readonly options: readonly string[] };
  readonly historicalIssueCounts: Readonly<Record<string, number>>;
  readonly summary: Readonly<Record<string, number>>;
  readonly commits: readonly HistoryCommit[];
  readonly merges: readonly HistoryMerge[];
}

function gitBytes(cwd: string, args: readonly string[]): Buffer {
  return execFileSync("git", ["--no-replace-objects", "-c", "core.quotePath=true", ...args], {
    cwd,
    maxBuffer: 64 * 1024 * 1024,
    stdio: ["ignore", "pipe", "pipe"],
    env: { ...process.env, LC_ALL: "C", GIT_OPTIONAL_LOCKS: "0", GIT_DIFF_OPTS: "" },
  });
}

function git(cwd: string, args: readonly string[]): string {
  return gitBytes(cwd, args).toString("utf8");
}

function splitNull(value: string): string[] {
  return value.split("\0").filter((entry) => entry.length > 0);
}

function snapshots(
  cwd: string,
  revision: string,
  paths: readonly string[] = [scope],
): Map<string, string> {
  const result = new Map<string, string>();
  for (const entry of splitNull(git(cwd, ["ls-tree", "-rz", revision, "--", ...paths]))) {
    const match = /^\d+ blob ([0-9a-f]+)\t([\s\S]+)$/.exec(entry);
    const [, blob, file] = match ?? [];
    if (blob !== undefined && file?.endsWith(".snap")) result.set(file, blob);
  }
  return result;
}

function commitMetadata(cwd: string, revision: string): CommitMetadata {
  const [sha, parents, subject] = git(cwd, ["show", "-s", "--format=%H%x00%P%x00%s", revision])
    .trimEnd()
    .split("\0");
  if (sha === undefined || !objectId.test(sha) || subject === undefined)
    throw new Error("invalid Git commit metadata");
  return { sha, parents: parents ? parents.split(" ") : [], subject };
}

function references(
  cwd: string,
  sha: string,
  parent: string | null,
  paths: readonly string[],
  currentSnapshots: ReadonlyMap<string, string>,
): SnapshotReference[] {
  const changedSnapshots = paths.filter(
    (path) => path.startsWith(`${scope}/`) && path.endsWith(".snap"),
  );
  if (changedSnapshots.length === 0) return [];
  const historicalSnapshots = snapshots(cwd, sha, changedSnapshots);
  const previousSnapshots = parent
    ? snapshots(cwd, parent, changedSnapshots)
    : new Map<string, string>();
  return changedSnapshots.map((path) => ({
    path,
    blobBeforeCommit: previousSnapshots.get(path) ?? null,
    blobAtCommit: historicalSnapshots.get(path) ?? null,
    blobAtRevision: currentSnapshots.get(path) ?? null,
    status: "candidate-only" as const,
  }));
}

function delta(
  cwd: string,
  sha: string,
  parent: string | null,
  currentSnapshots: ReadonlyMap<string, string>,
): Delta {
  const command = parent
    ? ["diff", parent, sha]
    : ["diff-tree", "--root", "--no-commit-id", "-r", sha];
  const touchedFiles = splitNull(git(cwd, [...command, "--name-only", "--no-renames", "-z"]));
  // Raw tree deltas bind full old/new blob IDs and modes, including binary
  // changes, without relying on patch context or user-defined diff drivers.
  const diff = gitBytes(cwd, [...command, ...diffOptions, "--", scope]);
  return {
    parent,
    scopedDiffSha256: createHash("sha256").update(diff).digest("hex"),
    scopedDiffBytes: Buffer.byteLength(diff),
    touchedFiles,
    snapshotReferenceCandidates: references(cwd, sha, parent, touchedFiles, currentSnapshots),
  };
}

/** Read Git objects only: an inventory is never fixture coverage or native acceptance. */
export function collectLinterHistory({
  cwd = process.cwd(),
  revision = "HEAD",
}: { cwd?: string; revision?: unknown } = {}): LinterHistoryInventory {
  if (typeof revision !== "string" || revision.length === 0 || revision.startsWith("-")) {
    throw new Error("expected a commit revision");
  }
  if (git(cwd, ["rev-parse", "--is-shallow-repository"]).trim() !== "false") {
    throw new Error("full linter history requires a non-shallow repository");
  }
  const sha = git(cwd, [
    "rev-parse",
    "--verify",
    "--end-of-options",
    `${revision}^{commit}`,
  ]).trim();
  if (!objectId.test(sha)) throw new Error("revision must resolve to a full commit object ID");
  const currentSnapshots = snapshots(cwd, sha);
  const history = (mergeFlag: string): CommitMetadata[] =>
    splitNull(
      git(cwd, [
        "log",
        "--full-history",
        "--topo-order",
        mergeFlag,
        "--format=%H%x00",
        sha,
        "--",
        scope,
      ]),
    )
      .map((entry) => entry.trim())
      .filter(Boolean)
      .map((entry) => {
        if (!objectId.test(entry)) throw new Error("invalid history object ID");
        return commitMetadata(cwd, entry);
      });
  const commits: HistoryCommit[] = history("--no-merges").map((commit) => ({
    ...commit,
    titleFixCandidate: /^fix(?:\([^)]*\))?!?: /.test(commit.subject),
    reviewState: "unreviewed" as const,
    ...delta(cwd, commit.sha, commit.parents[0] ?? null, currentSnapshots),
  }));
  if (commits.length === 0) throw new Error(`revision has no ${scope} history`);
  const merges: HistoryMerge[] = history("--merges").map((commit) => ({
    ...commit,
    reviewState: "unreviewed" as const,
    // Per-parent diffs preserve merge topology. They do not identify conflict
    // resolutions or independent requirements without a human remerge review.
    parentDeltas: commit.parents.map((parent) => delta(cwd, commit.sha, parent, currentSnapshots)),
  }));
  return {
    schema: "vize.linterFixHistoryInventory" as const,
    version: 1 as const,
    revision: sha,
    scope,
    diffProtocol: {
      command: "git diff / diff-tree --root",
      options: diffOptions,
    },
    historicalIssueCounts: { touchCommits: 499, reportedFixes: 255, sourceIssue: 6881 },
    summary: {
      touchCommits: commits.length,
      titleFixCandidates: commits.filter((commit) => commit.titleFixCandidate).length,
      nonFixTitleCommits: commits.filter((commit) => !commit.titleFixCandidate).length,
      mergeSupplements: merges.length,
      unreviewedCommits: commits.length,
      unreviewedMerges: merges.length,
      acceptedCorpusCases: 0,
      executedTests: 0,
      nativePasses: 0,
    },
    commits,
    merges,
  };
}

export function inventoryTsv(inventory: LinterHistoryInventory): string {
  const columns: readonly (keyof HistoryCommit)[] = [
    "sha",
    "subject",
    "titleFixCandidate",
    "reviewState",
    "scopedDiffSha256",
    "touchedFiles",
    "snapshotReferenceCandidates",
  ];
  return `${columns.join("\t")}\n${inventory.commits.map((commit) => columns.map((column) => JSON.stringify(commit[column])).join("\t")).join("\n")}\n`;
}

export function writeInventory(inventory: LinterHistoryInventory, directory: string): void {
  mkdirSync(directory, { recursive: true });
  writeFileSync(resolve(directory, "history.json"), `${JSON.stringify(inventory, null, 2)}\n`);
  writeFileSync(resolve(directory, "history.tsv"), inventoryTsv(inventory));
  const rows = inventory.merges.map((commit) =>
    [commit.sha, commit.subject, commit.parents, commit.reviewState, commit.parentDeltas]
      .map((value) => JSON.stringify(value))
      .join("\t"),
  );
  writeFileSync(
    resolve(directory, "merges.tsv"),
    `sha\tsubject\tparents\treviewState\tparentDeltas\n${rows.join("\n")}\n`,
  );
}

export function main(args: readonly string[] = process.argv.slice(2)): void {
  if (args.length !== 2) throw new Error("usage: linter-fix-history.ts REVISION OUTPUT_DIRECTORY");
  const inventory = collectLinterHistory({ revision: args[0] });
  writeInventory(inventory, resolve(args[1] ?? ""));
  process.stdout.write(
    `${JSON.stringify({ revision: inventory.revision, ...inventory.summary })}\n`,
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) main();
