import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const publicCorsaEnvironmentVariables = [
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
] as const;

type RuntimeResolutionOptions = {
  arch?: string;
  packageRoot?: string;
  platform?: NodeJS.Platform;
};

export function configureBundledCorsaRuntime(
  environment: NodeJS.ProcessEnv = process.env,
  options: RuntimeResolutionOptions = {},
): string | null {
  if (hasPublicRuntimeOverride(environment)) return null;

  const executable = resolveBundledCorsaRuntime(options);
  if (executable == null) return null;

  environment.CORSA_PATH = executable;
  return executable;
}

export function resolveBundledCorsaRuntime(options: RuntimeResolutionOptions = {}): string | null {
  const packageRoot =
    options.packageRoot ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const platform = options.platform ?? process.platform;
  const arch = options.arch ?? process.arch;

  try {
    const cliManifestPath = path.join(packageRoot, "package.json");
    const cliManifest = readManifest(cliManifestPath);
    const packageRequire = createRequire(cliManifestPath);

    const platformPackage = `@typescript/typescript-${platform}-${arch}`;
    const declaredBundledPlatformVersion = cliManifest.optionalDependencies?.[platformPackage];
    if (typeof declaredBundledPlatformVersion === "string") {
      const platformManifestPath = packageRequire.resolve(`${platformPackage}/package.json`);
      const platformManifest = readManifest(platformManifestPath);
      if (
        platformManifest.name !== platformPackage ||
        !isTypeScriptSevenOrNewer(platformManifest.version) ||
        !matchesDeclaredVersion(platformManifest.version, declaredBundledPlatformVersion)
      ) {
        return null;
      }
      return resolvePlatformExecutable(platformManifestPath, platform);
    }

    const declaredMetaVersion = cliManifest.optionalDependencies?.typescript;
    if (typeof declaredMetaVersion !== "string") return null;

    const metaManifestPath = packageRequire.resolve("typescript/package.json");
    const metaManifest = readManifest(metaManifestPath);
    if (
      metaManifest.name !== "typescript" ||
      !isTypeScriptSevenOrNewer(metaManifest.version) ||
      !matchesDeclaredVersion(metaManifest.version, declaredMetaVersion)
    ) {
      return null;
    }

    const declaredPlatformVersion = metaManifest.optionalDependencies?.[platformPackage];
    if (typeof declaredPlatformVersion !== "string") return null;

    const metaRequire = createRequire(metaManifestPath);
    const platformManifestPath = metaRequire.resolve(`${platformPackage}/package.json`);
    const platformManifest = readManifest(platformManifestPath);
    if (
      platformManifest.name !== platformPackage ||
      !matchesDeclaredVersion(platformManifest.version, declaredPlatformVersion)
    ) {
      return null;
    }

    return resolvePlatformExecutable(platformManifestPath, platform);
  } catch {
    // The runtime is optional. Existing Rust-side discovery remains the fallback.
    return null;
  }
}

function resolvePlatformExecutable(
  platformManifestPath: string,
  platform: NodeJS.Platform,
): string | null {
  const executable = path.join(
    path.dirname(platformManifestPath),
    "lib",
    platform === "win32" ? "tsc.exe" : "tsc",
  );
  return fs.existsSync(executable) ? executable : null;
}

function hasPublicRuntimeOverride(environment: NodeJS.ProcessEnv): boolean {
  return publicCorsaEnvironmentVariables.some((name) => {
    const value = environment[name];
    return value != null && value !== "";
  });
}

function matchesDeclaredVersion(actual: unknown, declared: string): boolean {
  return typeof actual === "string" && (declared.startsWith("catalog:") || actual === declared);
}

function isTypeScriptSevenOrNewer(version: unknown): boolean {
  if (typeof version !== "string") return false;
  const major = Number.parseInt(version.split(".")[0] ?? "", 10);
  return Number.isFinite(major) && major >= 7;
}

function readManifest(filename: string): {
  name?: string;
  optionalDependencies?: Record<string, string>;
  version?: string;
} {
  return JSON.parse(fs.readFileSync(filename, "utf8"));
}
