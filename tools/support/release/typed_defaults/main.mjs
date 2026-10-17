import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { authenticate, consumerPaths, githubReader, optionsFrom, sha256 } from "./custody.mjs";
import { acquireDistribution, assertLoadedBinary } from "./distribution.mjs";
import { loadFixtures } from "./fixtures.mjs";
import { observe } from "./observe.ts";
import { assertStableFix, assertStableOfficial } from "./post_observation.mjs";
import { originalContracts, originalObservers, originalScenarios } from "./contracts.mjs";
import {
  authenticateReturnFix,
  returnContract,
  returnObserver,
  returnScenarios,
} from "./return_prefilter.mjs";

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
  schema: "vize.released-typed-defaults-acceptance",
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
  receipt.returnFix = await authenticateReturnFix(options.head, json);
  persist();
  const fixtures = await loadFixtures(
    options.head,
    path.join(consumer, "source-fixtures"),
    json,
    official.commit.tree.sha,
  );
  fixtures.vectors = [...originalScenarios(fixtures), ...returnScenarios(fixtures)];
  receipt.originalContracts = originalContracts;
  receipt.originalObservers = originalObservers;
  receipt.returnContract = returnContract;
  receipt.returnObserver = returnObserver;
  receipt.sourceAuthorities = fixtures.authorities;
  receipt.corpora = {
    typed: fixtures.typed,
    returns: fixtures.returns,
    first: fixtures.first,
    second: fixtures.second,
    projection: fixtures.projection,
  };
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
  receipt.observations = observe(fixtures, distribution, consumer, output);
  const final = await authenticate(options, json);
  const finalReturnFix = await authenticateReturnFix(options.head, json);
  receipt.finalOfficial = final;
  receipt.finalReturnFix = finalReturnFix;
  persist();
  assertStableFix(receipt.returnFix, finalReturnFix);
  assertStableOfficial(official, final);
  assertLoadedBinary(distribution.member.binary, distribution.member, consumer);
  receipt.completedAt = new Date().toISOString();
  receipt.success = true;
  receipt.evidence =
    "56 original typed-default, 224 original compatibility, and 112 original return-prefilter whole CLI packets; no full ecosystem Matrix acceptance";
  persist();
  console.log(
    JSON.stringify({
      success: true,
      typed: 56,
      compatibility: 224,
      returns: 112,
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
