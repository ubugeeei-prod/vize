import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { pathToFileURL } from "node:url";

type FeatureDeltaOptions = {
  cwd: string;
  historicalParent: string;
  historicalFeature: string;
  landedFeature: string;
  landedMerge: string;
  head: string;
  paths: string[];
};

type FeatureDelta = {
  historicalParent: string;
  historicalFeature: string;
  selectedParent: string;
  selectedFeature: string;
  parentPatchId: string;
  featurePatchId: string;
  landedFeature: string;
  landedMerge: string;
  head: string;
  changed: string[];
};

function git(args: string[], cwd: string, input?: string): string {
  const run = spawnSync("git", args, {
    cwd,
    encoding: "utf8",
    input,
    maxBuffer: 16 * 1024 * 1024,
  });
  assert.equal(run.status, 0, `git ${args.join(" ")} failed: ${run.stderr || run.stdout}`);
  return run.stdout.trim();
}

function isAncestor(ancestor: string, descendant: string, cwd: string): boolean {
  const run = spawnSync("git", ["merge-base", "--is-ancestor", ancestor, descendant], {
    cwd,
    encoding: "utf8",
  });
  assert.ok(
    run.status === 0 || run.status === 1,
    `git merge-base failed: ${run.stderr || run.stdout}`,
  );
  return run.status === 0;
}

export function stablePatchId(revision: string, cwd: string): string {
  const patch = git(["show", "--pretty=format:", "--binary", revision], cwd);
  const run = spawnSync("git", ["patch-id", "--stable"], {
    cwd,
    encoding: "utf8",
    input: `${patch}\n`,
  });
  assert.equal(run.status, 0, `git patch-id failed: ${run.stderr || run.stdout}`);
  const id = run.stdout.trim().split(/\s+/, 1)[0];
  assert.match(id, /^[0-9a-f]{40}$/, `${revision} must have one stable patch identity`);
  return id;
}

export function rebasedEquivalent(historical: string, head: string, cwd: string): string {
  const subject = git(["show", "-s", "--format=%s", historical], cwd);
  assert.ok(subject, `${historical} must have a commit subject`);
  const expectedPatch = stablePatchId(historical, cwd);
  const candidates = git(["log", "--format=%H", "--fixed-strings", `--grep=${subject}`, head], cwd)
    .split("\n")
    .filter(Boolean)
    .filter((candidate) => git(["show", "-s", "--format=%s", candidate], cwd) === subject)
    .filter((candidate) => stablePatchId(candidate, cwd) === expectedPatch);
  assert.equal(
    candidates.length,
    1,
    `${historical} must map to exactly one same-subject, same-patch ancestor of ${head}`,
  );
  const selected = candidates[0];
  assert.ok(isAncestor(selected, head, cwd), `${selected} must be an ancestor of ${head}`);
  return selected;
}

export function selectFeatureDelta(options: FeatureDeltaOptions): FeatureDelta {
  const { cwd, historicalParent, historicalFeature, landedFeature, landedMerge, head, paths } =
    options;
  assert.ok(
    isAncestor(historicalParent, historicalFeature, cwd),
    "the historical feature must descend from its pinned production parent",
  );
  const selectedParent = rebasedEquivalent(historicalParent, landedFeature, cwd);
  const selectedFeature = rebasedEquivalent(historicalFeature, landedFeature, cwd);
  assert.ok(
    isAncestor(selectedParent, selectedFeature, cwd),
    "the selected feature must descend from its selected production parent",
  );
  const changed = git(["diff", "--name-only", selectedParent, selectedFeature, "--", ...paths], cwd)
    .split("\n")
    .filter(Boolean);
  assert.ok(changed.length > 0, "the selected feature delta must not be empty");
  const landingDrift = git(
    ["diff", "--name-only", landedFeature, landedMerge, "--", ...changed],
    cwd,
  )
    .split("\n")
    .filter(Boolean);
  assert.deepEqual(
    landingDrift,
    [],
    "the landed squash must preserve every selected feature path byte-for-byte",
  );
  assert.ok(isAncestor(landedMerge, head, cwd), "the landed squash must be an ancestor of head");
  return {
    historicalParent,
    historicalFeature,
    selectedParent,
    selectedFeature,
    parentPatchId: stablePatchId(historicalParent, cwd),
    featurePatchId: stablePatchId(historicalFeature, cwd),
    landedFeature,
    landedMerge,
    head,
    changed,
  };
}

function main(): void {
  const separator = process.argv.indexOf("--", 2);
  assert.ok(
    separator >= 0,
    "usage: feature-delta.ts <parent> <feature> <landed-feature> <landed-merge> <head> -- <paths...>",
  );
  const [historicalParent, historicalFeature, landedFeature, landedMerge, head] =
    process.argv.slice(2, separator);
  const paths = process.argv.slice(separator + 1);
  assert.ok(
    historicalParent &&
      historicalFeature &&
      landedFeature &&
      landedMerge &&
      head &&
      paths.length > 0,
  );
  process.stdout.write(
    `${JSON.stringify(
      selectFeatureDelta({
        cwd: process.cwd(),
        historicalParent,
        historicalFeature,
        landedFeature,
        landedMerge,
        head,
        paths,
      }),
      null,
      2,
    )}\n`,
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href)
  main();
