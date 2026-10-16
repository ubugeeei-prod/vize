import { appendFileSync } from "node:fs";
import { isEntrypoint, runEntrypoint } from "../command-runtime.ts";

export interface ReleaseVersion {
  major: number;
  minor: number;
  patch: number;
}
export interface CliPlatform {
  host: string;
  target: string;
  archive: string;
}
export interface NativePlatform {
  host: string;
  target: string;
  cross_compile: boolean;
}

// The GNU builds retain Ubuntu 22.04's glibc floor. Windows ARM64 is built
// with an explicit amd64_arm64 MSVC environment on the faster x64 runner.
export const cliReleasePlatforms: CliPlatform[] = [
  {
    host: "blacksmith-12vcpu-macos-15",
    target: "x86_64-apple-darwin",
    archive: "vize-x86_64-apple-darwin.tar.gz",
  },
  {
    host: "blacksmith-12vcpu-macos-15",
    target: "aarch64-apple-darwin",
    archive: "vize-aarch64-apple-darwin.tar.gz",
  },
  {
    host: "blacksmith-32vcpu-windows-2025",
    target: "x86_64-pc-windows-msvc",
    archive: "vize-x86_64-pc-windows-msvc.zip",
  },
  {
    host: "blacksmith-32vcpu-windows-2025",
    target: "aarch64-pc-windows-msvc",
    archive: "vize-aarch64-pc-windows-msvc.zip",
  },
  {
    host: "blacksmith-32vcpu-ubuntu-2404",
    target: "x86_64-unknown-linux-gnu",
    archive: "vize-x86_64-unknown-linux-gnu.tar.gz",
  },
  {
    host: "blacksmith-32vcpu-ubuntu-2404",
    target: "x86_64-unknown-linux-musl",
    archive: "vize-x86_64-unknown-linux-musl.tar.gz",
  },
  {
    host: "blacksmith-32vcpu-ubuntu-2404",
    target: "aarch64-unknown-linux-gnu",
    archive: "vize-aarch64-unknown-linux-gnu.tar.gz",
  },
  {
    host: "blacksmith-32vcpu-ubuntu-2404",
    target: "aarch64-unknown-linux-musl",
    archive: "vize-aarch64-unknown-linux-musl.tar.gz",
  },
];

export const nativeReleasePlatforms: NativePlatform[] = [
  { host: "blacksmith-12vcpu-macos-15", target: "x86_64-apple-darwin", cross_compile: false },
  { host: "blacksmith-12vcpu-macos-15", target: "aarch64-apple-darwin", cross_compile: false },
  {
    host: "blacksmith-32vcpu-windows-2025",
    target: "x86_64-pc-windows-msvc",
    cross_compile: false,
  },
  {
    host: "blacksmith-32vcpu-windows-2025",
    target: "aarch64-pc-windows-msvc",
    cross_compile: false,
  },
  {
    host: "blacksmith-32vcpu-ubuntu-2204",
    target: "x86_64-unknown-linux-gnu",
    cross_compile: false,
  },
  {
    host: "blacksmith-32vcpu-ubuntu-2404",
    target: "x86_64-unknown-linux-musl",
    cross_compile: true,
  },
  {
    host: "blacksmith-32vcpu-ubuntu-2204-arm",
    target: "aarch64-unknown-linux-gnu",
    cross_compile: false,
  },
  {
    host: "blacksmith-32vcpu-ubuntu-2404-arm",
    target: "aarch64-unknown-linux-musl",
    cross_compile: true,
  },
];

export function parseReleaseVersion(refName: string): ReleaseVersion {
  const match = /^v?(\d+)\.(\d+)\.(\d+)(?:-[0-9A-Za-z.-]+)?$/.exec(refName);
  if (!match)
    throw new Error(`Release tag must look like vMAJOR.MINOR.PATCH[-PRERELEASE], got ${refName}`);
  const [major, minor, patch] = match.slice(1).map(Number);
  if (![major, minor, patch].every(Number.isSafeInteger))
    throw new Error("Release version exceeds the safe integer range");
  return { major, minor, patch };
}

export function releasePlatformPlan(refName: string) {
  return {
    version: parseReleaseVersion(refName),
    includeSlowPlatforms: true,
    skippedTargets: [] as string[],
    cliMatrix: cliReleasePlatforms,
    nativeMatrix: nativeReleasePlatforms,
  };
}

// All shipping targets are published on every release. Keep the command for
// callers preparing manifests without rewriting any manifest or dropping a target.
export function applyReleasePlatformCadence(refName: string, _packageJsonPath: string) {
  parseReleaseVersion(refName);
  return { changed: false, skippedTargets: [] as string[] };
}

export function main(args: readonly string[]): number {
  const [command, refName, ...paths] = args;
  if (!refName) throw new Error("A release ref name is required");
  const plan = releasePlatformPlan(refName);
  if (command === "print") {
    console.log(JSON.stringify(plan, null, 2));
    return 0;
  }
  if (command === "apply-cadence" && paths.length > 0) {
    for (const packagePath of paths)
      console.log(`Skipped release platform cadence for ${packagePath}; skipped targets: none`);
    return 0;
  }
  if (command === "github-output") {
    const output = process.env.GITHUB_OUTPUT;
    if (!output) throw new Error("GITHUB_OUTPUT is required for github-output");
    appendFileSync(
      output,
      [
        "include_slow_platforms=true",
        `release_minor=${plan.version.minor}`,
        `cli_matrix=${JSON.stringify(plan.cliMatrix)}`,
        `native_matrix=${JSON.stringify(plan.nativeMatrix)}`,
        "",
      ].join("\n"),
    );
    if (process.env.GITHUB_STEP_SUMMARY)
      appendFileSync(
        process.env.GITHUB_STEP_SUMMARY,
        `All ${plan.cliMatrix.length} CLI and ${plan.nativeMatrix.length} native targets are included for ${refName}.\n`,
      );
    return 0;
  }
  throw new Error(
    "Usage: node tools/support/github/release-platforms.ts <github-output|apply-cadence|print> <ref-name> [...package-json]",
  );
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => main(process.argv.slice(2)));
