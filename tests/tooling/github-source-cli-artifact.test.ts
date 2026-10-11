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
  normalizeCargoConfig,
  sharedOracleAuthority,
} from "../../tools/support/compat/github/source-cli-identity.ts";

void test("only owned job linker paths normalize; complete configuration and GNU driver stay binding", () => {
  const config = (linker: string) =>
    `[build]\njobs = 12\n[target.x86_64-unknown-linux-gnu]\nlinker = "clang"\nrustflags = ["-Clink-arg=--ld-path=${linker}"]\n[target.x86_64-unknown-linux-musl]\nlinker = "clang"\nrustflags = ["-Clink-arg=--ld-path=${linker}"]\n`;
  const first = "/tmp/producer/wild-install/wild";
  const second = "/tmp/consumer/wild-install/wild";
  assert.equal(
    normalizeCargoConfig(config(first), first),
    normalizeCargoConfig(config(second), second),
  );
  assert.notEqual(
    normalizeCargoConfig(config(first), first),
    normalizeCargoConfig(config(second).replace("jobs = 12", "jobs = 1"), second),
  );
  assert.throws(() => normalizeCargoConfig(config(first), second), /owned Wild/);
  assert.throws(() => normalizeCargoConfig(config(`${first}-foreign`), first), /owned Wild/);
  assert.throws(() =>
    normalizeCargoConfig(config(first).replace('linker = "clang"', 'linker = "foreign"'), first),
  );
});

// Inert bytes qualify the real tar/receipt transport, never a Rust product build.
function fixture() {
  const base = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "source-cli-law-")));
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
    "test fixture",
  ]);
  command(base, "git", ["clone", "--quiet", producer, consumer]);
  for (const member of [CLI_MEMBERS[0], CLI_MEMBERS[2]]) {
    fs.mkdirSync(path.dirname(path.join(producer, member)), { recursive: true });
    fs.writeFileSync(path.join(producer, member), `inert transport ${member}\n`, { mode: 0o755 });
  }
  writeBuildReceipt(producer);
  fs.chmodSync(path.join(producer, CLI_MEMBERS[1]), 0o644);
  const identity: ReturnType<typeof runtimeIdentity> = {
    sourceRevision: command(producer, "git", ["rev-parse", "HEAD"]),
    sourceTree: command(producer, "git", ["rev-parse", "HEAD^{tree}"]),
    runId: "123",
    runAttempt: "2",
    platform: "linux",
    arch: "x64",
    rustc: "fixture rustc",
    cargo: "fixture cargo",
    linker: "fixture wild",
    linkerSha256: "1".repeat(64),
    driver: "fixture clang",
    cargoConfig: 'linker = "clang"\n',
    flags: {},
  };
  const archive = path.join(base, "source-cli.tar.gz");
  const hashes = packSourceCli(producer, archive, identity);
  return { base, producer, consumer, identity, archive, hashes };
}
type Fixture = ReturnType<typeof fixture>;
function law(name: string, run: (fixture: Fixture) => void): void {
  void test(name, () => {
    const value = fixture();
    try {
      run(value);
    } finally {
      fs.rmSync(value.base, { recursive: true, force: true });
    }
  });
}
function rebuild(value: Fixture, mutate: (stage: string) => void): string {
  const stage = path.join(value.base, "repack");
  fs.mkdirSync(stage);
  command(stage, "tar", ["-xzf", value.archive]);
  mutate(stage);
  command(stage, "tar", ["-czf", value.archive, ...CLI_MEMBERS, TRANSPORT_MEMBER]);
  return digest(fs.readFileSync(value.archive));
}
function changeEnvelope(
  value: Fixture,
  mutate: (envelope: Record<string, unknown>) => void,
): string {
  return rebuild(value, (stage) => {
    const filename = path.join(stage, TRANSPORT_MEMBER);
    const envelope = JSON.parse(fs.readFileSync(filename, "utf8"));
    mutate(envelope);
    fs.writeFileSync(filename, `${JSON.stringify(envelope)}\n`);
  });
}

