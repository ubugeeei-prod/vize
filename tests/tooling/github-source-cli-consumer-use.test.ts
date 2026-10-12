import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { writeBuildReceipt } from "../differential/build-receipt.ts";
import {
  packSourceCli,
  restoreSourceCli,
} from "../../tools/support/compat/github/source-cli-artifact.ts";
import {
  CLI_MEMBERS,
  CONSUMER_USE_MEMBER,
  command,
  digest,
  runtimeIdentity,
  sourceCliCheckout,
} from "../../tools/support/compat/github/source-cli-identity.ts";

const adapter = fileURLToPath(
  new URL("../../tools/support/compat/github/source-cli-artifact.ts", import.meta.url),
);

// Real Git/tar and complete inert members prove the subprocess adapter, not a Rust build.
void test("consumer-use CLI authenticates the same immutable retry receipts and refuses foreign use", () => {
  const base = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "source-cli-use-")));
  try {
    const producer = path.join(base, "producer");
    const consumer = path.join(base, "consumer");
    fs.mkdirSync(producer);
    fs.writeFileSync(path.join(producer, "Cargo.toml"), '[workspace.package]\nversion = "0.1.0"\n');
    command(producer, "git", ["init", "--quiet"]);
    command(producer, "git", ["add", "Cargo.toml"]);
    command(producer, "git", [
      "-c",
      "user.name=Transport law",
      "-c",
      "user.email=transport@example.invalid",
      "commit",
      "--quiet",
      "-m",
      "physical fixture",
    ]);
    command(base, "git", ["clone", "--quiet", producer, consumer]);
    for (const member of [CLI_MEMBERS[0], CLI_MEMBERS[2]]) {
      fs.mkdirSync(path.dirname(path.join(producer, member)), { recursive: true });
      fs.writeFileSync(path.join(producer, member), `complete inert ${member}\n`, { mode: 0o755 });
    }
    writeBuildReceipt(producer);
    fs.chmodSync(path.join(producer, CLI_MEMBERS[1]), 0o644);
    const identity: ReturnType<typeof runtimeIdentity> = {
      ...sourceCliCheckout(producer),
      runId: "123",
      runAttempt: "1",
      platform: "linux",
      arch: "x64",
      rustc: "fixture rustc",
      cargo: "fixture cargo",
      linker: "fixture Wild",
      linkerSha256: "1".repeat(64),
      driver: "fixture clang",
      cargoConfig: "binding config",
      flags: {},
    };
    const archive = path.join(base, "original.tar.gz");
    const hashes = packSourceCli(producer, archive, identity);
    const useSha = restoreSourceCli(
      consumer,
      archive,
      hashes.sha256,
      { ...identity, runAttempt: "2" },
      "1",
      hashes.receiptSha256,
    );
    const environment: Record<string, string> = {
      GITHUB_SHA: identity.sourceRevision,
      GITHUB_RUN_ID: identity.runId,
      GITHUB_RUN_ATTEMPT: "2",
      VIZE_SOURCE_CLI_RECEIPT_SHA256: hashes.receiptSha256,
      VIZE_SOURCE_CLI_USE_RECEIPT_SHA256: useSha,
    };
    const run = (overrides: Record<string, string | undefined> = {}, args: string[] = []) => {
      const env = { ...process.env, ...environment, ...overrides };
      for (const [key, value] of Object.entries(env)) if (value === undefined) delete env[key];
      return spawnSync(process.execPath, [adapter, "consumer-use", ...args], {
        cwd: consumer,
        env,
        encoding: "utf8",
        timeout: 30_000,
      });
    };
    const positive = run();
    assert.equal(positive.error, undefined);
    assert.equal(positive.signal, null);
    assert.equal(positive.status, 0, positive.stderr);
    assert.equal(positive.stderr, "");
    assert.equal(
      positive.stdout,
      `${JSON.stringify({ useReceiptSha256: useSha, producerAttempt: "1", consumerAttempt: "2" })}\n`,
    );
    const refuse = (overrides: Record<string, string | undefined>, reason: RegExp) => {
      const result = run(overrides);
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      assert.equal(result.status, 1);
      assert.equal(result.stdout, "");
      assert.match(result.stderr, reason);
    };
    refuse({ VIZE_SOURCE_CLI_RECEIPT_SHA256: undefined }, /original producer receipt digest/);
    refuse(
      { VIZE_SOURCE_CLI_USE_RECEIPT_SHA256: undefined },
      /verified consumer use receipt digest/,
    );
    refuse({ VIZE_SOURCE_CLI_RECEIPT_SHA256: "0".repeat(64) }, /shared producer receipt changed/);
    refuse({ VIZE_SOURCE_CLI_USE_RECEIPT_SHA256: "0".repeat(64) }, /consumer use receipt changed/);
    refuse({ VIZE_SOURCE_CLI_USE_RECEIPT_SHA256: "unknown" }, /\[a-f0-9\]/);
    refuse({ GITHUB_SHA: "0".repeat(40) }, /checked source differs from workflow source/);
    refuse({ GITHUB_RUN_ID: "456" }, /runId/);
    refuse({ GITHUB_RUN_ID: "0" }, /\[1-9\]/);
    refuse({ GITHUB_RUN_ATTEMPT: "3" }, /runAttempt/);
    refuse({ GITHUB_RUN_ATTEMPT: "01" }, /runAttempt/);
    const usePath = path.join(consumer, CONSUMER_USE_MEMBER);
    const useBytes = fs.readFileSync(usePath);
    fs.writeFileSync(usePath, Buffer.concat([useBytes, Buffer.from(" ")]));
    refuse({}, /consumer use receipt changed/);
    fs.writeFileSync(usePath, useBytes);
    fs.chmodSync(usePath, 0o600);
    refuse({}, /384 !== 420/);
    fs.chmodSync(usePath, 0o644);
    fs.renameSync(usePath, `${usePath}.saved`);
    refuse({}, /ENOENT/);
    fs.symlinkSync(`${usePath}.saved`, usePath);
    refuse({}, /regular member required/);
    fs.unlinkSync(usePath);
    fs.renameSync(`${usePath}.saved`, usePath);
    const extraArgument = run({}, ["untrusted"]);
    assert.equal(extraArgument.status, 1);
    assert.equal(extraArgument.stdout, "");
    assert.match(extraArgument.stderr, /2 !== 1/);
    assert.equal(digest(fs.readFileSync(usePath)), useSha);
    assert.equal(
      digest(fs.readFileSync(path.join(consumer, "target/ci/vize.source-build.json"))),
      hashes.receiptSha256,
    );
    for (const member of CLI_MEMBERS)
      assert.deepEqual(
        fs.readFileSync(path.join(consumer, member)),
        fs.readFileSync(path.join(producer, member)),
      );
    assert.equal(run().stdout, positive.stdout);
  } finally {
    fs.rmSync(base, { recursive: true, force: true });
  }
});
