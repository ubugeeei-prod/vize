import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createHash } from "node:crypto";
import path from "node:path";
import os from "node:os";
import { sourceRevision } from "../../../../tests/differential/build-receipt.ts";

export const CLI_MEMBERS = [
  "target/ci/vize",
  "target/ci/vize.differential-build.json",
  "target/ci/examples/cli_build_result_oracle",
] as const;
export const ORACLE_RECIPE =
  "cargo build --profile ci -p vize_atelier_sfc --example cli_build_result_oracle";
export const TRANSPORT_MEMBER = "source-cli.json";
export const digest = (bytes: Buffer): string => createHash("sha256").update(bytes).digest("hex");

export function command(cwd: string, executable: string, args: string[]): string {
  const result = spawnSync(executable, args, { cwd, encoding: "utf8", timeout: 30_000 });
  assert.equal(result.error, undefined, `${executable}: ${result.error?.message}`);
  assert.equal(result.signal, null, `${executable} interrupted`);
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
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
  const source = sourceRevision(root);
  assert.equal(process.env.GITHUB_SHA, source, "checked source differs from workflow source");
  assert.match(process.env.GITHUB_RUN_ID ?? "", /^[1-9][0-9]*$/);
  assert.match(process.env.GITHUB_RUN_ATTEMPT ?? "", /^[1-9][0-9]*$/);
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
    sourceTree: command(root, "git", ["rev-parse", "HEAD^{tree}"]),
    runId: process.env.GITHUB_RUN_ID,
    runAttempt: process.env.GITHUB_RUN_ATTEMPT,
    platform: process.platform,
    arch: process.arch,
    rustc: command(root, "rustc", ["-vV"]),
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
  const producer = JSON.parse(fs.readFileSync(path.join(root, receiptPath), "utf8")) as {
    identity: ReturnType<typeof runtimeIdentity>;
    recipe: string;
    oracleRecipe: string;
    members: ReturnType<typeof regularMember>[];
  };
  assert.equal(producer.identity.sourceRevision, sourceRevision(root));
  assert.equal(producer.identity.sourceRevision, process.env.GITHUB_SHA);
  assert.equal(producer.identity.runId, process.env.GITHUB_RUN_ID);
  assert.equal(producer.identity.runAttempt, process.env.GITHUB_RUN_ATTEMPT);
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
