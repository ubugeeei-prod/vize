// Repository scripts are written in ush (https://github.com/ubugeeei-prod/ush). Each
// program `<dir>/<name>.ush` ships with its POSIX sh build at `<dir>/compiled/<name>.sh`,
// so CI runners and contributors run it with `sh` alone and need ush only to edit it.
// Modules under a `lib/` directory are inlined into the programs that import them and
// have no build of their own.

import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";

const compiledDirectory = "compiled";

/** The flake input pins the one ush release whose output is committed. */
const flakePin = /url = "github:ubugeeei-prod\/ush\/v(\d+\.\d+\.\d+)";/u;

export interface UshSync {
  readonly programs: readonly string[];
  readonly updated: readonly string[];
}

function isLibrary(source: string): boolean {
  return source.split("/").includes("lib");
}

/** The committed build of a program, relative to the repository root. */
export function compiledTarget(source: string): string {
  const name = path.posix.basename(source, ".ush") + ".sh";
  return path.posix.join(path.posix.dirname(source), compiledDirectory, name);
}

function trackedFiles(root: string, pattern: string): string[] {
  return execFileSync(
    "git",
    ["ls-files", "--cached", "--others", "--exclude-standard", "-z", "--", pattern],
    { cwd: root, encoding: "utf8" },
  )
    .split("\0")
    .filter((file) => file && existsSync(path.join(root, file)));
}

export function listUshPrograms(root: string): string[] {
  return trackedFiles(root, "*.ush")
    .filter((source) => !isLibrary(source))
    .sort();
}

export function pinnedUshVersion(root: string): string {
  const flake = readFileSync(path.join(root, "flake.nix"), "utf8");
  const version = flakePin.exec(flake)?.[1];
  if (!version) throw new Error("flake.nix must pin the ush input to a release tag");
  return version;
}

function installedUshVersion(): string | undefined {
  const result = spawnSync("ush", ["--version"], { encoding: "utf8" });
  if (result.error || result.status !== 0) return undefined;
  return /^ush (\S+)$/mu.exec(result.stdout)?.[1];
}

/** Builds that no longer have a program, such as those left by a rename. */
function orphanedBuilds(root: string, programs: readonly string[]): string[] {
  const targets = new Set(programs.map(compiledTarget));
  return trackedFiles(root, `*/${compiledDirectory}/*.sh`).filter((file) => !targets.has(file));
}

/**
 * Compile every program, or with `check` verify that each committed build is current.
 * Writing without ush on PATH only warns, so contributors who never touch a `.ush`
 * file can still regenerate the Pkl outputs; checking always needs it.
 */
export function syncUshPrograms(root: string, check: boolean): UshSync {
  const programs = listUshPrograms(root);
  const orphans = orphanedBuilds(root, programs);
  if (orphans.length)
    throw new Error(`Delete builds whose ush program is gone:\n${orphans.join("\n")}`);
  if (!programs.length) return { programs, updated: [] };

  const pinned = pinnedUshVersion(root);
  const installed = installedUshVersion();
  if (installed === undefined) {
    if (check) throw new Error(`Checking ush programs needs ush ${pinned} (nix develop)`);
    console.warn(`ush is not on PATH; ush programs were not compiled (nix develop provides it).`);
    return { programs, updated: [] };
  }
  if (installed !== pinned)
    throw new Error(`ush programs compile with ush ${pinned}, but ${installed} is on PATH`);

  const before = new Map(programs.map((source) => [source, readBuild(root, source)]));
  const result = spawnSync(
    "ush",
    [
      "compile",
      "--checked-in",
      ...(check ? ["--check"] : []),
      "--out-dir",
      compiledDirectory,
      ...programs,
    ],
    { cwd: root, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  if (result.error) throw result.error;
  if (result.status !== 0) {
    const fix = check ? "\nRegenerate them with `node tools/commands/generate/repository-config.ts`." : "";
    throw new Error(`${result.stderr.trim() || "ush compile failed"}${fix}`);
  }
  const updated = programs.filter((source) => readBuild(root, source) !== before.get(source));
  return { programs, updated: updated.map(compiledTarget) };
}

function readBuild(root: string, source: string): string | undefined {
  const file = path.join(root, compiledTarget(source));
  return existsSync(file) ? readFileSync(file, "utf8") : undefined;
}
