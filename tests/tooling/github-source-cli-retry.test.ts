import assert from "node:assert/strict";
import { type SpawnSyncReturns } from "node:child_process";
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
  CONSUMER_USE_MEMBER,
  TRANSPORT_MEMBER,
  command,
  digest,
  rustcIdentity,
  runtimeIdentity,
  sourceCliProducerOutputs,
  sharedOracleAuthority,
  verifySourceCliConsumerUse,
} from "../../tools/support/compat/github/source-cli-identity.ts";

function commandResult(stdout: string, error?: NodeJS.ErrnoException): SpawnSyncReturns<string> {
  return {
    pid: 1,
    output: [null, stdout, ""],
    stdout,
    stderr: "",
    status: error === undefined ? 0 : null,
    signal: error === undefined ? null : "SIGTERM",
    error,
  };
}

void test("rustc identity retries one transient timeout with a bounded larger budget", () => {
  const timeouts: number[] = [];
  const output = rustcIdentity("/fixture", (_executable, _args, options) => {
    timeouts.push(options.timeout);
    if (timeouts.length === 1) {
      return commandResult(
        "",
        Object.assign(new Error("spawnSync rustc ETIMEDOUT"), { code: "ETIMEDOUT" }),
      );
    }
    return commandResult("rustc 1.90.0\n");
  });

  assert.equal(output, "rustc 1.90.0");
  assert.deepEqual(timeouts, [30_000, 60_000]);
});

void test("rustc identity stops after the finite timeout recovery budget", () => {
  const timeouts: number[] = [];
  assert.throws(
    () =>
      rustcIdentity("/fixture", (_executable, _args, options) => {
        timeouts.push(options.timeout);
        return commandResult(
          "",
          Object.assign(new Error("spawnSync rustc ETIMEDOUT"), { code: "ETIMEDOUT" }),
        );
      }),
    /ETIMEDOUT/,
  );
  assert.deepEqual(timeouts, [30_000, 60_000]);
});

// Real Git/tar and complete inert member bytes prove transport, never a Rust build.
function fixture() {
  const base = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "source-cli-retry-")));
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
    "fixture",
  ]);
  command(base, "git", ["clone", "--quiet", producer, consumer]);
  for (const member of [CLI_MEMBERS[0], CLI_MEMBERS[2]]) {
    fs.mkdirSync(path.dirname(path.join(producer, member)), { recursive: true });
    fs.writeFileSync(path.join(producer, member), `complete inert ${member}\n`, { mode: 0o755 });
  }
  writeBuildReceipt(producer);
  fs.chmodSync(path.join(producer, CLI_MEMBERS[1]), 0o644);
  const identity: ReturnType<typeof runtimeIdentity> = {
    sourceRevision: command(producer, "git", ["rev-parse", "HEAD"]),
    sourceTree: command(producer, "git", ["rev-parse", "HEAD^{tree}"]),
    runId: "123",
    runAttempt: "1",
    platform: "linux",
    arch: "x64",
    rustc: "fixture rustc",
    cargo: "fixture cargo",
    linker: "fixture wild",
    linkerSha256: "1".repeat(64),
    driver: "fixture clang",
    cargoConfig: "binding config",
    flags: {},
  };
  const archive = path.join(base, "source-cli.tar.gz");
  const hashes = packSourceCli(producer, archive, identity);
  const retry = { ...identity, runAttempt: "2" };
  return { base, producer, consumer, identity, retry, archive, hashes };
}
type Fixture = ReturnType<typeof fixture>;
function law(name: string, run: (value: Fixture) => void) {
  void test(name, () => {
    const value = fixture();
    try {
      run(value);
    } finally {
      fs.rmSync(value.base, { recursive: true, force: true });
    }
  });
}
function restore(value: Fixture) {
  return restoreSourceCli(
    value.consumer,
    value.archive,
    value.hashes.sha256,
    value.retry,
    "1",
    value.hashes.receiptSha256,
  );
}
function withEnvironment(values: Record<string, string | undefined>, run: () => void) {
  const before = Object.fromEntries(Object.keys(values).map((key) => [key, process.env[key]]));
  try {
    for (const [key, value] of Object.entries(values)) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
    run();
  } finally {
    for (const [key, value] of Object.entries(before)) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
  }
}

