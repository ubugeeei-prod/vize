import { execFileSync, spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { loadCompilerCssApiCases } from "../../../../tests/differential/compiler-css-api.ts";

const historyTests = new Set([
  "tests/tooling/compiler-css-api-custody.test.mjs",
  "tests/tooling/compiler-css-binding-custody.test.mjs",
]);
const fixtureRoot = "tests/_fixtures/differential/compiler";

/** Read only the authenticated CSS authorities needed by this actual shard. */
export function cssCustodyRevisions(repository, selected) {
  if (!selected.some((file) => historyTests.has(file))) return [];
  const read = (file) => readFileSync(join(repository, file));
  const manifest = JSON.parse(read(`${fixtureRoot}/manifest.json`));
  // Keep the existing closed schema, complete fixture hashes and producer
  // assertions authoritative; fetching a commit grants no compatibility credit.
  loadCompilerCssApiCases(manifest.cssApiCases, read);
  return [
    ...new Set(
      manifest.cssApiCases.flatMap((row) => {
        const pin = row.inputs.files.find((file) => file.role === "authority");
        const authority = JSON.parse(read(`${fixtureRoot}/${row.inputs.root}/${pin.path}`));
        return [authority.beforeRevision, authority.fixRevision];
      }),
    ),
  ].sort((left, right) => (left < right ? -1 : left > right ? 1 : 0));
}

/** Restore immutable Git objects lost from ordinary main history by squash. */
export function fetchCssCustodyRevisions(repository, revisions) {
  for (const revision of revisions) {
    if (typeof revision !== "string" || !/^[0-9a-f]{40}$/u.test(revision)) {
      throw new Error("CSS custody requires complete immutable commit IDs");
    }
  }
  if (revisions.length === 0) return [];
  const git = (args) =>
    execFileSync("git", args, {
      cwd: repository,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }).trim();
  const source = git(["rev-parse", "HEAD"]);
  const available = (revision) => {
    const result = spawnSync("git", ["cat-file", "-e", `${revision}^{commit}`], {
      cwd: repository,
      stdio: "ignore",
    });
    if (result.error) throw result.error;
    if (result.signal) throw new Error(`CSS history lookup terminated by ${result.signal}`);
    return result.status === 0;
  };
  const missing = [...new Set(revisions)].filter((revision) => !available(revision));
  if (missing.length) git(["fetch", "--no-tags", "origin", ...missing]);
  for (const revision of revisions) {
    if (!available(revision)) throw new Error(`Missing CSS custody commit ${revision}`);
  }
  if (git(["rev-parse", "HEAD"]) !== source) {
    throw new Error("Candidate HEAD changed while fetching CSS custody history");
  }
  return missing;
}

export function prepareCssCustodyHistory(repository, selected) {
  const revisions = cssCustodyRevisions(repository, selected);
  const fetched = fetchCssCustodyRevisions(repository, revisions);
  return { revisions, fetched };
}
