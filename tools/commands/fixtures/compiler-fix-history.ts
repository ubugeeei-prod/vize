import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

// This is an audit index, never a denominator or an execution certificate.
export const HISTORY_REVISION = "9aaa1fe458a09e0d0c6604dc8835ccf7c737d943";
export const HISTORY_CRATES = [
  "vize_atelier_core",
  "vize_atelier_dom",
  "vize_atelier_ssr",
  "vize_atelier_vapor",
  "vize_atelier_sfc",
  "vize_atelier_jsx",
  "vize_armature",
  "vize_croquis",
];
const INDEX = "docs/davinci/plan/compiler-fix-history.tsv";
const RECEIPT = "docs/davinci/plan/compiler-fix-history-index.json";
const FIX = /^fix(?:\([^)]*\))?!?:/;
const digest = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");

function git(root: string, args: string[], input?: string): string {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    input,
    maxBuffer: 32 * 1024 * 1024,
  });
}

type CaseReference = { id: string; evidence: string };
const MERGE_ORIGINS = new Map([
  ["0de7787aeac48a931e15bc590a3e3b803286dcb8", "14a4ff375bc1a31dac829da5c650347353130331"],
]);

// Only explicit provenance links count. A commit subject or changed snapshot
// does not establish that its original input or complete target bytes survive.
function fixtureReferences(root: string): Map<string, CaseReference[]> {
  const links = new Map<string, CaseReference[]>();
  const packs = [
    "vize_atelier_sfc/tests/fixtures/fix-history/provenance.json",
    "vize_atelier_sfc/tests/fixtures/fix-history/diagnostic-provenance.json",
    "vize_atelier_sfc/tests/fixtures/fix-history/map-diagnostic-input-provenance.json",
    "vize_atelier_sfc/tests/fixtures/fix-history/next-provenance.json",
    "vize_atelier_ssr/tests/fixtures/fix-history/input-provenance.json",
    "vize_atelier_vapor/tests/fixtures/fix-history/input-provenance.json",
  ];
  for (const pack of packs) {
    const evidence = `crates/${pack}`;
    if (!fs.existsSync(path.join(root, evidence))) continue;
    const document = JSON.parse(fs.readFileSync(path.join(root, evidence), "utf8"));
    assert(Array.isArray(document.cases), `missing provenance cases: ${evidence}`);
    for (const fixture of document.cases) {
      const fixes: string[] = fixture.fixCommits ?? [fixture.fixCommit];
      assert.equal(typeof fixture.id, "string");
      for (const fix of fixes) {
        assert.match(fix, /^[a-f0-9]{40}$/);
        const cases = links.get(fix) ?? [];
        cases.push({ id: fixture.id, evidence });
        links.set(fix, cases);
      }
    }
  }
  return links;
}

export function collectHistory(root: string) {
  assert.equal(
    git(root, ["rev-parse", "--is-shallow-repository"]).trim(),
    "false",
    "a complete local Git graph is required; no network fetch is implicit",
  );
  const scopes = HISTORY_CRATES.map((name) => `crates/${name}`);
  const args = [
    "log",
    "--full-history",
    "--format=%H%x09%P%x09%s",
    HISTORY_REVISION,
    "--",
    ...scopes,
  ];
  const raw = git(root, args);
  const commits = raw
    .trimEnd()
    .split("\n")
    .map((line) => {
      const [sha, parents, ...subject] = line.split("\t");
      return { sha, parents: parents.split(" ").filter(Boolean), subject: subject.join("\t") };
    });
  const fixes = commits.filter((commit) => FIX.test(commit.subject));
  const nonMergeFixes = fixes.filter((commit) => commit.parents.length < 2);
  const links = fixtureReferences(root);
  const rows = nonMergeFixes.map((commit) => {
    const paths = git(root, ["show", "--format=", "--name-only", commit.sha, "--", ...scopes])
      .trimEnd()
      .split("\n")
      .filter(Boolean);
    const patch = git(root, ["show", "--format=", "--no-ext-diff", commit.sha, "--", ...scopes]);
    const patchId = git(root, ["patch-id", "--stable"], patch).trim().split(" ")[0] || null;
    const references = links.get(commit.sha) ?? [];
    return {
      ...commit,
      paths,
      patchId,
      witnessCandidates: paths.filter((file) =>
        /(?:\/tests(?:\/|\.rs)|_tests\.rs$|\.snap$)/.test(file),
      ),
      references,
      review: references.length
        ? "fixture-linked;target-and-semantic-review-pending"
        : "semantic-review-pending",
    };
  });
  const indexed = new Set(rows.map((row) => row.sha));
  const nonIndexedFixtureReferences = [...links]
    .filter(([sha]) => !indexed.has(sha))
    .map(([sha, references]) => {
      const sourceCommit = MERGE_ORIGINS.get(sha);
      if (sourceCommit) {
        const mergePatch = git(root, ["diff", `${sha}^1`, sha, "--", ...scopes]);
        const sourcePatch = git(root, ["show", "--format=", sourceCommit, "--", ...scopes]);
        const mergePatchId = git(root, ["patch-id", "--stable"], mergePatch).trim().split(" ")[0];
        assert.equal(
          git(root, ["patch-id", "--stable"], sourcePatch).trim().split(" ")[0],
          mergePatchId,
        );
        const row = rows.find((candidate) => candidate.sha === sourceCommit);
        assert(row, "reviewed merge source is absent from the pinned census");
        row.references.push(...references);
        row.review = "fixture-linked-via-reviewed-merge;target-and-semantic-review-pending";
      }
      return {
        sha,
        subject: git(root, ["show", "-s", "--format=%s", sha]).trim(),
        references,
        sourceCommit: sourceCommit ?? null,
        review: sourceCommit
          ? "merge-source-patch-equivalence-verified;target-review-pending"
          : "semantic-scope-review-pending",
      };
    });
  return { raw, commits, fixes, rows, nonIndexedFixtureReferences };
}

