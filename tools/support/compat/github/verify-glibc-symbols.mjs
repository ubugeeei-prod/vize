import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { pathToFileURL } from "node:url";

const DEFAULT_MAX_GLIBC = "2.36";

export function parseGlibcVersions(text) {
  const versions = new Map();
  for (const match of text.matchAll(/\bGLIBC_(\d+)\.(\d+)(?:\.(\d+))?\b/g)) {
    const patch = match[3] == null ? 0 : Number(match[3]);
    const version = {
      major: Number(match[1]),
      minor: Number(match[2]),
      patch,
      text: `${Number(match[1])}.${Number(match[2])}${patch === 0 ? "" : `.${patch}`}`,
    };
    versions.set(version.text, version);
  }
  return [...versions.values()].sort(compareGlibcVersions);
}

export function parseGlibcVersion(value) {
  const match = /^(\d+)\.(\d+)(?:\.(\d+))?$/.exec(value);
  if (!match) {
    throw new Error(`glibc version must look like MAJOR.MINOR[.PATCH], got ${value}`);
  }
  const patch = match[3] == null ? 0 : Number(match[3]);
  return {
    major: Number(match[1]),
    minor: Number(match[2]),
    patch,
    text: `${Number(match[1])}.${Number(match[2])}${patch === 0 ? "" : `.${patch}`}`,
  };
}

export function compareGlibcVersions(left, right) {
  return left.major - right.major || left.minor - right.minor || left.patch - right.patch;
}

export function highestGlibcVersion(versions) {
  return versions.at(-1) ?? null;
}

export function glibcVersionsAbove(versions, maxVersion) {
  return versions.filter((version) => compareGlibcVersions(version, maxVersion) > 0);
}

function readElfVersionInfo(filePath) {
  const result = spawnSync("readelf", ["--version-info", filePath], {
    encoding: "utf8",
  });
  if (result.error != null) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(
      `readelf --version-info failed for ${filePath}\n${result.stderr}${result.stdout}`.trim(),
    );
  }
  return `${result.stdout}\n${result.stderr}`;
}

function parseArgs(args) {
  let max = DEFAULT_MAX_GLIBC;
  const files = [];
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--max") {
      const value = args[index + 1];
      if (!value) {
        throw new Error("--max requires a value");
      }
      max = value;
      index += 1;
      continue;
    }
    files.push(arg);
  }
  return { files, max: parseGlibcVersion(max) };
}

function reportError(message) {
  console.error(`::error::${message}`);
}

function main() {
  const { files, max } = parseArgs(process.argv.slice(2));
  if (files.length === 0) {
    throw new Error(
      "Usage: rust-script tools/commands/ci/github/verify-glibc-symbols.rs [--max 2.36] <file.node>...",
    );
  }

  let failed = false;
  for (const file of files) {
    if (!fs.existsSync(file)) {
      reportError(`native binary does not exist: ${file}`);
      failed = true;
      continue;
    }

    const versions = parseGlibcVersions(readElfVersionInfo(file));
    if (versions.length === 0) {
      reportError(`native binary has no GLIBC_* version records: ${file}`);
      failed = true;
      continue;
    }

    const tooNew = glibcVersionsAbove(versions, max);
    if (tooNew.length > 0) {
      const highest = highestGlibcVersion(versions);
      reportError(
        `${file} requires GLIBC_${highest.text}, above the supported ceiling GLIBC_${max.text}`,
      );
      failed = true;
      continue;
    }

    const highest = highestGlibcVersion(versions);
    console.log(`${file}: GLIBC_${highest.text} <= GLIBC_${max.text}`);
  }

  if (failed) {
    process.exitCode = 1;
  }
}

if (process.argv[1] != null && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main();
}
