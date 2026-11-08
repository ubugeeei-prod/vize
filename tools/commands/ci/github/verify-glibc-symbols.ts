import { spawnSync } from "node:child_process";
import fs from "node:fs";

import { isEntrypoint, runEntrypoint } from "../../../support/command-runtime.ts";

const defaultMaximum = "2.36";

export interface GlibcVersion {
  readonly major: number;
  readonly minor: number;
  readonly patch: number;
  readonly text: string;
}

export function compareGlibcVersions(left: GlibcVersion, right: GlibcVersion): number {
  return left.major - right.major || left.minor - right.minor || left.patch - right.patch;
}

export function parseGlibcVersion(value: string): GlibcVersion {
  const match = /^(\d+)\.(\d+)(?:\.(\d+))?$/u.exec(value);
  if (match === null) {
    throw new Error(`glibc version must look like MAJOR.MINOR[.PATCH], got ${value}`);
  }
  const major = Number(match[1]);
  const minor = Number(match[2]);
  const patch = match[3] === undefined ? 0 : Number(match[3]);
  return { major, minor, patch, text: `${major}.${minor}${patch === 0 ? "" : `.${patch}`}` };
}

export function parseGlibcVersions(text: string): GlibcVersion[] {
  const versions = new Map<string, GlibcVersion>();
  for (const match of text.matchAll(/\bGLIBC_(\d+\.\d+(?:\.\d+)?)\b/gu)) {
    const version = parseGlibcVersion(match[1]!);
    versions.set(version.text, version);
  }
  return [...versions.values()].sort(compareGlibcVersions);
}

export function highestGlibcVersion(versions: readonly GlibcVersion[]): GlibcVersion | undefined {
  return versions.at(-1);
}

export function glibcVersionsAbove(
  versions: readonly GlibcVersion[],
  maximum: GlibcVersion,
): GlibcVersion[] {
  return versions.filter((version) => compareGlibcVersions(version, maximum) > 0);
}

function readElfVersionInfo(file: string): string {
  const result = spawnSync("readelf", ["--version-info", file], { encoding: "utf8" });
  if (result.error !== undefined) throw new Error(`failed to run readelf: ${result.error}`);
  if (result.status !== 0) {
    throw new Error(
      `readelf --version-info failed for ${file}\n${result.stderr}${result.stdout}`.trim(),
    );
  }
  return `${result.stdout}\n${result.stderr}`;
}

function parseArguments(arguments_: readonly string[]): {
  files: string[];
  maximum: GlibcVersion;
} {
  let maximum = defaultMaximum;
  const files: string[] = [];
  for (let index = 0; index < arguments_.length; index += 1) {
    const argument = arguments_[index]!;
    if (argument === "--max") {
      maximum = arguments_[index + 1] ?? "";
      if (maximum === "") throw new Error("--max requires a value");
      index += 1;
    } else {
      files.push(argument);
    }
  }
  return { files, maximum: parseGlibcVersion(maximum) };
}

export function verifyGlibcSymbols(arguments_: readonly string[]): number {
  const { files, maximum } = parseArguments(arguments_);
  if (files.length === 0) {
    throw new Error(
      "Usage: node tools/commands/ci/github/verify-glibc-symbols.ts [--max 2.36] <file.node>...",
    );
  }
  let failed = false;
  for (const file of files) {
    if (!fs.existsSync(file)) {
      console.error(`::error::native binary does not exist: ${file}`);
      failed = true;
      continue;
    }
    const versions = parseGlibcVersions(readElfVersionInfo(file));
    const highest = highestGlibcVersion(versions);
    if (highest === undefined) {
      console.error(`::error::native binary has no GLIBC_* version records: ${file}`);
      failed = true;
    } else if (glibcVersionsAbove(versions, maximum).length > 0) {
      console.error(
        `::error::${file} requires GLIBC_${highest.text}, above the supported ceiling GLIBC_${maximum.text}`,
      );
      failed = true;
    } else {
      console.log(`${file}: GLIBC_${highest.text} <= GLIBC_${maximum.text}`);
    }
  }
  if (failed) throw new Error("glibc symbol verification failed");
  return 0;
}

if (isEntrypoint(import.meta.url)) {
  runEntrypoint(() => verifyGlibcSymbols(process.argv.slice(2)));
}
