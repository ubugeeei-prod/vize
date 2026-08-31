#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { dirname, relative, resolve, sep } from "node:path";
import { pathToFileURL } from "node:url";
import { appendFileSync, writeFileSync } from "node:fs";

import { CRITERION_SUITES } from "./criterion-ab.mjs";

const FULL_SWEEP_PATHS = new Set([
  ".github/workflows/criterion-bench.yml",
  "Cargo.lock",
  "Cargo.toml",
  "tools/benchmarks/scripts/criterion-ab.mjs",
  "tools/benchmarks/scripts/criterion-baselines.mjs",
  "tools/benchmarks/scripts/criterion-impact.mjs",
  "tools/benchmarks/scripts/criterion-summary.mjs",
  "tools/benchmarks/scripts/dialect-guard.mjs",
  "tools/benchmarks/scripts/generate.mjs",
  "rust-toolchain.toml",
]);
const HOSTED_FALLBACK_SMOKE_SUITES = ["vize_glyph"];
const RUST_BENCHMARK_SUBJECT_PATHS = new Set(["Cargo.lock", "Cargo.toml", "rust-toolchain.toml"]);

function parseArgs(argv) {
  const args = {};
  for (let index = 0; index < argv.length; index++) {
    const arg = argv[index];
    if (!arg.startsWith("--")) continue;
    const key = arg.slice(2);
    const value = argv[index + 1];
    if (value == null || value.startsWith("--")) {
      args[key] = true;
    } else {
      args[key] = value;
      index++;
    }
  }
  return args;
}

function requireArg(args, key) {
  const value = args[key];
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`Missing required argument: --${key}`);
  }
  return value;
}