law(
  "successful producer1 remains byte-exact while retry consumer2 gets separate immutable use custody",
  (value) => {
    const useSha = restore(value);
    for (const member of CLI_MEMBERS)
      assert.deepEqual(
        fs.readFileSync(path.join(value.consumer, member)),
        fs.readFileSync(path.join(value.producer, member)),
      );
    const producerPath = path.join(value.consumer, "target/ci/vize.source-build.json");
    assert.equal(digest(fs.readFileSync(producerPath)), value.hashes.receiptSha256);
    const producer = JSON.parse(fs.readFileSync(producerPath, "utf8"));
    assert.equal(producer.identity.runAttempt, "1");
    const use = verifySourceCliConsumerUse(
      value.consumer,
      producer,
      value.retry,
      value.hashes.receiptSha256,
      useSha,
    );
    assert.equal(use.consumerIdentity.runAttempt, "2");
    assert.equal(use.producerIdentity.runAttempt, "1");
    assert.equal(use.archiveSha256, value.hashes.sha256);
    assert.equal(fs.statSync(path.join(value.consumer, CONSUMER_USE_MEMBER)).mode & 0o777, 0o644);
    withEnvironment(
      {
        GITHUB_SHA: value.retry.sourceRevision,
        GITHUB_RUN_ID: "123",
        GITHUB_RUN_ATTEMPT: "2",
        VIZE_SOURCE_CLI_RECEIPT_SHA256: value.hashes.receiptSha256,
        VIZE_SOURCE_CLI_USE_RECEIPT_SHA256: useSha,
      },
      () =>
        assert.deepEqual(
          sharedOracleAuthority(value.consumer, path.join(value.consumer, CLI_MEMBERS[2])),
          producer,
        ),
    );
  },
);

law(
  "same-attempt default succeeds but an unauthorized earlier producer or future attempt refuses before writes",
  (value) => {
    assert.throws(() =>
      restoreSourceCli(value.consumer, value.archive, value.hashes.sha256, value.retry),
    );
    for (const attempt of ["2", "3", "0", "01", "-1", "2.0", "NaN", "99999999999"])
      assert.throws(() =>
        restoreSourceCli(value.consumer, value.archive, value.hashes.sha256, value.retry, attempt),
      );
    assert.throws(
      () =>
        restoreSourceCli(
          value.consumer,
          value.archive,
          value.hashes.sha256,
          value.retry,
          "1",
          "0".repeat(64),
        ),
      /producer envelope digest/,
    );
    assert.equal(fs.existsSync(path.join(value.consumer, "target")), false);
    restoreSourceCli(value.consumer, value.archive, value.hashes.sha256, value.identity);
  },
);

law(
  "retry does not authorize different head, tree, run, compiler, linker, flags or platform",
  (value) => {
    for (const key of [
      "sourceRevision",
      "sourceTree",
      "runId",
      "rustc",
      "cargo",
      "linker",
      "linkerSha256",
      "driver",
      "cargoConfig",
      "platform",
      "arch",
    ] as const)
      assert.throws(() =>
        restoreSourceCli(
          value.consumer,
          value.archive,
          value.hashes.sha256,
          { ...value.retry, [key]: "foreign" },
          "1",
        ),
      );
    assert.throws(() =>
      restoreSourceCli(
        value.consumer,
        value.archive,
        value.hashes.sha256,
        { ...value.retry, flags: { RUSTFLAGS: "foreign" } },
        "1",
      ),
    );
    assert.equal(fs.existsSync(path.join(value.consumer, "target")), false);
  },
);

