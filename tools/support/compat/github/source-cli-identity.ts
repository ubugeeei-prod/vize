import assert from "node:assert/strict";
import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import fs from "node:fs";
import { createHash } from "node:crypto";
import path from "node:path";
import os from "node:os";
import { BUILD_RECIPE } from "../../../../tests/differential/build-receipt.ts";

export const CLI_MEMBERS = [
  "target/ci/vize",
  "target/ci/vize.differential-build.json",
  "target/ci/examples/cli_build_result_oracle",
] as const;
export const ORACLE_RECIPE =
  "cargo build --profile ci -p vize_atelier_sfc --example cli_build_result_oracle";
export const TRANSPORT_MEMBER = "source-cli.json";
export const digest = (bytes: Buffer): string => createHash("sha256").update(bytes).digest("hex");

type CommandRunner = (
  executable: string,
  args: string[],
  options: { cwd: string; encoding: "utf8"; timeout: number },
) => SpawnSyncReturns<string>;

function checkedCommand(executable: string, result: SpawnSyncReturns<string>): string {
  assert.equal(result.error, undefined, `${executable}: ${result.error?.message}`);
  assert.equal(result.signal, null, `${executable} interrupted`);
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}

export function command(cwd: string, executable: string, args: string[]): string {
  const result = spawnSync(executable, args, { cwd, encoding: "utf8", timeout: 30_000 });
  return checkedCommand(executable, result);
}

export function rustcIdentity(cwd: string, run: CommandRunner = spawnSync): string {
  let result: SpawnSyncReturns<string> | undefined;
  for (const timeout of [30_000, 60_000]) {
    result = run("rustc", ["-vV"], { cwd, encoding: "utf8", timeout });
    if ((result.error as NodeJS.ErrnoException | undefined)?.code !== "ETIMEDOUT") break;
  }
  assert.ok(result);
  return checkedCommand("rustc", result);
}

