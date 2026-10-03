#!/usr/bin/env node
/** Replay the #6832 differential feature rename without altering published edges. */
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const suffixes = new Set([".rs", ".toml", ".ts", ".mjs", ".yml", ".yaml"]);
const extraPaths = [
  "tools/commands/fixtures/davinci-dom-corpus-workflow.rs",
  "tools/support/compat/fixtures/davinci-dom-corpus-workflow.mjs",
  "docs/davinci/plan/test-suites.md",
];
const renames = [
  ["davinci-dom-differential", "legacy-dom-differential"],
  ["davinci-differential", "legacy-differential"],
] as const;
const protectedMessages = [
  "davinci-differential (P1-",
  "davinci-differential corpus ",
  "davinci-differential totals:",
  "vize_s1 --features davinci-differential",
];
const aliasManifests = new Set([
  "crates/vize_atelier_core/Cargo.toml",
  "crates/vize_atelier_dom/Cargo.toml",
  "crates/vize_atelier_jsx/Cargo.toml",
  "crates/vize_atelier_sfc/Cargo.toml",
  "crates/vize_atelier_ssr/Cargo.toml",
  "crates/vize_atelier_vapor/Cargo.toml",
  "crates/vize_canon/Cargo.toml",
  "crates/vize_croquis/Cargo.toml",
  "davinci/vize_l1/Cargo.toml",
  "davinci/vize_l1_to_l2/Cargo.toml",
  "crates/vize_patina/Cargo.toml",
]);
const sfcManifest = "crates/vize_atelier_sfc/Cargo.toml";
const escape = (literal: string): string => literal.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

export function eligible(relative: string): boolean {
  if (extraPaths.includes(relative)) return true;
  const parts = relative.split("/");
  if (
    !suffixes.has(path.posix.extname(relative)) ||
    parts.some((p) => ["fixtures", "snapshots"].includes(p))
  )
    return false;
  if (parts[0] === "tests") return ["tooling", "davinci_test_support"].includes(parts[1]);
  return [".github", "crates"].includes(parts[0]);
}

export function rewrite(input: string, relative: string): string {
  const literals: string[] = [];
  const protect = (literal: string): string => {
    const token = `__VIZE_PUBLISHED_FEATURE_${literals.length}__`;
    if (input.includes(token)) throw new Error("reserved rewrite placeholder in input");
    literals.push(literal);
    return token;
  };
  let source = input;
  if (relative === sfcManifest) {
    const definitions = [...source.matchAll(/^davinci-production-bench\s*=/gm)];
    const blocks = [...source.matchAll(/^davinci-production-bench\s*=\s*\[\r?\n[\s\S]*?^\]\s*$/gm)];
    if (definitions.length !== 1 || blocks.length !== 1)
      throw new Error("expected exactly one published SFC benchmark block");
    const block = blocks[0][0];
    for (const edge of ["davinci-dom-differential", "vize_atelier_ssr/davinci-differential"]) {
      if ([...block.matchAll(new RegExp(`^\\s*"${escape(edge)}",?\\s*$`, "gm"))].length !== 1)
        throw new Error(`expected exactly one published benchmark edge: ${edge}`);
    }
    source = source.replace(block, protect(block));
  }
  if (aliasManifests.has(relative)) {
    for (const [old, replacement] of renames) {
      if (old === "davinci-dom-differential" && relative !== sfcManifest) continue;
      const alias = `${old} = ["${replacement}"]`;
      source = source.replace(new RegExp(`^${escape(alias)}$`, "gm"), () => protect(alias));
    }
  }
  for (const message of protectedMessages) source = source.replaceAll(message, protect(message));
  for (const [old, replacement] of renames) source = source.replaceAll(old, replacement);
  for (let index = 0; index < literals.length; index++)
    source = source.replaceAll(`__VIZE_PUBLISHED_FEATURE_${index}__`, literals[index]);
  return source;
}

export function run(mode: "--check" | "--write", checkout = root): number {
  const tracked = execFileSync(
    "git",
    ["ls-files", "-z", "--", ".github", "crates", "tests", ...extraPaths],
    { cwd: checkout, encoding: "utf8" },
  );
  // Validate every protected block before --write can mutate any tracked file.
  const changes = tracked
    .split("\0")
    .filter((file) => file && eligible(file))
    .flatMap((relative) => {
      const source = readFileSync(path.join(checkout, relative), "utf8");
      const rewritten = rewrite(source, relative);
      return rewritten === source ? [] : [{ relative, rewritten }];
    });
  for (const { relative, rewritten } of changes) {
    if (mode === "--write") writeFileSync(path.join(checkout, relative), rewritten);
    console.log(relative);
  }
  console.log(
    `${changes.length} files ${mode === "--write" ? "rewritten" : "contain old selectors"}`,
  );
  return mode === "--check" && changes.length > 0 ? 1 : 0;
}

if (process.argv[1] && pathToFileURL(path.resolve(process.argv[1])).href === import.meta.url) {
  const mode = process.argv[2];
  if (process.argv.length !== 3 || (mode !== "--check" && mode !== "--write")) {
    console.error("usage: rename-differential-features.ts --check|--write");
    process.exitCode = 2;
  } else {
    try {
      process.exitCode = run(mode);
    } catch (error) {
      console.error(error instanceof Error ? error.message : String(error));
      process.exitCode = 2;
    }
  }
}
