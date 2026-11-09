import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import { isEntrypoint, runEntrypoint } from "../../../support/command-runtime.ts";

interface PackageMapEntry {
  readonly url?: unknown;
}

interface PackageMap {
  readonly packages?: unknown;
}

interface PackageManifest {
  readonly name?: unknown;
  readonly bin?: unknown;
  readonly dependencies?: unknown;
  readonly devDependencies?: unknown;
  readonly optionalDependencies?: unknown;
}

function readJson<T>(filename: string): T {
  try {
    return JSON.parse(fs.readFileSync(filename, "utf8")) as T;
  } catch (error) {
    throw new Error(`failed to read or parse ${filename}: ${String(error)}`);
  }
}

function record(value: unknown): Record<string, unknown> | undefined {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}

export function isExactVersion(value: string): boolean {
  const [core] = value.split(/[-+]/u);
  if (core === undefined) return false;
  const parts = core.split(".");
  return parts.length === 3 && parts.every((part) => /^\d+$/u.test(part));
}

export function scorePackageId(
  id: string,
  packageName: string,
  expectedVersion: string | undefined,
): number {
  let score = 0;
  if (expectedVersion !== undefined && id.startsWith(`${packageName}@${expectedVersion}`))
    score += 2;
  if (!id.includes("(")) score += 1;
  return score;
}

function exactManifestVersion(cwd: string, packageName: string): string | undefined {
  const manifestPath = path.join(cwd, "package.json");
  if (!fs.existsSync(manifestPath)) return undefined;
  const manifest = readJson<PackageManifest>(manifestPath);
  for (const field of [
    manifest.dependencies,
    manifest.devDependencies,
    manifest.optionalDependencies,
  ]) {
    const value = record(field)?.[packageName];
    if (typeof value === "string" && isExactVersion(value)) return value;
  }
  return undefined;
}

export function resolvePackageRoot(cwd: string, packageName: string): string {
  const nodeModules = path.join(cwd, "node_modules");
  const direct = path.join(nodeModules, ...packageName.split("/"));
  if (fs.existsSync(path.join(direct, "package.json"))) return direct;

  const packageMapPath = path.join(nodeModules, ".package-map.json");
  if (!fs.existsSync(packageMapPath)) {
    throw new Error(`Cannot resolve ${packageName}: ${packageMapPath} does not exist`);
  }
  const packages = record(readJson<PackageMap>(packageMapPath).packages);
  if (packages === undefined) {
    throw new Error(`Cannot resolve ${packageName}: ${packageMapPath} has no packages object`);
  }
  const expectedVersion = exactManifestVersion(cwd, packageName);
  const candidates: Array<{ readonly path: string; readonly score: number }> = [];
  for (const [id, rawEntry] of Object.entries(packages)) {
    const entry = rawEntry as PackageMapEntry;
    if (typeof entry.url !== "string") continue;
    const normalizedUrl = entry.url.replaceAll("\\", "/");
    if (!normalizedUrl.endsWith(`/node_modules/${packageName}`)) continue;
    const packageRoot = path.join(nodeModules, entry.url);
    if (fs.existsSync(path.join(packageRoot, "package.json"))) {
      candidates.push({
        path: packageRoot,
        score: scorePackageId(id, packageName, expectedVersion),
      });
    }
  }
  candidates.sort((left, right) => right.score - left.score);
  const selected = candidates[0];
  if (selected === undefined) {
    throw new Error(
      `Cannot resolve ${packageName}: no matching package entry in ${packageMapPath}`,
    );
  }
  return selected.path;
}

export function resolveBinPath(manifest: PackageManifest, binName: string): string {
  if (typeof manifest.bin === "string") return manifest.bin;
  const relativePath = record(manifest.bin)?.[binName];
  if (typeof relativePath === "string") return relativePath;
  const packageName = typeof manifest.name === "string" ? manifest.name : "(unknown)";
  throw new Error(`Package ${packageName} does not expose bin "${binName}"`);
}

function main(args = process.argv.slice(2), cwd = process.cwd()): number {
  const [packageName, binName, ...binArgs] = args;
  if (packageName === undefined || binName === undefined) {
    throw new Error(
      "Usage: node tools/commands/editors/vscode/run-package-bin.ts <package-name> <bin-name> [args...]",
    );
  }
  const packageRoot = resolvePackageRoot(cwd, packageName);
  const relativeBin = resolveBinPath(
    readJson<PackageManifest>(path.join(packageRoot, "package.json")),
    binName,
  );
  const binPath = path.join(packageRoot, relativeBin);
  if (!fs.existsSync(binPath)) {
    throw new Error(`Package bin "${binName}" for ${packageName} does not exist at ${binPath}`);
  }

  const extension = path.extname(binPath).toLowerCase();
  const runWithNode = process.platform === "win32" && ![".bat", ".cmd", ".exe"].includes(extension);
  const result = spawnSync(
    runWithNode ? process.execPath : binPath,
    runWithNode ? [binPath, ...binArgs] : binArgs,
    {
      stdio: "inherit",
    },
  );
  if (result.error !== undefined) throw result.error;
  return result.status ?? 1;
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => main());