function run(command, args, cwd) {
  const result = spawnSync(command, args, {
    cwd,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(
      `${command} ${args.join(" ")} exited with ${result.status}: ${result.stderr.trim()}`,
    );
  }
  return result.stdout;
}

function normalizeRepoPath(path) {
  return path.split(sep).join("/").replace(/^\.\//, "");
}

export function parseNameStatusZ(output) {
  const fields = output.split("\0");
  if (fields.at(-1) === "") fields.pop();
  const paths = [];
  for (let index = 0; index < fields.length;) {
    const status = fields[index++];
    if (!/^(?:[ACDMRTUXB]|R\d+|C\d+)$/.test(status ?? "")) {
      throw new Error(`Malformed git diff status: ${status ?? "<missing>"}`);
    }
    const path = fields[index++];
    if (!path) throw new Error(`Missing path for git diff status ${status}`);
    paths.push(normalizeRepoPath(path));
    if (status.startsWith("R") || status.startsWith("C")) {
      const destination = fields[index++];
      if (!destination) throw new Error(`Missing destination for git diff status ${status}`);
      paths.push(normalizeRepoPath(destination));
    }
  }
  return [...new Set(paths)];
}

function workspacePackages(metadata, repoDir) {
  if (!Array.isArray(metadata.workspace_members) || !Array.isArray(metadata.packages)) {
    throw new Error("cargo metadata is missing workspace package data");
  }
  const members = new Set(metadata.workspace_members);
  return metadata.packages
    .filter((pkg) => members.has(pkg.id))
    .map((pkg) => ({
      id: pkg.id,
      name: pkg.name,
      root: normalizeRepoPath(relative(repoDir, dirname(pkg.manifest_path))),
    }))
    .sort((left, right) => right.root.length - left.root.length);
}

function dependencyGraph(metadata) {
  if (!Array.isArray(metadata.resolve?.nodes)) {
    throw new Error("cargo metadata is missing the resolved dependency graph");
  }
  return new Map(metadata.resolve.nodes.map((node) => [node.id, new Set(node.dependencies ?? [])]));
}

function ownerForPath(path, packages) {
  return packages.find((pkg) => path === pkg.root || path.startsWith(`${pkg.root}/`));
}

function dependsOnAny(packageId, changedIds, graph) {
  const pending = [packageId];
  const visited = new Set();
  while (pending.length > 0) {
    const current = pending.pop();
    if (changedIds.has(current)) return true;
    if (visited.has(current)) continue;
    visited.add(current);
    for (const dependency of graph.get(current) ?? []) pending.push(dependency);
  }
  return false;
}

function isCrateBenchmarkSubjectPath(path) {
  const parts = path.split("/");
  if (parts[0] !== "crates" || parts.length < 3) return false;
  const cratePath = parts.slice(2).join("/");
  return (
    cratePath === "Cargo.toml" ||
    cratePath === "build.rs" ||
    cratePath.startsWith("src/") ||
    cratePath.startsWith("benches/")
  );
}

function isRustBenchmarkSubjectPath(path) {
  return (
    RUST_BENCHMARK_SUBJECT_PATHS.has(path) ||
    path.startsWith(".cargo/") ||
    path.startsWith("tools/benchmarks/crates/") ||
    isCrateBenchmarkSubjectPath(path)
  );
}

export function selectCriterionSuites({ changedPaths, metadata, repoDir, hostedFallback = false }) {
  const inventory = CRITERION_SUITES.map((suite) => suite.package);
  const normalizedPaths = [...new Set(changedPaths.map(normalizeRepoPath))].sort((left, right) =>
    left.localeCompare(right),
  );
  const infrastructure = normalizedPaths.filter(
    (path) => FULL_SWEEP_PATHS.has(path) || path.startsWith(".cargo/"),
  );
  if (infrastructure.length > 0) {
    const subjectPaths = normalizedPaths.filter(isRustBenchmarkSubjectPath);
    if (hostedFallback && subjectPaths.length === 0) {
      const selected = inventory.filter((suite) => HOSTED_FALLBACK_SMOKE_SUITES.includes(suite));
      return {
        mode: "hosted-smoke",
        selected,
        skipped: inventory.filter((suite) => !selected.includes(suite)),
        reason: `Hosted fallback smoke: Criterion infrastructure changed without Rust benchmark subjects (${infrastructure.join(", ")}).`,
      };
    }
    return {
      mode: "full",
      selected: inventory,
      skipped: [],
      reason: `Full sweep: shared benchmark or workspace infrastructure changed (${infrastructure.join(", ")}).`,
    };
  }

  const packages = workspacePackages(metadata, repoDir);
  const packageByName = new Map(packages.map((pkg) => [pkg.name, pkg]));
  const missingSuites = inventory.filter((name) => !packageByName.has(name));
  if (missingSuites.length > 0) {
    throw new Error(
      `Criterion package(s) missing from cargo metadata: ${missingSuites.join(", ")}`,
    );
  }

  const rustPaths = normalizedPaths.filter(
    (path) => path.startsWith("crates/") || path.startsWith("tools/benchmarks/crates/"),
  );
  const unknownPaths = rustPaths.filter((path) => ownerForPath(path, packages) == null);
  if (unknownPaths.length > 0) {
    return {
      mode: "full",
      selected: inventory,
      skipped: [],
      reason: `Full sweep: changed Rust path is not owned by a workspace package (${unknownPaths.join(", ")}).`,
    };
  }

  const changedPackages = [...new Set(rustPaths.map((path) => ownerForPath(path, packages)?.id))]
    .filter(Boolean)
    .sort((left, right) => left.localeCompare(right));
  const changedIds = new Set(changedPackages);
  const graph = dependencyGraph(metadata);
  const selected = inventory.filter((name) =>
    dependsOnAny(packageByName.get(name).id, changedIds, graph),
  );
  const skipped = inventory.filter((name) => !selected.includes(name));
  const changedNames = packages
    .filter((pkg) => changedIds.has(pkg.id))
    .map((pkg) => pkg.name)
    .sort();
  return {
    mode: "scoped",
    selected,
    skipped,
    reason:
      selected.length === 0
        ? `No configured Criterion suite depends on changed package(s): ${changedNames.join(", ") || "none"}.`
        : `Selected from reverse dependency impact of: ${changedNames.join(", ")}.`,
  };
}

export function changedPathsBetween(repoDir, baseSha, headSha) {
  const mergeBase = run("git", ["merge-base", baseSha, headSha], repoDir).trim();
  if (!/^[0-9a-f]{40}$/.test(mergeBase)) {
    throw new Error("git merge-base did not return a full lowercase commit SHA");
  }
  const diff = run(
    "git",
    ["diff", "--name-status", "-z", "--find-renames", mergeBase, headSha, "--"],
    repoDir,
  );
  return parseNameStatusZ(diff);
}

export function main(argv = process.argv.slice(2)) {
  const args = parseArgs(argv);
  const repoDir = resolve(requireArg(args, "repo-dir"));
  const baseSha = requireArg(args, "base-sha");
  const headSha = requireArg(args, "head-sha");
  const out = resolve(requireArg(args, "out"));
  for (const [label, sha] of [
    ["base", baseSha],
    ["head", headSha],
  ]) {
    if (!/^[0-9a-f]{40}$/.test(sha))
      throw new Error(`${label} SHA must be a full lowercase commit SHA`);
  }

  const metadata = JSON.parse(
    run("cargo", ["metadata", "--format-version", "1", "--locked"], repoDir),
  );
  const selection = selectCriterionSuites({
    changedPaths: changedPathsBetween(repoDir, baseSha, headSha),
    metadata,
    repoDir,
    hostedFallback: process.env.VIZE_CRITERION_HOSTED_FALLBACK === "1",
  });
  writeFileSync(out, `${JSON.stringify(selection, null, 2)}\n`);
  if (process.env.GITHUB_OUTPUT) {
    appendFileSync(
      process.env.GITHUB_OUTPUT,
      `has_suites=${selection.selected.length > 0 ? "true" : "false"}\n`,
    );
  }
  process.stdout.write(
    `${selection.reason}\nSelected: ${selection.selected.join(", ") || "none"}\n`,
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    main();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
