import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { writeBuildReceipt } from "../differential/build-receipt.ts";
import {
  packSourceCli,
  restoreSourceCli,
} from "../../tools/support/compat/github/source-cli-artifact.ts";
import {
  CLI_MEMBERS,
  TRANSPORT_MEMBER,
  command,
  digest,
  runtimeIdentity,
  sourceCliCheckout,
} from "../../tools/support/compat/github/source-cli-identity.ts";

// Inert members qualify real Git object and tar custody, not a Rust build.
void test("real replacement objects refuse a false tuple over original physical source", () => {
  const base = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "source-cli-replace-")));
  try {
    const producer = path.join(base, "producer");
    const consumer = path.join(base, "consumer");
    fs.mkdirSync(producer);
    const cargo = '[workspace.package]\nversion = "0.1.0"\n';
    fs.writeFileSync(path.join(producer, "Cargo.toml"), cargo);
    command(producer, "git", ["init", "--quiet"]);
    const commit = (message: string) => {
      command(producer, "git", ["add", "Cargo.toml"]);
      command(producer, "git", [
        "-c",
        "user.name=Transport law",
        "-c",
        "user.email=transport@example.invalid",
        "commit",
        "--quiet",
        "-m",
        message,
      ]);
    };
    commit("physical original");
    const original = sourceCliCheckout(producer);
    fs.writeFileSync(path.join(producer, "Cargo.toml"), cargo.replace("0.1.0", "0.2.0"));
    commit("replacement counterexample");
    const replacement = sourceCliCheckout(producer);
    command(producer, "git", ["update-ref", "HEAD", original.sourceRevision]);
    fs.writeFileSync(path.join(producer, "Cargo.toml"), cargo);
    assert.equal(command(producer, "git", ["diff", "HEAD"]), "");
    command(base, "git", ["clone", "--quiet", producer, consumer]);
    command(consumer, "git", ["fetch", "--quiet", producer, replacement.sourceRevision]);
    command(consumer, "git", ["replace", original.sourceRevision, replacement.sourceRevision]);
    assert.equal(command(consumer, "git", ["rev-parse", "HEAD^{tree}"]), replacement.sourceTree);
    assert.deepEqual(sourceCliCheckout(consumer), original);
    assert.equal(fs.readFileSync(path.join(consumer, "Cargo.toml"), "utf8"), cargo);
    for (const member of [CLI_MEMBERS[0], CLI_MEMBERS[2]]) {
      fs.mkdirSync(path.dirname(path.join(producer, member)), { recursive: true });
      fs.writeFileSync(path.join(producer, member), `complete inert ${member}\n`, { mode: 0o755 });
    }
    writeBuildReceipt(producer);
    fs.chmodSync(path.join(producer, CLI_MEMBERS[1]), 0o644);
    const identity: ReturnType<typeof runtimeIdentity> = {
      ...original,
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
    const stage = path.join(base, "forged");
    fs.mkdirSync(stage);
    command(stage, "tar", ["-xzf", archive]);
    const file = path.join(stage, TRANSPORT_MEMBER);
    const packet = JSON.parse(fs.readFileSync(file, "utf8"));
    packet.identity.sourceTree = replacement.sourceTree;
    fs.writeFileSync(file, JSON.stringify(packet));
    const forged = path.join(base, "forged.tar.gz");
    command(stage, "tar", ["-czf", forged, ...CLI_MEMBERS, TRANSPORT_MEMBER]);
    const falseConsumer = { ...identity, sourceTree: replacement.sourceTree, runAttempt: "2" };
    assert.throws(
      () =>
        restoreSourceCli(
          consumer,
          forged,
          digest(fs.readFileSync(forged)),
          falseConsumer,
          "1",
          digest(fs.readFileSync(file)),
        ),
      /sourceTree/,
    );
    assert.equal(fs.existsSync(path.join(consumer, "target")), false);
    restoreSourceCli(
      consumer,
      archive,
      hashes.sha256,
      { ...identity, runAttempt: "2" },
      "1",
      hashes.receiptSha256,
    );
    assert.equal(command(consumer, "git", ["rev-parse", "HEAD^{tree}"]), replacement.sourceTree);
    for (const member of CLI_MEMBERS)
      assert.deepEqual(
        fs.readFileSync(path.join(consumer, member)),
        fs.readFileSync(path.join(producer, member)),
      );
  } finally {
    fs.rmSync(base, { recursive: true, force: true });
  }
});
