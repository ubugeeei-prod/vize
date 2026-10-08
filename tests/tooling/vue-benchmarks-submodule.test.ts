import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { UPSTREAM } from "../../tools/benchmarks/scripts/vue-benchmarks-replay-core.mjs";
import {
  createCompatibilityContext,
  readCompatibilityLedger,
  validateCompatibilityLedger,
} from "../../tools/support/compat/fixtures/fixture-compatibility-ledger.mjs";
import manifest from "../_fixtures/vue-benchmarks-upstream.json" with { type: "json" };

const fixture = "tests/_fixtures/_git/vue-benchmarks";
const revision = "5489aee433cd1054b9d72973457498544da7c467";
const git = (...args: string[]) => execFileSync("git", args, { encoding: "utf8" }).trim();

void test("benchmark inputs are a real upstream gitlink with preserved license custody", () => {
  assert.equal(manifest.schema, "vize.benchmarkFixture");
  assert.equal(manifest.version, 1);
  assert.equal(manifest.trackingIssue, 7856);
  assert.equal(manifest.fixturePath, fixture);
  assert.equal(manifest.repository, "https://github.com/pikax/vue-benchmarks");
  assert.equal(manifest.revision, revision);
  assert.equal(git("ls-files", "--stage", "--", fixture), `160000 ${revision} 0\t${fixture}`);
  const section = `submodule.${fixture}`;
  assert.equal(git("config", "-f", ".gitmodules", "--get", `${section}.path`), fixture);
  assert.equal(git("config", "-f", ".gitmodules", "--get", `${section}.url`), manifest.repository);
  assert.equal(git("config", "-f", ".gitmodules", "--get", `${section}.shallow`), "true");
  assert.deepEqual(manifest.license, {
    spdx: "MIT",
    files: [
      {
        path: "LICENSE",
        gitBlob: "a7017290772b36ce31d5e94645177b289564983e",
        bytes: 1084,
      },
    ],
  });
});

void test("the new pin retains historical results without promoting current or native credit", () => {
  assert.deepEqual(manifest.publishedSnapshot.toolVersions, {
    vize: "0.429.1",
    "@vizejs/native": "0.429.1",
  });
  assert.equal(
    manifest.publishedSnapshot.sourceRevision,
    "8a8848276c52956d7e54e262e5846e41fc922288",
  );
  assert.equal(manifest.publishedSnapshot.workflowRunId, 36570273148);
  assert.equal(manifest.currentSourceValidation, "not-executed");
  assert.equal(manifest.nativeMigrationCredit, 0);
  const context = createCompatibilityContext();
  const ledger = readCompatibilityLedger();
  const validated = validateCompatibilityLedger(ledger, context);
  assert.deepEqual(validated.fixtureMap.get(fixture), {
    fixturePath: fixture,
    memberships: [],
    dialectCoverage: { state: "unknown", evidence: [] },
  });
  assert.equal(
    context.registry.projects.some((project) => project.fixturePath === fixture),
    false,
  );
  for (const members of validated.oracleFixtures.values())
    assert.equal(members.has(fixture), false);
  assert.equal(
    ledger.capabilities.some((row) => row.fixturePath === fixture),
    false,
  );
});

void test("the existing disposable replay keeps its independent old pin and paths", () => {
  assert.equal(UPSTREAM.commit, "65c6102504b14cd49c0b03305be8dd0b9d208c59");
  assert.notEqual(UPSTREAM.commit, manifest.revision);
  const workflow = readFileSync(".github/workflows/vue-benchmarks-replay.yml", "utf8");
  assert.equal(workflow.includes(fixture), false);
  assert.equal(workflow.includes("vue-benchmarks-replay.mjs"), true);
});
