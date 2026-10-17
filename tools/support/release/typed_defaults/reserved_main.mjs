import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { authenticate, consumerPaths, githubReader, optionsFrom, sha256 } from "./custody.mjs";
import { acquireDistribution, assertLoadedBinary } from "./distribution.mjs";
import { observePackets } from "./observe.ts";
import { assertStableFix, assertStableOfficial } from "./post_observation.mjs";
import { authenticateReservedFixes } from "./reserved_fixes.mjs";
import { loadReservedFixtures, reservedContract, reservedManifest } from "./reserved_fixtures.mjs";

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
  schema: "vize.released-reserved-keys-acceptance",
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
  receipt.reservedFixes = await authenticateReservedFixes(options.head, json);
  persist();
  const fixtures = await loadReservedFixtures(
    options.head,
    path.join(consumer, "source-fixtures"),
    json,
    official.commit.tree.sha,
  );
  receipt.reservedContract = reservedContract;
  receipt.reservedAuthorities = reservedManifest;
  assert.equal(fixtures.authorities.length, 97);
  receipt.sourceAuthorities = fixtures.authorities;
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
  assert.equal(receipt.observations.length, reservedContract.processes);
  for (const [suite, count] of Object.entries(reservedContract.suites))
    assert.equal(receipt.observations.filter((entry) => entry.suite === suite).length, count);
  const final = await authenticate(options, json);
  const finalReservedFixes = await authenticateReservedFixes(options.head, json);
  receipt.finalOfficial = final;
  receipt.finalReservedFixes = finalReservedFixes;
  persist();
  assert.equal(finalReservedFixes.length, receipt.reservedFixes.length);
  receipt.reservedFixes.forEach((fix, index) => assertStableFix(fix, finalReservedFixes[index]));
  assertStableOfficial(official, final);
  assertLoadedBinary(distribution.member.binary, distribution.member, consumer);
  receipt.completedAt = new Date().toISOString();
  receipt.success = true;
  receipt.evidence =
    "443 separately declared reserved-key whole CLI packets (113 + 161 + 169); original 392 and 357 remain separate, no full ecosystem Matrix acceptance";
  persist();
  console.log(
    JSON.stringify({
      success: true,
      reserved: 443,
      suites: reservedContract.suites,
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
