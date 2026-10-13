import assert from "node:assert/strict";
import { apiRoot, repository, sha256 } from "./custody.mjs";

export const returnDirectory = "tests/_fixtures/differential/lint/return-prefilter-8090";
export const returnMerge = "1d988f6bcac8afc6a0fcad9b76b0d5f2da27be71";
export const returnObserver = "839232890e34af66ee2930b7cb4ae303f479e68ab8796f623c0af12810823873";
export const returnContract = {
  processes: 112,
  sha256: "8bb6461a908369549defff9d5807dcaa49d816ffe557f91513bb4161cdf6290c",
};

export async function authenticateReturnFix(head, json) {
  const fix = await json(`${apiRoot}/pulls/8643`);
  assert.equal(fix.merged, true);
  assert.equal(fix.merge_commit_sha, returnMerge);
  assert.equal(fix.base.repo.full_name, repository);
  assert.equal(fix.head.repo.full_name, repository);
  const ancestry = await json(`${apiRoot}/compare/${returnMerge}...${head}`);
  assert.equal(ancestry.base_commit.sha, returnMerge);
  assert.equal(ancestry.merge_base_commit.sha, returnMerge);
  assert(["ahead", "identical"].includes(ancestry.status));
  return { fix, ancestry };
}

/** Additional 112 whole packets; the original 56 + 80 + 144 stay separate. */
export function returnScenarios({ returns, projection, wholeJson }) {
  const vectors = returns.cases.flatMap((entry) => [
    ...[null, "recommended", "essential", "ecosystem"].map((preset) => ({
      entry,
      preset,
      config: null,
      findings: entry.diagnostics,
      suite: "returns",
    })),
    { entry, preset: "incremental", config: null, findings: [], suite: "returns" },
    {
      entry,
      preset: null,
      suite: "returns",
      findings: [],
      config:
        JSON.stringify({
          linter: { rules: Object.fromEntries(returns.rules.map((rule) => [rule, "off"])) },
        }) + "\n",
    },
    {
      entry,
      preset: null,
      suite: "returns",
      findings: [],
      config: JSON.stringify({ linter: projection.linter }) + "\n",
    },
  ]);
  const normalized = vectors.map(({ entry, preset, config, findings, suite }) => ({
    suite,
    id: entry.id,
    filename: entry.filename,
    source: entry.source,
    preset,
    config,
    expected: wholeJson(entry, findings),
    expectedStatus: findings.some((finding) => finding.severity === "error") ? 1 : 0,
  }));
  assert.equal(normalized.length, returnContract.processes);
  assert.equal(
    sha256(JSON.stringify(normalized)),
    returnContract.sha256,
    "original return whole behavior vectors changed",
  );
  return vectors;
}
