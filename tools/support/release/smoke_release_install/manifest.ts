import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { assertExecutable } from "./process.ts";

export type Manifest = Record<string, unknown> & { name: string; version: string };
export interface PackageInfo {
  compatible: boolean;
  name: string;
  tarball: string;
  version: string;
}
export const DEPENDENCY_SECTIONS = [
  "dependencies",
  "optionalDependencies",
  "peerDependencies",
  "devDependencies",
] as const;
export function object(value: unknown): Record<string, unknown> | undefined {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}
export function readJson(filename: string): unknown {
  let text: string;
  try {
    // Rust read_to_string refuses invalid UTF-8 and preserves a leading BOM.
    text = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(
      fs.readFileSync(filename),
    );
  } catch (error) {
    throw new Error(
      `cannot read ${filename}: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
  try {
    // serde_json rejects lone UTF-16 surrogates and overflowing JSON numbers.
    // Keep that input domain while retaining native parser diagnostics.
    const loneSurrogate = /[\uD800-\uDBFF](?![\uDC00-\uDFFF])|(?<![\uD800-\uDBFF])[\uDC00-\uDFFF]/;
    return JSON.parse(text, (key: string, value: unknown): unknown => {
      if (loneSurrogate.test(key) || (typeof value === "string" && loneSurrogate.test(value)))
        throw new SyntaxError("JSON strings must contain valid Unicode scalar values");
      if (typeof value === "number" && !Number.isFinite(value))
        throw new SyntaxError("JSON number is out of range");
      return value;
    }) as unknown;
  } catch (error) {
    throw new Error(
      `cannot parse JSON ${filename}: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}
export function readPackageJson(directory: string): Manifest {
  const filename = path.join(directory, "package.json"),
    value = object(readJson(filename));
  if (value?.private === true) throw new Error(`${filename} must be publishable`);
  if (typeof value?.name !== "string") throw new Error(`${filename} is missing name`);
  if (typeof value.version !== "string") throw new Error(`${filename} is missing version`);
  return value as Manifest;
}
export function assertNoWorkspaceProtocols(directory: string, manifest: Manifest) {
  const unresolved: string[] = [];
  for (const section of DEPENDENCY_SECTIONS) {
    const dependencies = object(manifest[section]);
    if (!dependencies) continue;
    for (const name of Object.keys(dependencies).sort()) {
      const version = dependencies[name];
      if (
        typeof version === "string" &&
        (version.startsWith("workspace:") || version.startsWith("catalog:"))
      ) {
        unresolved.push(`${section}.${name}=${version}`);
      }
    }
  }
  if (unresolved.length)
    throw new Error(
      `${path.join(directory, "package.json")} is not publishable: ${unresolved.join(", ")}`,
    );
}
function collectStrings(value: unknown, out: string[]) {
  if (typeof value === "string") out.push(value);
  else if (Array.isArray(value)) for (const item of value) collectStrings(item, out);
  else if (value !== null && typeof value === "object")
    for (const item of Object.values(value)) collectStrings(item, out);
}
export function assertPublishEntrypointsExist(directory: string, manifest: Manifest) {
  const entries: string[] = [];
  for (const field of ["main", "types", "bin", "exports"]) collectStrings(manifest[field], entries);
  const missing: string[] = [];
  for (const entry of [...new Set(entries)].sort()) {
    const trimmed = entry.trim();
    if (!trimmed || trimmed.startsWith("#") || /^[A-Za-z][A-Za-z0-9+.-]*:/.test(trimmed)) continue;
    const relative = trimmed.startsWith("./") ? trimmed.slice(2) : trimmed;
    try {
      fs.statSync(path.resolve(directory, relative));
    } catch {
      missing.push(entry);
    }
  }
  if (missing.length)
    throw new Error(`${manifest.name} publishes missing entrypoint files: ${missing.join(", ")}`);
}
function npmAllows(value: unknown, current: string): boolean {
  if (!Array.isArray(value)) return true;
  const strings = value.filter((item): item is string => typeof item === "string");
  if (strings.includes(`!${current}`)) return false;
  const positives = strings.filter((item) => !item.startsWith("!"));
  return !positives.length || positives.includes(current);
}
function currentLibc(): string {
  if (process.platform !== "linux") return "";
  const output = spawnSync("ldd", ["--version"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (output.error) return "";
  return `${output.stdout}\n${output.stderr}`.toLowerCase().includes("musl") ? "musl" : "glibc";
}
export function isCompatibleWithCurrentRunner(manifest: Manifest): boolean {
  return (
    npmAllows(manifest.os, process.platform) &&
    npmAllows(manifest.cpu, process.arch) &&
    npmAllows(manifest.libc, currentLibc())
  );
}
export function isPlatformSpecific(manifest: Manifest): boolean {
  return Array.isArray(manifest.os) || Array.isArray(manifest.cpu);
}
export function installedPackageDir(nodeModules: string, name: string): string {
  return path.join(nodeModules, name);
}
export function assertInstalledPackage(nodeModules: string, info: PackageInfo) {
  const directory = installedPackageDir(nodeModules, info.name),
    manifest = readPackageJson(directory);
  if (manifest.name !== info.name || manifest.version !== info.version)
    throw new Error(
      `installed package mismatch for ${info.name}: ${manifest.name}@${manifest.version}`,
    );
  assertNoWorkspaceProtocols(directory, manifest);
  if (!isPlatformSpecific(manifest)) assertPublishEntrypointsExist(directory, manifest);
  if (process.platform !== "win32") {
    const bins =
      typeof manifest.bin === "string"
        ? [info.name]
        : Object.keys(object(manifest.bin) ?? {}).sort();
    for (const name of bins) {
      const filename = path.join(nodeModules, ".bin", name);
      if (!fs.existsSync(filename)) throw new Error(`${info.name} did not install ${name}`);
      assertExecutable(filename, `${info.name} installed non-executable ${name}`);
    }
  }
}
export function resolveInstalledBin(
  installDir: string,
  packageName: string,
  binName: string,
): string {
  const directory = installedPackageDir(path.join(installDir, "node_modules"), packageName);
  const manifest = readPackageJson(directory),
    bins = object(manifest.bin);
  const relative =
    typeof manifest.bin === "string"
      ? manifest.bin
      : (bins?.[binName] ?? bins?.[packageName.split("/").at(-1) ?? packageName]);
  if (typeof relative !== "string")
    throw new Error(`installed ${packageName} does not expose a "${binName}" bin entry`);
  return path.resolve(directory, relative);
}