export function renderHistory(history: ReturnType<typeof collectHistory>) {
  const groups = new Map<string, string[]>();
  for (const row of history.rows) {
    if (!row.patchId) continue;
    groups.set(row.patchId, [...(groups.get(row.patchId) ?? []), row.sha]);
  }
  const sanitize = (value: string) => value.replace(/[\t\r\n]/g, " ");
  const columns = [
    "fix_sha",
    "subject",
    "touched_paths",
    "changed_witness_candidates",
    "fixture_references",
    "review",
    "patch_equivalent_candidates",
  ];
  const body = history.rows.map((row) =>
    [
      row.sha,
      row.subject,
      row.paths.join(";"),
      row.witnessCandidates.join(";"),
      row.references.map((ref) => `${ref.id}@${ref.evidence}`).join(";"),
      row.review,
      row.patchId ? (groups.get(row.patchId) ?? []).filter((sha) => sha !== row.sha).join(";") : "",
    ]
      .map((value) => sanitize(value) || "-")
      .join("\t"),
  );
  const index = `${columns.join("\t")}\n${body.join("\n")}\n`;
  const receipt = {
    schema: "vize.compiler.fix-history-index",
    version: 1,
    sourceRevision: HISTORY_REVISION,
    paths: HISTORY_CRATES.map((name) => `crates/${name}`),
    command: [
      "git",
      "log",
      "--full-history",
      "--format=%H%x09%P%x09%s",
      HISTORY_REVISION,
      "--",
      ...HISTORY_CRATES.map((name) => `crates/${name}`),
    ],
    historySha256: digest(history.raw),
    touchingCommits: history.commits.length,
    conventionalFixSubjects: history.fixes.length,
    nonMergeFixSubjects: history.rows.length,
    fixtureLinkedFixSubjects: history.rows.filter((row) => row.references.length).length,
    nonIndexedFixtureReferences: history.nonIndexedFixtureReferences,
    patchEquivalentGroups: [...groups.values()].filter((group) => group.length > 1),
    index: { path: INDEX, sha256: digest(index) },
    semanticDenominator: null,
    nativeAcceptance: 0,
    limitations: [
      "The original 609-fix snapshot has no verified source revision.",
      "Changed witness paths and stable patch equivalence are candidates, not coverage or automatic duplicate exclusions.",
      "Fixture references cover only their recorded entrypoint, target, options and public output facets.",
      "Every row retains pending semantic and target review; historical or local execution does not certify current Actions or a native route.",
    ],
  };
  return { index, receipt: `${JSON.stringify(receipt, null, 2)}\n` };
}

function main() {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
  const mode = process.argv[2];
  assert(
    ["--write", "--check"].includes(mode),
    "usage: vp node tools/commands/fixtures/compiler-fix-history.ts --write|--check",
  );
  const rendered = renderHistory(collectHistory(root));
  for (const [relative, bytes] of [
    [INDEX, rendered.index],
    [RECEIPT, rendered.receipt],
  ]) {
    if (mode === "--write") fs.writeFileSync(path.join(root, relative), bytes);
    else {
      const actual = fs.readFileSync(path.join(root, relative), "utf8");
      if (relative === RECEIPT)
        assert.deepEqual(
          JSON.parse(actual),
          JSON.parse(bytes),
          `stale history receipt: ${relative}`,
        );
      else assert.equal(actual, bytes, `stale history index: ${relative}`);
    }
  }
  const receipt = JSON.parse(rendered.receipt);
  console.log(
    JSON.stringify({
      nonMergeFixSubjects: receipt.nonMergeFixSubjects,
      fixtureLinkedFixSubjects: receipt.fixtureLinkedFixSubjects,
      semanticDenominator: null,
      nativeAcceptance: 0,
    }),
  );
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
