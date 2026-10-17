import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { authenticate, consumerPaths, githubReader, optionsFrom, sha256 } from "./custody.mjs";
import { acquireDistribution, assertLoadedBinary } from "./distribution.mjs";
import { observePackets } from "./observe.ts";
import { assertStableFix, assertStableOfficial } from "./post_observation.mjs";
import {
  authenticateSetupFix,
  loadSetupFixtures,
  setupContract,
  setupOriginals,
  setupScenarios,
} from "./setup_emits.mjs";

const checkout = fs.realpathSync(fileURLToPath(new URL("../../../../", import.meta.url)));
const options = optionsFrom(process.env);
assert.equal(process.platform, "linux", "actual hosted GNU consumer required");
assert.equal(process.arch, "x64");
assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
assert.equal(process.env.GITHUB_REF, "refs/heads/main", "reviewed main observer required");
assert.match(process.env.GITHUB_WORKFLOW_SHA ?? "", /^[0-9a-f]{40}$/);
const driver = spawnSync("git", ["rev-parse", "HEAD"], { cwd: checkout, encoding: "utf8" });
assert.equal(driver.status, 0);
assert.equal(driver.stdout.trim(), process.env.GITHUB_WORKFLOW_SHA);
const { consumer, output } = consumerPaths(process.env, checkout);
fs.mkdirSync(consumer, { mode: 0o700 });
fs.mkdirSync(output, { mode: 0o700 });
const receipt = {
  schema: "vize.released-setup-emits-acceptance",
  version: 1,
  options,
  observer: {
    source: driver.stdout.trim(),
    run: process.env.GITHUB_RUN_ID,
    attempt: process.env.GITHUB_RUN_ATTEMPT,
  },
  consumer: { root: consumer, platform: process.platform, architecture: process.arch },
  startedAt: new Date().toISOString(),
  success: false,
};
const persist = () =>
  fs.writeFileSync(path.join(output, "receipt.json"), `${JSON.stringify(receipt, null, 2)}\n`);
try {
  persist();
  const json = githubReader(output);
  const official = await authenticate(options, json);
  receipt.official = official;
  receipt.setupFix = await authenticateSetupFix(options.head, json);
  persist();
  const fixtures = await loadSetupFixtures(
    options.head,
    path.join(consumer, "source-fixtures"),
    json,
    official.commit.tree.sha,
  );
  fixtures.vectors = setupScenarios(fixtures);
  receipt.setupContract = setupContract;
  receipt.setupOriginals = setupOriginals;
  receipt.sourceAuthorities = fixtures.authorities;
  receipt.corpora = { setup: fixtures.setup, projection: fixtures.projection };
  persist();
  const distribution = await acquireDistribution(official.release, options.tag, consumer);
  receipt.distribution = distribution;
  persist();
  assertLoadedBinary(distribution.member.binary, distribution.member, consumer);
  const version = spawnSync(distribution.member.binary, ["--version"], {
    cwd: consumer,
    timeout: 60_000,
    maxBuffer: 4 * 1024 * 1024,
  });
  const versionOut = version.stdout ?? Buffer.alloc(0),
    versionErr = version.stderr ?? Buffer.alloc(0);
  receipt.versionObservation = {
    status: version.status,
    signal: version.signal,
    stdout: versionOut.toString("utf8"),
    stderr: versionErr.toString("utf8"),
    raw: {
      stdoutBase64: versionOut.toString("base64"),
      stdoutSha256: sha256(versionOut),
      stderrBase64: versionErr.toString("base64"),
      stderrSha256: sha256(versionErr),
    },
    error: version.error ? String(version.error) : null,
  };
  persist();
  assertLoadedBinary(distribution.member.binary, distribution.member, consumer);
  assert.equal(version.error, undefined);
  assert.equal(version.signal, null);
  assert.equal(version.status, 0);
  assert.deepEqual(versionErr, Buffer.alloc(0));
  assert.deepEqual(versionOut, Buffer.from(`vize ${options.tag.slice(1)}\n`));
  receipt.observations = observePackets(fixtures, distribution, consumer, output);
  assert.equal(receipt.observations.length, setupContract.processes);
  assert(receipt.observations.every((entry) => entry.suite === "setup"));
  const final = await authenticate(options, json);
  const finalSetupFix = await authenticateSetupFix(options.head, json);
  receipt.finalOfficial = final;
  receipt.finalSetupFix = finalSetupFix;
  persist();
  assertStableFix(receipt.setupFix, finalSetupFix);
  assertStableOfficial(official, final);
  assertLoadedBinary(distribution.member.binary, distribution.member, consumer);
  receipt.completedAt = new Date().toISOString();
  receipt.success = true;
  receipt.evidence =
    "357 separately declared setup-emits whole CLI packets; original 392 remain separate, no full ecosystem Matrix acceptance";
  persist();
  console.log(
    JSON.stringify({
      success: true,
      setup: 357,
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