law(
  "retry rejects foreign recipes and all changed member bodies even under a recomputed outer digest",
  (value) => {
    const original = fs.readFileSync(value.archive);
    for (const kind of ["recipe", "oracleRecipe", ...CLI_MEMBERS]) {
      fs.writeFileSync(value.archive, original);
      const stage = fs.mkdtempSync(path.join(value.base, "repack-"));
      command(stage, "tar", ["-xzf", value.archive]);
      if (kind === "recipe" || kind === "oracleRecipe") {
        const file = path.join(stage, TRANSPORT_MEMBER);
        const packet = JSON.parse(fs.readFileSync(file, "utf8"));
        packet[kind] = "cargo build --release";
        fs.writeFileSync(file, JSON.stringify(packet));
      } else fs.appendFileSync(path.join(stage, kind), "foreign bytes");
      command(stage, "tar", ["-czf", value.archive, ...CLI_MEMBERS, TRANSPORT_MEMBER]);
      assert.throws(() =>
        restoreSourceCli(
          value.consumer,
          value.archive,
          digest(fs.readFileSync(value.archive)),
          value.retry,
          "1",
        ),
      );
      assert.equal(fs.existsSync(path.join(value.consumer, "target")), false);
    }
  },
);

law(
  "Oracle retry requires the verified use hash; stale, absent or changed use receipt cannot inherit custody",
  (value) => {
    const useSha = restore(value);
    const producer = JSON.parse(
      fs.readFileSync(path.join(value.consumer, "target/ci/vize.source-build.json"), "utf8"),
    );
    const file = path.join(value.consumer, CONSUMER_USE_MEMBER);
    const original = fs.readFileSync(file);
    withEnvironment(
      {
        GITHUB_SHA: value.retry.sourceRevision,
        GITHUB_RUN_ID: "123",
        GITHUB_RUN_ATTEMPT: "2",
        VIZE_SOURCE_CLI_RECEIPT_SHA256: value.hashes.receiptSha256,
        VIZE_SOURCE_CLI_USE_RECEIPT_SHA256: undefined,
      },
      () =>
        assert.throws(
          () => sharedOracleAuthority(value.consumer, path.join(value.consumer, CLI_MEMBERS[2])),
          /verified consumer use/,
        ),
    );
    assert.throws(() =>
      verifySourceCliConsumerUse(
        value.consumer,
        producer,
        { ...value.retry, runAttempt: "3" },
        value.hashes.receiptSha256,
        useSha,
      ),
    );
    for (const mutation of [
      "malformed",
      "unknown",
      "foreign member",
      "foreign producer",
      "foreign consumer",
      "mode",
      "symlink",
      "absent",
    ]) {
      fs.rmSync(file, { force: true });
      fs.writeFileSync(file, original, { mode: 0o644 });
      if (mutation === "malformed") fs.writeFileSync(file, "{");
      else if (
        ["unknown", "foreign member", "foreign producer", "foreign consumer"].includes(mutation)
      ) {
        const use = JSON.parse(original.toString());
        if (mutation === "unknown") use.version = 2;
        if (mutation === "foreign member") use.members[0].sha256 = "0".repeat(64);
        if (mutation === "foreign producer") use.producerIdentity.runAttempt = "2";
        if (mutation === "foreign consumer") use.consumerIdentity.flags = { RUSTFLAGS: "foreign" };
        fs.writeFileSync(file, JSON.stringify(use));
      } else if (mutation === "mode") fs.chmodSync(file, 0o755);
      else if (mutation === "symlink") {
        fs.rmSync(file);
        fs.symlinkSync("vize.source-build.json", file);
      } else fs.rmSync(file);
      assert.throws(() =>
        verifySourceCliConsumerUse(
          value.consumer,
          producer,
          value.retry,
          value.hashes.receiptSha256,
          ["mode", "symlink", "absent"].includes(mutation) ? useSha : digest(fs.readFileSync(file)),
        ),
      );
    }
  },
);

void test("retained outputs bind exact digests and producer attempt; no latest lookup or consumer restamping", () => {
  const outputs = {
    sha256: "1".repeat(64),
    "receipt-sha256": "2".repeat(64),
    "producer-attempt": "1",
  };
  assert.deepEqual(sourceCliProducerOutputs(JSON.stringify(outputs), "2"), outputs);
  for (const changed of [
    { ...outputs, "producer-attempt": "3" },
    { ...outputs, "producer-attempt": "01" },
    { ...outputs, sha256: "foreign" },
    { ...outputs, "receipt-sha256": "foreign" },
    { ...outputs, latest: "true" },
    { sha256: outputs.sha256 },
  ])
    assert.throws(() => sourceCliProducerOutputs(JSON.stringify(changed), "2"));
});
