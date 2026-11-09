/**
 * Lays out napi platform packages for the native binaries this job built.
 *
 * Usage: node tools/support/github/stage-built-native-packages.ts <package-dir>
 *
 * Smoke and conformance jobs build only their host binding, while
 * `napi pre-publish` rejects every configured platform package that lacks its
 * binary. Every platform manifest is still created, so the root package keeps
 * its full optionalDependencies and Yarn still finds packed foreign manifests.
 * Each built binary is copied into its own package, and napi's release
 * validator checks exactly those packages without rewriting any manifest.
 * release.yml lays out all binaries and runs the full `pre-publish`.
 */
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";

import {
  commandArguments,
  isEntrypoint,
  printLine,
  runCommand,
  runEntrypoint,
} from "../command-runtime.ts";

const usage = "Usage: node tools/support/github/stage-built-native-packages.ts <package-dir>";

export interface BuiltTarget {
  readonly triple: string;
  readonly platform: string;
  readonly binary: string;
}

interface NapiApi {
  readonly parseTriple: (triple: string) => { platformArchABI: string };
}

/** Pairs each built `<binaryName>.<platform>.node` with its configured target. */
export function selectBuiltTargets(
  binaryName: string,
  targets: readonly string[],
  files: readonly string[],
  platformOf: (triple: string) => string,
): BuiltTarget[] {
  const prefix = `${binaryName}.`;
  const binaries = new Set(
    files.filter((file) => file.startsWith(prefix) && file.endsWith(".node")),
  );
  const built = targets.flatMap((triple) => {
    const platform = platformOf(triple);
    const binary = `${prefix}${platform}.node`;
    return binaries.delete(binary) ? [{ triple, platform, binary }] : [];
  });
  if (binaries.size > 0)
    throw new Error(`No configured napi target for ${[...binaries].sort().join(", ")}`);
  if (built.length === 0) throw new Error(`No ${prefix}<platform>.node binary was built`);
  return built;
}

function readNapiConfig(manifestPath: string): { binaryName: string; targets: string[] } {
  const manifest: unknown = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  const napi = (manifest as { napi?: { binaryName?: unknown; targets?: unknown } }).napi;
  const targets = napi?.targets;
  if (
    typeof napi?.binaryName !== "string" ||
    !Array.isArray(targets) ||
    !targets.every((target) => typeof target === "string")
  )
    throw new Error(`${manifestPath} must declare napi.binaryName and napi.targets`);
  return { binaryName: napi.binaryName, targets };
}

export function main(args: readonly string[]): number {
  if (args.length !== 1) {
    printLine(usage);
    return 1;
  }
  const packageDir = path.resolve(args[0]!);
  const manifestPath = path.join(packageDir, "package.json");
  const { binaryName, targets } = readNapiConfig(manifestPath);
  const require = createRequire(manifestPath);
  const cliManifestPath = require.resolve("@napi-rs/cli/package.json");
  const cliManifest = JSON.parse(fs.readFileSync(cliManifestPath, "utf8")) as {
    bin: { napi: string };
  };
  const cli = path.join(path.dirname(cliManifestPath), cliManifest.bin.napi);
  const { parseTriple } = require("@napi-rs/cli") as NapiApi;
  const napi = (...napiArgs: string[]): void => {
    const status = runCommand(process.execPath, [cli, ...napiArgs], { cwd: packageDir });
    if (status !== 0) throw new Error(`napi ${napiArgs[0]} exited with status ${status}`);
  };

  napi("create-npm-dirs");
  const built = selectBuiltTargets(
    binaryName,
    targets,
    fs.readdirSync(packageDir),
    (triple) => parseTriple(triple).platformArchABI,
  );
  for (const { platform, binary } of built) {
    fs.copyFileSync(path.join(packageDir, binary), path.join(packageDir, "npm", platform, binary));
    printLine(`Staged ${binary} into npm/${platform}`);
  }
  const configDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-napi-built-targets-"));
  try {
    const configPath = path.join(configDir, "napi.json");
    fs.writeFileSync(configPath, `${JSON.stringify({ targets: built.map((t) => t.triple) })}\n`);
    napi(
      "pre-publish",
      "--config-path",
      configPath,
      "-t",
      "npm",
      "--no-gh-release",
      "--skip-optional-publish",
      "--dry-run",
    );
  } finally {
    fs.rmSync(configDir, { recursive: true, force: true });
  }
  return 0;
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => main(commandArguments()));
