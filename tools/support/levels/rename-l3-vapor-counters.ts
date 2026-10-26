#!/usr/bin/env node
/** Replay the #6832 rename of the Vapor selection counters to the `l3.vapor.` namespace. */
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = fileURLToPath(new URL("../../../", import.meta.url));
// Spelled in parts so this script never matches itself.
const oldPrefix = ["davinci", "s3_vapor", ""].join(".");
const newPrefix = "l3.vapor.";
const suffixes = new Set([".rs", ".ts", ".mjs", ".toml"]);
const scopes = ["crates", "davinci", "tools", "tests", "npm"];

/** Live sources only: docs and captured fixtures keep their historical bytes. */
export function eligible(relative: string): boolean {
  const parts = relative.split("/");
  if (!suffixes.has(path.posix.extname(relative))) return false;
  if (parts.some((part) => ["fixtures", "snapshots", "_fixtures"].includes(part))) return false;
  return scopes.includes(parts[0] ?? "");
}

export function rewrite(source: string): string {
  return source.split(oldPrefix).join(newPrefix);
}

export function run(mode: string, cwd = root): string[] {
  if (mode !== "--check" && mode !== "--write") {
    throw new Error("usage: rename-l3-vapor-counters.ts --check|--write");
  }
  const tracked = execFileSync("git", ["ls-files", "-z", "--", ...scopes], {
    cwd,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  })
    .split("\0")
    .filter((relative) => relative && eligible(relative));
  const stale = tracked.filter((relative) =>
    readFileSync(path.join(cwd, relative), "utf8").includes(oldPrefix),
  );
  if (mode === "--write")
    for (const relative of stale) {
      const file = path.join(cwd, relative);
      writeFileSync(file, rewrite(readFileSync(file, "utf8")));
    }
  return stale;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const mode = process.argv[2] ?? "";
  const stale = run(mode);
  process.stdout.write(
    `${stale.length} files ${mode === "--write" ? "rewritten" : "contain old counters"}\n`,
  );
  if (mode === "--check" && stale.length > 0) process.exitCode = 1;
}
