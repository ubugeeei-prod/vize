// Compare the checked-out PR merge against its actual base parent.
// usage: node tools/commands/ci/github/check-semver.ts CRATE
import { spawnSync, type SpawnSyncOptions } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const breakingMarker = /^[A-Za-z0-9_-]+(\([^)]+\))?!:|^BREAKING CHANGE:/mu;

/** Run a command with inherited output; a failed command fails the check with its status. */
function run(command: string, args: readonly string[], options: SpawnSyncOptions = {}): number {
  const result = spawnSync(command, args, { stdio: "inherit", ...options });
  if (result.error) throw result.error;
  return result.status ?? 1;
}

function capture(command: string, args: readonly string[]): { status: number; stdout: string } {
  const result = spawnSync(command, args, {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  });
  if (result.error) throw result.error;
  return { status: result.status ?? 1, stdout: result.stdout.replace(/\n+$/u, "") };
}

function baselineRevision(): string {
  const configured = process.env.BASELINE_REV ?? "";
  // pull_request checkout is a merge commit. The event's base SHA can
  // predate commits that landed on the base before this job started.
  if (capture("git", ["rev-parse", "--verify", "-q", "HEAD^2"]).status === 0) {
    const parent = capture("git", ["rev-parse", "HEAD^1"]);
    if (parent.status !== 0) throw new Error("git rev-parse HEAD^1 failed");
    return parent.stdout;
  }
  return configured === "0".repeat(40) ? "" : configured;
}

function semverArgs(crate: string, runnerTemp: string, baselineDir: string): string[] | number {
  const baselineRev = baselineRevision();
  const marker = fs.readFileSync(path.join(runnerTemp, "semver-change-marker.txt"), "utf8");
  const args = breakingMarker.test(marker) ? ["--release-type", "major"] : [];
  const helper = ["tools/commands/ci/github/semver-baseline.rs", crate, baselineDir];
  const baseline = capture("rust-script", baselineRev ? [...helper, baselineRev] : helper);
  if (baseline.status !== 0) return baseline.status;
  if (baseline.stdout) args.push("--baseline-root", baseline.stdout);
  else if (baselineRev) args.push("--baseline-rev", baselineRev);
  return args;
}

function checkRelease(
  crate: string,
  runnerTemp: string,
  baselineDir: string,
  outputDir: string,
): number {
  const args = semverArgs(crate, runnerTemp, baselineDir);
  if (typeof args === "number") return args;
  const output = path.join(outputDir, "semver-output.txt");
  const sink = fs.openSync(output, "w");
  let status: number;
  try {
    status = run(
      "cargo",
      ["semver-checks", "check-release", "--package", crate, "--color", "never", ...args],
      { stdio: ["inherit", sink, sink] },
    );
  } finally {
    fs.closeSync(sink);
  }
  process.stdout.write(fs.readFileSync(output));
  if (status !== 0) {
    if (status !== 1 || crate !== "vize_armature") return status;
    const allowed = run(process.execPath, [
      "tools/commands/ci/github/allow-armature-tokenizer-reexports.ts",
      output,
    ]);
    if (allowed !== 0) return allowed;
  }
  return 0;
}

/**
 * The same source must execute as a separate crate against the published
 * 0.429.1 API and this candidate. This covers the re-exports rustdoc cannot
 * resolve for cargo-semver-checks (upstream issue #355). Share target objects
 * between both manifests and omit debug info to bound release-runner disk use.
 */
function armatureConsumers(runnerTemp: string): number {
  const env = {
    ...process.env,
    CARGO_TARGET_DIR:
      process.env.CARGO_TARGET_DIR || path.join(runnerTemp, "armature-tokenizer-consumer-target"),
    CARGO_PROFILE_DEV_DEBUG: "0",
    CARGO_PROFILE_TEST_DEBUG: "0",
  };
  const fixture = "tests/external-consumers/armature-tokenizer";
  const baseline = run(
    "cargo",
    ["test", "--lib", "--locked", "--manifest-path", `${fixture}/baseline/Cargo.toml`],
    { env },
  );
  if (baseline !== 0) return baseline;
  // This path dependency changes version with the release branch, so resolve
  // a fresh fixture lock rather than pinning it to the source version.
  return run("cargo", ["test", "--lib", "--manifest-path", `${fixture}/Cargo.toml`], { env });
}

function main(crate: string | undefined): number {
  const runnerTemp = process.env.RUNNER_TEMP;
  if (!crate || !runnerTemp) {
    process.stderr.write("usage: RUNNER_TEMP=DIR node check-semver.ts CRATE\n");
    return 2;
  }
  const baselineDir = fs.mkdtempSync(path.join(runnerTemp, "semver-baseline."));
  const outputDir = fs.mkdtempSync(path.join(runnerTemp, "semver-output."));
  try {
    const status = checkRelease(crate, runnerTemp, baselineDir, outputDir);
    if (status !== 0) return status;
  } finally {
    fs.rmSync(baselineDir, { recursive: true, force: true });
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
  return crate === "vize_armature" ? armatureConsumers(runnerTemp) : 0;
}

process.exitCode = main(process.argv[2]);