law(
  "shared source transport preserves complete original bytes, modes and unmodified receipt",
  (value) => {
    restoreSourceCli(value.consumer, value.archive, value.hashes.sha256, value.identity);
    for (const member of CLI_MEMBERS) {
      assert.deepEqual(
        fs.readFileSync(path.join(value.consumer, member)),
        fs.readFileSync(path.join(value.producer, member)),
      );
      assert.equal(
        fs.statSync(path.join(value.consumer, member)).mode & 0o777,
        fs.statSync(path.join(value.producer, member)).mode & 0o777,
      );
    }
    assert.equal(
      digest(fs.readFileSync(path.join(value.consumer, "target/ci/vize.source-build.json"))),
      value.hashes.receiptSha256,
    );
  },
);

law("every producer source/runtime tuple mismatch refuses before destination writes", (value) => {
  for (const field of [
    "sourceRevision",
    "sourceTree",
    "runId",
    "runAttempt",
    "platform",
    "arch",
    "rustc",
    "cargo",
    "linker",
    "linkerSha256",
    "driver",
    "cargoConfig",
  ] as const) {
    assert.throws(
      () =>
        restoreSourceCli(value.consumer, value.archive, value.hashes.sha256, {
          ...value.identity,
          [field]: "foreign",
        }),
      /identity changed/,
    );
    assert.equal(fs.existsSync(path.join(value.consumer, "target")), false);
  }
  assert.throws(
    () =>
      restoreSourceCli(value.consumer, value.archive, value.hashes.sha256, {
        ...value.identity,
        flags: { RUSTFLAGS: "foreign" },
      }),
    /identity changed/,
  );
});

law("complete archive hash rejects same-size preserved-time mutation", (value) => {
  const stat = fs.statSync(value.archive);
  const bytes = fs.readFileSync(value.archive);
  bytes[80] ^= 1;
  fs.writeFileSync(value.archive, bytes);
  fs.utimesSync(value.archive, stat.atime, stat.mtime);
  assert.equal(fs.statSync(value.archive).size, stat.size);
  assert.throws(
    () => restoreSourceCli(value.consumer, value.archive, value.hashes.sha256, value.identity),
    /digest changed/,
  );
});

