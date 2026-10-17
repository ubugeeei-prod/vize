import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { consumerPaths, githubReader, sha256 } from "../typed_defaults/custody.mjs";
import { acquireDistribution, assertLoadedBinary } from "../typed_defaults/distribution.mjs";
import { assertStableFix, assertStableOfficial } from "../typed_defaults/post_observation.mjs";
import { authenticateReports, reportOptions } from "./custody.mjs";
import { loadFixtures } from "./fixtures.mjs";
import { installDependencies } from "./dependencies.mjs";
import { observeReports } from "./observe.ts";

const checkout = fs.realpathSync(fileURLToPath(new URL("../../../../", import.meta.url)));
const options = reportOptions(process.env);
assert.equal(process.platform, "linux");
assert.equal(process.arch, "x64");
assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
assert.equal(process.env.GITHUB_REF, "refs/heads/main");
assert.match(process.env.GITHUB_WORKFLOW_SHA ?? "", /^[0-9a-f]{40}$/);
const driver = spawnSync("git", ["rev-parse", "HEAD"], { cwd: checkout, encoding: "utf8" });
assert.equal(driver.status, 0);
assert.equal(driver.stdout.trim(), process.env.GITHUB_WORKFLOW_SHA);
const { consumer, output } = consumerPaths(process.env, checkout);
fs.mkdirSync(consumer, { mode: 0o700 });
fs.mkdirSync(output, { mode: 0o700 });
const receipt = {
  schema: "vize.released-lint-reports-8752-8756",
  version: 1,
  options,
  observer: {
    source: driver.stdout.trim(),
    run: process.env.GITHUB_RUN_ID,
    attempt: process.env.GITHUB_RUN_ATTEMPT,
  },
  startedAt: new Date().toISOString(),
  consumer,
  success: false,
};
const persist = () =>
  fs.writeFileSync(path.join(output, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
try {
  persist();
  const json = githubReader(output);
  const authenticated = await authenticateReports(options, json);
  receipt.authority = authenticated;
  persist();
  const fixtures = await loadFixtures(
    options.head,
    authenticated.official.commit.tree.sha,
    path.join(consumer, "source-fixtures"),
    json,
  );
  receipt.fixtureAuthorities = fixtures.authorities;
  persist();
  const distribution = await acquireDistribution(
    authenticated.official.release,
    options.tag,
    consumer,
  );
  receipt.distribution = distribution;
  persist();
  assertLoadedBinary(distribution.member.binary, distribution.member, consumer);
  const probe = spawnSync(distribution.member.binary, ["--version"], {
    cwd: consumer,
    timeout: 30_000,
  });
  receipt.versionObservation = {
    status: probe.status,
    signal: probe.signal,
    error: probe.error?.message ?? null,
    stdoutBase64: probe.stdout?.toString("base64"),
    stderrBase64: probe.stderr?.toString("base64"),
  };
  persist();
  assert.equal(probe.error, undefined);
  assert.equal(probe.signal, null);
  assert.equal(probe.status, 0);
  assert.equal(probe.stdout.toString("utf8"), `vize ${options.tag.slice(1)}\n`);
  assert.equal(probe.stderr.length, 0);
  const installed = await installDependencies(consumer, output, fixtures.source);
  receipt.provider = installed.providerObservation;
  persist();
  receipt.observations = observeReports(
    fixtures,
    distribution,
    consumer,
    output,
    installed.assertDependencies,
  );
  const final = await authenticateReports(options, json);
  receipt.finalAuthority = final;
  persist();
  assertStableOfficial(authenticated.official, final.official);
  for (let i = 0; i < authenticated.reports.length; i++)
    assertStableFix(authenticated.reports[i], final.reports[i]);
  assertLoadedBinary(distribution.member.binary, distribution.member, consumer);
  installed.assertDependencies();
  receipt.completedAt = new Date().toISOString();
  receipt.success = true;
  receipt.evidence =
    "29 complete frozen template formatter/status packets and 36 original computed count/error contracts on the actual public GNU CLI; no full computed diagnostic, Nuxt, Vapor or ecosystem acceptance";
  persist();
  console.log(
    JSON.stringify({
      success: true,
      template: 29,
      computedCountContracts: 36,
      head: options.head,
      binarySha256: distribution.member.sha256,
      receiptSha256: sha256(fs.readFileSync(path.join(output, "receipt.json"))),
    }),
  );
} catch (error) {
  receipt.failure = { name: error.name, message: error.message };
  persist();
  throw error;
}
