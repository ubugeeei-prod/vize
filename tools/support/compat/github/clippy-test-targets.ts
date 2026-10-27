// Crates listed in tools/config/clippy-test-targets.json stay clean under
// `cargo clippy --all-targets -- -D warnings`. PR CI lints the listed crates a
// PR touches, so the lane costs nothing for other crates; `all` lints the
// whole list locally. The list only grows.
import { execFileSync, spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

export const ratchetPath = "tools/config/clippy-test-targets.json";
const cratePattern = /^[A-Za-z0-9][A-Za-z0-9_-]*$/;

export function parseRatchet(text: string): string[] {
  const parsed: unknown = JSON.parse(text);
  const crates = (parsed as { crates?: unknown } | null)?.crates;
  if (!Array.isArray(crates) || crates.some((name) => typeof name !== "string")) {
    throw new Error(`${ratchetPath} must list crate names under "crates"`);
  }
  for (const [index, name] of crates.entries()) {
    if (!cratePattern.test(name)) throw new Error(`invalid crate name: ${name}`);
    if (index > 0 && crates[index - 1] >= name) {
      throw new Error(`${ratchetPath} must stay sorted and unique: ${name}`);
    }
  }
  return crates;
}

/** Listed crates that own a changed file. */
export function touchedRatchetCrates(
  plan: { changedPackages?: unknown },
  ratchet: readonly string[],
): string[] {
  const changed = plan.changedPackages;
  if (!Array.isArray(changed) || changed.some((name) => typeof name !== "string")) {
    throw new Error("expected the Rust plan to list its changedPackages");
  }
  return ratchet.filter((name) => changed.includes(name));
}

/** Entries a change drops; the ratchet only grows. */
export function droppedRatchetCrates(base: readonly string[], head: readonly string[]): string[] {
  return base.filter((name) => !head.includes(name));
}

export function clippyArgs(crates: readonly string[]): string[] {
  return [
    "clippy",
    ...crates.flatMap((name) => ["--package", name]),
    "--all-targets",
    "--profile",
    "ci",
    "--",
    "-D",
    "warnings",
  ];
}

function readHead(): string[] {
  return parseRatchet(readFileSync(ratchetPath, "utf8"));
}

function hasObject(spec: string): boolean {
  return spawnSync("git", ["cat-file", "-e", spec], { stdio: "ignore" }).status === 0;
}

function runClippy(crates: readonly string[]): number {
  if (!crates.length) {
    process.stdout.write("No listed crate is touched; test-target Clippy is skipped.\n");
    return 0;
  }
  process.stdout.write(`Test-target Clippy: ${crates.join(", ")}\n`);
  const result = spawnSync("cargo", clippyArgs(crates), { stdio: "inherit", shell: false });
  if (result.error) throw result.error;
  return result.status ?? 1;
}

export function main(argv = process.argv.slice(2)): number {
  const [command, argument] = argv;
  if (command === "touched" && argv.length === 2) {
    const plan: unknown = JSON.parse(readFileSync(argument, "utf8"));
    return runClippy(touchedRatchetCrates(plan as { changedPackages?: unknown }, readHead()));
  }
  if (command === "all" && argv.length === 1) return runClippy(readHead());
  if (command === "hold" && argv.length === 2 && /^[0-9a-f]{40}$/.test(argument)) {
    // An unknown predecessor has nothing to hold.
    if (/^0+$/.test(argument)) return 0;
    if (!hasObject(`${argument}^{commit}`)) throw new Error(`base ${argument} is not fetched`);
    // The change that introduces the list has nothing to hold either.
    if (!hasObject(`${argument}:${ratchetPath}`)) return 0;
    const base = execFileSync("git", ["show", `${argument}:${ratchetPath}`], { encoding: "utf8" });
    const dropped = droppedRatchetCrates(parseRatchet(base), readHead());
    if (!dropped.length) return 0;
    process.stderr.write(
      `${ratchetPath} only grows; restore ${dropped.join(", ")} and fix their test-target Clippy findings.\n`,
    );
    return 1;
  }
  throw new Error("usage: clippy-test-targets.ts touched <rust-plan.json> | all | hold <base-sha>");
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  process.exitCode = main();
}
