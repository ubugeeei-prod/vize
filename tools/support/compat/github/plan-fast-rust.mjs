import { execFileSync } from "node:child_process";
import { appendFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

import { readCargoMetadata, workspaceGraph } from "./plan-affected-rust.mjs";

function changedEntries(base, head, cwd = process.cwd()) {
  const fields = execFileSync(
    "git",
    ["diff", "--no-renames", "--name-status", "--diff-filter=ACDMRT", "-z", base, head],
    { cwd, encoding: "utf8" },
  ).split("\0");
  fields.pop();
  if (fields.length % 2 !== 0) throw new Error("invalid changed Rust path inventory");
  const entries = [];
  for (let index = 0; index < fields.length; index += 2) {
    entries.push({ status: fields[index], path: fields[index + 1] });
  }
  return entries;
}

export function planFastRust(metadata, entries, eventName = "pull_request") {
  if (!["pull_request", "merge_group"].includes(eventName) || !Array.isArray(entries)) {
    throw new Error("invalid fast Rust planning input");
  }
  const graph = workspaceGraph(metadata);
  const packages = new Set();
  const reasons = [];
  if (eventName === "merge_group") reasons.push("merge queue requires the full Rust tier");
  if (entries.length === 0) reasons.push("empty comparison");
  for (const entry of entries) {
    const path = entry?.path;
    if (
      typeof path !== "string" ||
      !path ||
      path.includes("\\") ||
      path.includes("\0") ||
      path.startsWith("/") ||
      path.split("/").some((part) => !part || part === "." || part === "..") ||
      !["A", "M"].includes(entry.status)
    ) {
      reasons.push(`unreviewed or deleted input: ${String(path)}`);
      continue;
    }
    const owner = graph.roots
      .filter((pkg) => path.startsWith(`${pkg.path}/src/`))
      .sort((left, right) => right.path.length - left.path.length)[0];
    const metadataPackage = metadata.packages.find((pkg) => pkg.name === owner?.name);
    if (
      !owner ||
      !path.endsWith(".rs") ||
      !metadataPackage?.targets?.some((target) => target.kind?.includes("lib"))
    ) {
      reasons.push(`non-library Rust source or shared input: ${path}`);
      continue;
    }
    packages.add(owner.name);
  }
  const selected = [...packages].sort();
  const mode = reasons.length === 0 && selected.length > 0 ? "fast" : "broad";
  return {
    schemaVersion: 1,
    mode,
    scope: "affected",
    packages: mode === "fast" ? selected : [],
    cargoArgs: mode === "fast" ? selected.flatMap((name) => ["--package", name]) : [],
    reasons,
  };
}

export function main(argv = process.argv.slice(2)) {
  const [base, head, eventName, outputPath] = argv;
  if (
    argv.length !== 4 ||
    !/^[0-9a-f]{40}$/.test(base ?? "") ||
    !/^[0-9a-f]{40}$/.test(head ?? "") ||
    eventName !== "pull_request"
  ) {
    throw new Error("expected pull request base and head SHAs and JSON output path");
  }
  const entries = /^0+$/.test(base)
    ? [{ status: "M", path: "Cargo.toml" }]
    : changedEntries(base, head);
  const plan = planFastRust(readCargoMetadata(), entries, eventName);
  const json = JSON.stringify(plan);
  writeFileSync(outputPath, `${json}\n`);
  if (process.env.GITHUB_OUTPUT) {
    appendFileSync(process.env.GITHUB_OUTPUT, `rust-fast=${plan.mode === "fast"}\nrust-fast-plan=${json}\n`);
  }
  if (process.env.GITHUB_STEP_SUMMARY) {
    appendFileSync(
      process.env.GITHUB_STEP_SUMMARY,
      `### PR Rust source tier\n\n${plan.mode}: ${plan.packages.join(", ") || plan.reasons.join("; ")}\n`,
    );
  }
  process.stdout.write(`${json}\n`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) main();
