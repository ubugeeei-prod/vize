import { execFileSync } from "node:child_process";
import { pathToFileURL } from "node:url";

const retainingRef = "refs/pull/10558/head";

export function fetchEmptyGenericBaseline(options: {
  repository: string;
  source: string;
  baseline: string;
  retaining: string;
}) {
  for (const [role, revision] of Object.entries(options)) {
    if (role !== "repository" && !/^[0-9a-f]{40}$/.test(revision)) {
      throw new Error(role + " must be a complete immutable commit");
    }
  }
  const git = (args: string[]) =>
    execFileSync("git", args, { cwd: options.repository, encoding: "utf8" }).trim();
  const requireSource = () => {
    if (git(["rev-parse", "HEAD"]) !== options.source) {
      throw new Error("Candidate HEAD differs from the authenticated source");
    }
  };
  requireSource();
  // The closed PR retains the reviewed before-image after the squash merge.
  execFileSync("git", ["fetch", "--no-tags", "origin", retainingRef], {
    cwd: options.repository,
    stdio: "inherit",
  });
  if (git(["rev-parse", "FETCH_HEAD"]) !== options.retaining) {
    throw new Error("Retaining PR head differs from the frozen reviewed parent");
  }
  git(["merge-base", "--is-ancestor", options.baseline, options.retaining]);
  const baselineTree = git(["rev-parse", options.baseline + "^{tree}"]);
  requireSource();
  return {
    schema: "vize.empty-generic-baseline-fetch",
    version: 1,
    source: options.source,
    baseline: options.baseline,
    retaining: options.retaining,
    retainingRef,
    baselineTree,
    candidateHeadUnchanged: true,
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const required = (name: string) => {
    const value = process.env[name];
    if (!value) throw new Error(name + " is required");
    return value;
  };
  const receipt = fetchEmptyGenericBaseline({
    repository: process.cwd(),
    source: required("SOURCE_SHA"),
    baseline: required("BASE_SHA"),
    retaining: required("BASE_RETAINING_SHA"),
  });
  process.stdout.write(JSON.stringify(receipt, null, 2) + "\n");
}
