// Writes the verified full-workspace typechecker observation (T1) into the
// shared differential bundle. It re-runs the same verification as
// `typechecker-workspace.ts verify` over the downloaded workspace artifact; a
// producer chain that fails verification writes no entry, so the census shows
// the typechecker as unmeasured instead of trusting an unverified report.
import assert from "node:assert/strict";
import path from "node:path";
import { writeBundleEntry } from "./bundle.ts";
import { loadTypecheckerManifest } from "./typechecker.ts";
import { aggregateTypecheckerWorkspace } from "./typechecker-workspace.ts";
import { typecheckerSourceIdentity } from "./typechecker-shards.ts";

const [sha, resultDir, runId, attempt, bundleDir] = process.argv.slice(2);
assert(
  sha && resultDir && runId && attempt && bundleDir,
  "usage: typechecker-bundle.ts SHA RESULT_DIR RUN_ID PRODUCER_ATTEMPT BUNDLE_DIR",
);
const repoRoot = process.cwd();
assert.equal(typecheckerSourceIdentity(repoRoot).sha, sha);
assert.equal(runId, process.env.GITHUB_RUN_ID, "the producer must belong to this run");
const result = aggregateTypecheckerWorkspace({
  repoRoot,
  resultDir: path.resolve(resultDir),
  expectedExecution: { runId: Number(runId), runAttempt: Number(attempt) },
});
const manifestPath = path.join(repoRoot, "tests/_fixtures/differential/typechecker/manifest.json");
const entry = writeBundleEntry({
  repoRoot,
  bundleDir: path.resolve(bundleDir),
  tier: "T1",
  pack: "workspace",
  manifestPath,
  loaded: loadTypecheckerManifest(manifestPath),
  report: result.report,
  acceptance: result.acceptance,
});
process.stdout.write(`typechecker bundle entry: ${entry}\n`);