for (const field of ["recipe", "oracleRecipe"] as const) {
  law(`a recomputed outer hash does not authorize foreign ${field}`, (value) => {
    const sha = changeEnvelope(value, (envelope) => {
      envelope[field] = "cargo build --release";
    });
    assert.throws(() => restoreSourceCli(value.consumer, value.archive, sha, value.identity));
    assert.equal(fs.existsSync(path.join(value.consumer, "target")), false);
  });
}
for (const member of CLI_MEMBERS) {
  law(`a recomputed outer hash refuses changed bytes of ${member}`, (value) => {
    const sha = rebuild(value, (stage) => {
      const filename = path.join(stage, member);
      const bytes = fs.readFileSync(filename);
      bytes[0] ^= 1;
      fs.writeFileSync(filename, bytes);
    });
    assert.throws(() => restoreSourceCli(value.consumer, value.archive, sha, value.identity));
    assert.equal(fs.existsSync(path.join(value.consumer, "target")), false);
  });
}
law("mode change cannot inherit the producer receipt", (value) => {
  const sha = rebuild(value, (stage) => {
    fs.chmodSync(path.join(stage, CLI_MEMBERS[0]), 0o644);
  });
  assert.throws(() => restoreSourceCli(value.consumer, value.archive, sha, value.identity));
});
law("symbolic archive links refuse even with a new outer hash", (value) => {
  const sha = rebuild(value, (stage) => {
    fs.unlinkSync(path.join(stage, CLI_MEMBERS[2]));
    fs.symlinkSync(path.join(value.producer, CLI_MEMBERS[2]), path.join(stage, CLI_MEMBERS[2]));
  });
  assert.throws(
    () => restoreSourceCli(value.consumer, value.archive, sha, value.identity),
    /regular members/,
  );
});
law("hard archive aliases cannot replace either independently hashed executable", (value) => {
  const sha = rebuild(value, (stage) => {
    fs.unlinkSync(path.join(stage, CLI_MEMBERS[2]));
    fs.linkSync(path.join(stage, CLI_MEMBERS[0]), path.join(stage, CLI_MEMBERS[2]));
  });
  assert.throws(
    () => restoreSourceCli(value.consumer, value.archive, sha, value.identity),
    /regular members/,
  );
});
law("missing archive or member refuses without falling back to a local CLI", (value) => {
  assert.throws(() =>
    restoreSourceCli(
      value.consumer,
      `${value.archive}.absent`,
      value.hashes.sha256,
      value.identity,
    ),
  );
  command(value.producer, "tar", ["-czf", value.archive, ...CLI_MEMBERS]);
  assert.throws(() =>
    restoreSourceCli(
      value.consumer,
      value.archive,
      digest(fs.readFileSync(value.archive)),
      value.identity,
    ),
  );
});
law("source and destination directory links do not escape physical custody", (value) => {
  fs.symlinkSync(value.producer, path.join(value.consumer, "target"));
  assert.throws(
    () => restoreSourceCli(value.consumer, value.archive, value.hashes.sha256, value.identity),
    /physical parent/,
  );
  fs.unlinkSync(path.join(value.consumer, "target"));
  fs.renameSync(path.join(value.producer, "target"), path.join(value.base, "foreign-target"));
  fs.symlinkSync(path.join(value.base, "foreign-target"), path.join(value.producer, "target"));
  assert.throws(
    () => packSourceCli(value.producer, value.archive, value.identity),
    /regular member/,
  );
});
law("actual checkout and Cargo version remain independent consumer authorities", (value) => {
  fs.writeFileSync(
    path.join(value.consumer, "Cargo.toml"),
    '[workspace.package]\nversion = "0.2.0"\n',
  );
  assert.throws(
    () => restoreSourceCli(value.consumer, value.archive, value.hashes.sha256, value.identity),
    /cliVersion/,
  );
  command(value.consumer, "git", ["add", "Cargo.toml"]);
  command(value.consumer, "git", [
    "-c",
    "user.name=Transport law",
    "-c",
    "user.email=transport@example.invalid",
    "commit",
    "--quiet",
    "-m",
    "foreign source",
  ]);
  assert.throws(
    () => restoreSourceCli(value.consumer, value.archive, value.hashes.sha256, value.identity),
    /sourceRevision/,
  );
});
law(
  "shared oracle boundary preserves producer provenance and refuses changed loaded bytes",
  (value) => {
    restoreSourceCli(value.consumer, value.archive, value.hashes.sha256, value.identity);
    const fields = [
      "GITHUB_SHA",
      "GITHUB_RUN_ID",
      "GITHUB_RUN_ATTEMPT",
      "VIZE_SOURCE_CLI_RECEIPT_SHA256",
    ];
    const before = fields.map((field) => process.env[field]);
    try {
      process.env.GITHUB_SHA = value.identity.sourceRevision;
      process.env.GITHUB_RUN_ID = value.identity.runId;
      process.env.GITHUB_RUN_ATTEMPT = value.identity.runAttempt;
      process.env.VIZE_SOURCE_CLI_RECEIPT_SHA256 = value.hashes.receiptSha256;
      const executable = path.join(value.consumer, CLI_MEMBERS[2]);
      assert.deepEqual(sharedOracleAuthority(value.consumer, executable)?.identity, value.identity);
      const bytes = fs.readFileSync(executable);
      bytes[0] ^= 1;
      fs.writeFileSync(executable, bytes);
      assert.throws(() => sharedOracleAuthority(value.consumer, executable), /identity changed/);
      fs.writeFileSync(path.join(value.consumer, "target/ci/vize.source-build.json"), "{}");
      assert.throws(() => sharedOracleAuthority(value.consumer, executable), /receipt changed/);
    } finally {
      fields.forEach((field, index) => {
        if (before[index] === undefined) delete process.env[field];
        else process.env[field] = before[index];
      });
    }
  },
);