export function normalizeCargoConfig(config: string, linker: string): string {
  assert.equal(path.resolve(linker), linker, "owned linker path must be absolute");
  const target = config.match(/\[target\.x86_64-unknown-linux-gnu\]\n([\s\S]*?)(?=\n\[|$)/);
  assert.ok(target, "host GNU linker configuration is required");
  assert.match(target[1], /^linker = "clang"$/m);
  assert.ok(
    target[1].includes(`"-Clink-arg=--ld-path=${linker}"`),
    "Cargo must select the owned Wild linker",
  );
  return config.replaceAll(linker, "$RUNNER_TEMP/wild-install/wild");
}

export function runtimeIdentity(root: string) {
  const checkout = sourceCliCheckout(root);
  const source = checkout.sourceRevision;
  assert.equal(process.env.GITHUB_SHA, source, "checked source differs from workflow source");
  const runId = process.env.GITHUB_RUN_ID ?? "";
  const runAttempt = process.env.GITHUB_RUN_ATTEMPT ?? "";
  assert.match(runId, /^[1-9][0-9]*$/);
  assert.match(runAttempt, /^[1-9][0-9]*$/);
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  assert.ok(process.env.RUNNER_TEMP, "owned linker installation is required");
  const linkerRoot = fs.realpathSync(path.join(process.env.RUNNER_TEMP, "wild-install"));
  const linker = path.join(linkerRoot, "wild");
  const cargoHome = process.env.CARGO_HOME ?? path.join(os.homedir(), ".cargo");
  const cargoConfig = fs.readFileSync(path.join(cargoHome, "config.toml"), "utf8");
  const configuredLinker = path.join(process.env.RUNNER_TEMP, "wild-install", "wild");
  return {
    sourceRevision: source,
    sourceTree: checkout.sourceTree,
    runId,
    runAttempt,
    platform: process.platform,
    arch: process.arch,
    rustc: rustcIdentity(root),
    cargo: command(root, "cargo", ["--version"]),
    linker: command(root, linker, ["--version"]),
    linkerSha256: regularMember(linkerRoot, "wild").sha256,
    driver: command(root, "clang", ["--version"]),
    cargoConfig: normalizeCargoConfig(cargoConfig, configuredLinker),
    flags: Object.fromEntries(
      Object.entries(process.env)
        .filter(([key]) =>
          /^(RUSTFLAGS|RUSTC(?:_.*)?|RUSTUP_TOOLCHAIN|CARGO_(ENCODED_RUSTFLAGS|PROFILE_CI_.*|BUILD_.*|TARGET_.*|TARGET_DIR))$/.test(
            key,
          ),
        )
        .sort(([a], [b]) => a.localeCompare(b)),
    ),
  };
}

export function regularMember(root: string, relative: string) {
  assert.equal(fs.realpathSync(root), path.resolve(root), "member root must be physical");
  const parts = relative.split("/");
  assert.ok(parts.every((part) => part !== "" && part !== "." && part !== ".."));
  for (let index = 1; index <= parts.length; index++) {
    const stat = fs.lstatSync(path.join(root, ...parts.slice(0, index)));
    assert.ok(
      index === parts.length ? stat.isFile() : stat.isDirectory(),
      `${relative}: regular member required`,
    );
  }
  const filename = path.join(root, relative);
  const stat = fs.lstatSync(filename);
  return {
    path: relative,
    bytes: stat.size,
    mode: stat.mode & 0o7777,
    sha256: digest(fs.readFileSync(filename)),
  };
}

export type SourceCliEnvelope = {
  identity: ReturnType<typeof runtimeIdentity>;
  recipe: string;
  oracleRecipe: string;
  members: ReturnType<typeof regularMember>[];
};
export const CONSUMER_USE_MEMBER = "target/ci/vize.source-use.json";
export type SourceCliConsumerContext = Pick<
  ReturnType<typeof runtimeIdentity>,
  "sourceRevision" | "sourceTree" | "runId" | "runAttempt"
>;

export function sourceCliCheckout(root: string) {
  const git = (...args: string[]) => command(root, "git", ["--no-replace-objects", ...args]);
  return {
    sourceRevision: git("rev-parse", "HEAD"),
    sourceTree: git("rev-parse", "HEAD^{tree}"),
  };
}

export function sourceCliProducerOutputs(serialized: string, consumerAttempt: string) {
  const outputs = JSON.parse(serialized) as Record<string, string>;
  assert.deepEqual(Object.keys(outputs).sort(), ["producer-attempt", "receipt-sha256", "sha256"]);
  assert.match(outputs.sha256, /^[a-f0-9]{64}$/);
  assert.match(outputs["receipt-sha256"], /^[a-f0-9]{64}$/);
  for (const attempt of [outputs["producer-attempt"], consumerAttempt])
    assert.match(attempt, /^[1-9][0-9]{0,9}$/);
  assert.ok(
    BigInt(outputs["producer-attempt"]) <= BigInt(consumerAttempt),
    "source CLI producer identity changed: future producer attempt",
  );
  return outputs;
}

export function expectedProducerIdentity(
  consumer: ReturnType<typeof runtimeIdentity>,
  producerAttempt = consumer.runAttempt,
) {
  for (const attempt of [producerAttempt, consumer.runAttempt])
    assert.match(
      attempt,
      /^[1-9][0-9]{0,9}$/,
      "source CLI producer identity changed: canonical finite attempt is required",
    );
  assert.ok(
    BigInt(producerAttempt) <= BigInt(consumer.runAttempt),
    "source CLI producer identity changed: future producer attempt",
  );
  return { ...consumer, runAttempt: producerAttempt };
}

// Restore validates all physical binary bodies. Later consumers authenticate
// this immutable use record and retain their own original member/body gates.
export function verifySourceCliConsumerUse(
  root: string,
  producer: SourceCliEnvelope,
  consumer: SourceCliConsumerContext,
  producerReceiptSha256: string,
  useReceiptSha256: string,
) {
  assert.match(producerReceiptSha256, /^[a-f0-9]{64}$/);
  assert.match(useReceiptSha256, /^[a-f0-9]{64}$/);
  const original = regularMember(root, "target/ci/vize.source-build.json");
  assert.equal(original.mode, 0o644);
  assert.equal(original.sha256, producerReceiptSha256, "shared producer receipt changed");
  assert.deepEqual(JSON.parse(fs.readFileSync(path.join(root, original.path), "utf8")), producer);
  const member = regularMember(root, CONSUMER_USE_MEMBER);
  assert.equal(member.mode, 0o644);
  assert.equal(member.sha256, useReceiptSha256, "consumer use receipt changed");
  const use = JSON.parse(fs.readFileSync(path.join(root, CONSUMER_USE_MEMBER), "utf8"));
  assert.deepEqual(
    Object.keys(use).sort(),
    [
      "archiveSha256",
      "consumerIdentity",
      "members",
      "oracleRecipe",
      "producerIdentity",
      "producerReceiptSha256",
      "recipe",
      "schema",
      "version",
    ].sort(),
  );
  assert.equal(use.schema, "vize.source-cli.consumer-use");
  assert.equal(use.version, 1);
  assert.match(use.archiveSha256, /^[a-f0-9]{64}$/);
  assert.equal(use.producerReceiptSha256, producerReceiptSha256);
  assert.deepEqual(use.producerIdentity, producer.identity);
  assert.deepEqual(use.consumerIdentity, { ...producer.identity, ...consumer });
  assert.deepEqual(
    producer.identity,
    expectedProducerIdentity(use.consumerIdentity, producer.identity.runAttempt),
  );
  const checkout = sourceCliCheckout(root);
  assert.equal(consumer.sourceRevision, checkout.sourceRevision);
  assert.equal(consumer.sourceTree, checkout.sourceTree);
  assert.equal(producer.identity.sourceRevision, consumer.sourceRevision);
  assert.equal(producer.identity.sourceTree, consumer.sourceTree);
  assert.equal(producer.identity.runId, consumer.runId);
  assert.equal(use.recipe, BUILD_RECIPE);
  assert.equal(use.recipe, producer.recipe);
  assert.equal(use.oracleRecipe, ORACLE_RECIPE);
  assert.equal(use.oracleRecipe, producer.oracleRecipe);
  assert.deepEqual(
    producer.members.map((entry) => entry.path),
    CLI_MEMBERS,
  );
  assert.deepEqual(
    producer.members.map((entry) => entry.mode),
    [0o755, 0o644, 0o755],
  );
  for (const entry of producer.members) {
    assert.ok(Number.isSafeInteger(entry.bytes) && entry.bytes > 0);
    assert.match(entry.sha256, /^[a-f0-9]{64}$/);
  }
  assert.deepEqual(use.members, producer.members);
  return use;
}

export function physicalParent(root: string, relative: string): void {
  assert.equal(fs.realpathSync(root), path.resolve(root), "destination root must be physical");
  const parts = relative.split("/");
  for (let index = 1; index < parts.length; index++) {
    const directory = path.join(root, ...parts.slice(0, index));
    if (!fs.existsSync(directory)) fs.mkdirSync(directory);
    assert.ok(fs.lstatSync(directory).isDirectory(), `${relative}: physical parent required`);
  }
  const destination = path.join(root, relative);
  const stat = fs.lstatSync(destination, { throwIfNoEntry: false });
  if (stat) assert.ok(stat.isFile(), "regular destination required");
}

export function sharedOracleAuthority(root: string, executable: string) {
  const expected = process.env.VIZE_SOURCE_CLI_RECEIPT_SHA256;
  if (expected === undefined) return undefined;
  assert.match(expected, /^[a-f0-9]{64}$/);
  const receiptPath = "target/ci/vize.source-build.json";
  assert.equal(
    regularMember(root, receiptPath).sha256,
    expected,
    "shared producer receipt changed",
  );
  const producer = JSON.parse(
    fs.readFileSync(path.join(root, receiptPath), "utf8"),
  ) as SourceCliEnvelope;
  const checkout = sourceCliCheckout(root);
  assert.equal(producer.identity.sourceRevision, checkout.sourceRevision);
  assert.equal(producer.identity.sourceRevision, process.env.GITHUB_SHA);
  assert.equal(producer.identity.runId, process.env.GITHUB_RUN_ID);
  const useSha256 = process.env.VIZE_SOURCE_CLI_USE_RECEIPT_SHA256;
  if (producer.identity.runAttempt !== process.env.GITHUB_RUN_ATTEMPT || useSha256 !== undefined) {
    assert.ok(useSha256, "retry requires a verified consumer use receipt");
    verifySourceCliConsumerUse(
      root,
      producer,
      {
        sourceRevision: checkout.sourceRevision,
        sourceTree: checkout.sourceTree,
        runId: process.env.GITHUB_RUN_ID!,
        runAttempt: process.env.GITHUB_RUN_ATTEMPT!,
      },
      expected,
      useSha256,
    );
  } else assert.equal(producer.identity.runAttempt, process.env.GITHUB_RUN_ATTEMPT);
  assert.equal(producer.oracleRecipe, ORACLE_RECIPE);
  const relative = path.relative(root, executable);
  assert.equal(relative, CLI_MEMBERS[2]);
  assert.deepEqual(
    producer.members.find((member) => member.path === relative),
    regularMember(root, relative),
    "shared SFC oracle identity changed",
  );
  return producer;
}
