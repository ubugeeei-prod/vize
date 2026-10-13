/**
 * Builds one N-API package for a release target.
 *
 * Usage: node tools/support/github/build-napi-package.ts
 *   <package-dir> <manifest-path> <package-name> <target> [--cross-compile]
 *
 * Apple targets build the dynamic library with Cargo and copy it to the
 * package's platform `.node` file. Other targets run `vp exec napi build`.
 * Every native build receives a fresh 128-bit `VIZE_NATIVE_BUILD_TOKEN` that
 * overrides any inherited token while the rest of the environment is kept.
 */
import { randomBytes } from "node:crypto";
import fs from "node:fs";

import {
  type CommandEnvironment,
  commandArguments,
  isEntrypoint,
  isRegularFile,
  joinPath,
  printLine,
  runCommand,
  runEntrypoint,
  writeFileDurably,
} from "../command-runtime.ts";

const usage =
  "Usage: node tools/support/github/build-napi-package.ts <package-dir> <manifest-path> <package-name> <target> [--cross-compile]";

export type EntropySource = (size: number) => Uint8Array | undefined;

export function binaryName(crateName: string): string {
  return crateName.split("_").join("-");
}

export function napiFeatures(crateName: string): string {
  return crateName === "vize_vitrine" ? "napi,legacy" : "napi";
}

export function isAppleTarget(target: string): boolean {
  return target.endsWith("-apple-darwin");
}

export function applePlatformTag(target: string): string | undefined {
  switch (target) {
    case "aarch64-apple-darwin":
      return "darwin-arm64";
    case "x86_64-apple-darwin":
      return "darwin-x64";
    default:
      return undefined;
  }
}

export function builtAppleLibraryPath(
  packageDir: string,
  target: string,
  crateName: string,
): string {
  return joinPath(
    joinPath(joinPath(joinPath(packageDir, "../../target"), target), "release"),
    `lib${crateName}.dylib`,
  );
}

/** Encodes every byte as two lowercase hexadecimal digits. */
export function nativeBuildToken(random: Uint8Array): string {
  let token = "";
  for (const byte of random) token += byte.toString(16).padStart(2, "0");
  return token;
}

export function platformEntropy(size: number): Uint8Array | undefined {
  try {
    return randomBytes(size);
  } catch {
    return undefined;
  }
}

/** Overrides the inherited identity once for each actual native build invocation. */
export function nativeBuildEnvironment(
  env: CommandEnvironment = process.env,
  entropy: EntropySource = platformEntropy,
): CommandEnvironment | undefined {
  const random = entropy(16);
  if (random === undefined) {
    printLine("Could not allocate native build identity");
    return undefined;
  }
  return { ...env, VIZE_NATIVE_BUILD_TOKEN: nativeBuildToken(random) };
}

function buildApplePackage(
  packageDir: string,
  manifestPath: string,
  crateName: string,
  target: string,
): number {
  const env = nativeBuildEnvironment();
  if (env === undefined) return 1;
  const exitCode = runCommand(
    "cargo",
    [
      "build",
      "--release",
      "--manifest-path",
      manifestPath,
      "-p",
      crateName,
      "--features",
      napiFeatures(crateName),
      "--target",
      target,
    ],
    { cwd: packageDir, env },
  );
  if (exitCode !== 0) return exitCode;
  const platformTag = applePlatformTag(target);
  if (platformTag === undefined) {
    printLine(`Unsupported Apple target: ${target}`);
    return 1;
  }
  const builtLibrary = builtAppleLibraryPath(packageDir, target, crateName);
  if (!isRegularFile(builtLibrary)) {
    printLine(`Built library not found: ${builtLibrary}`);
    return 1;
  }
  const outputPath = joinPath(packageDir, `${binaryName(crateName)}.${platformTag}.node`);
  writeFileDurably(outputPath, fs.readFileSync(builtLibrary), 0o755);
  return 0;
}

export function main(args: readonly string[]): number {
  if (args.length < 4 || args.length > 5) {
    printLine(usage);
    return 1;
  }
  const [packageDir, manifestPath, packageName, target] = args as [
    string,
    string,
    string,
    string,
    ...string[],
  ];
  const crossCompile = args.length === 5 && args[4] === "--cross-compile";
  if (args.length === 5 && !crossCompile) {
    printLine("Unknown argument");
    return 1;
  }
  if (isAppleTarget(target)) {
    if (crossCompile) {
      printLine("Apple targets do not support --cross-compile in this helper.");
      return 1;
    }
    return buildApplePackage(packageDir, manifestPath, packageName, target);
  }
  const commandArgs = [
    "exec",
    "napi",
    "build",
    "--platform",
    "--release",
    "--manifest-path",
    manifestPath,
    "-p",
    packageName,
    "--features",
    napiFeatures(packageName),
    "--output-dir",
    ".",
    "--target",
    target,
  ];
  if (crossCompile) commandArgs.push("--cross-compile");
  const env = nativeBuildEnvironment();
  if (env === undefined) return 1;
  return runCommand("vp", commandArgs, { cwd: packageDir, env });
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => main(commandArguments()));
