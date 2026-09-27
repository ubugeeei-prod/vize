import { execFileSync } from "node:child_process";
import { appendFileSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, relative, resolve, sep } from "node:path";
import { pathToFileURL } from "node:url";

import { changedPaths, isSharedRustInput } from "./plan-source-checks.mjs";

const packageName = /^[A-Za-z0-9][A-Za-z0-9_-]*$/;
const contexts = new Set(["pull_request", "merge_group"]);

function compareStrings(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function repositoryPath(root, absolutePath) {
  if (!isAbsolute(absolutePath)) throw new Error("Cargo metadata paths must be absolute");
  const path = relative(root, absolutePath).split(sep).join("/");
  if (path === ".." || path.startsWith("../") || isAbsolute(path)) {
    throw new Error("workspace package is outside the repository");
  }
  return path;
}

export function workspaceGraph(metadata) {
  if (!Array.isArray(metadata?.packages) || !Array.isArray(metadata.workspace_members)) {
    throw new Error("invalid Cargo workspace metadata");
  }
  const members = new Set(metadata.workspace_members);
  const packages = metadata.packages.filter((pkg) => members.has(pkg.id));
  if (!members.size || packages.length !== members.size) {
    throw new Error("Cargo metadata does not contain every workspace member");
  }
  const names = new Set();
  const roots = new Map();
  const reverse = new Map();
  for (const pkg of packages) {
    if (!packageName.test(pkg.name) || names.has(pkg.name) || !Array.isArray(pkg.dependencies)) {
      throw new Error("invalid or ambiguous workspace package");
    }
    names.add(pkg.name);
    const directory = dirname(pkg.manifest_path);
    const path = repositoryPath(metadata.workspace_root, directory);
    if (!path || roots.has(directory)) throw new Error("ambiguous workspace package directory");
    roots.set(directory, { name: pkg.name, path });
    reverse.set(pkg.name, new Set());
  }
  for (const pkg of packages) {
    for (const dependency of pkg.dependencies) {
      // --no-deps retains optional, renamed, target-specific, dev and build
      // path dependencies, even when they are inactive on this runner.
      if (!dependency.path) continue;
      if (!isAbsolute(dependency.path)) throw new Error("invalid Cargo dependency path");
      const target = roots.get(resolve(dependency.path));
      if (target) reverse.get(target.name).add(pkg.name);
    }
  }
  return { names: [...names].sort(compareStrings), roots: [...roots.values()], reverse };
}

function validChangedPath(path) {
  return (
    typeof path === "string" &&
    path.length > 0 &&
    !path.includes("\0") &&
    !path.includes("\\") &&
    !isAbsolute(path) &&
    !path.split("/").some((part) => part === "." || part === ".." || !part)
  );
}

export function planAffectedRust(metadata, paths, eventName = "pull_request") {
  if (!contexts.has(eventName)) throw new Error("invalid affected Rust planning context");
  if (!Array.isArray(paths)) throw new Error("changed paths must be an array");
  const graph = workspaceGraph(metadata);
  const changed = new Set();
  const reasons = new Set();
  const excludedPaths = [];
  let full = eventName === "merge_group" || paths.length === 0;
  if (full)
    reasons.add(eventName === "merge_group" ? "merge queue validates every crate" : "empty diff");
  for (const path of paths) {
    if (!validChangedPath(path)) {
      full = true;
      reasons.add("invalid changed path");
      continue;
    }
    if (isSharedRustInput(path)) {
      full = true;
      reasons.add(`shared or unknown input: ${path}`);
      continue;
    }
    const owner = graph.roots
      .filter((pkg) => path.startsWith(`${pkg.path}/`))
      .sort((a, b) => b.path.length - a.path.length)[0];
    if (owner) {
      changed.add(owner.name);
      continue;
    }
    // Zed has its own [workspace] and is checked by the editor lane.
    if (path.startsWith("editors/zed/")) {
      excludedPaths.push(path);
      continue;
    }
    // Only documentation file extensions are exempt: a script or fixture
    // moved under docs must still fail closed.
    if (/^(docs\/|\.changeset\/)/.test(path) && /\.(md|mdx)$/.test(path)) continue;
    if (path === "README.md" || path === "AGENTS.md") continue;
    full = true;
    reasons.add(`shared or unknown input: ${path}`);
  }
  const affected = new Set(changed);
  const pending = [...changed];
  for (let index = 0; index < pending.length; index++) {
    for (const consumer of graph.reverse.get(pending[index])) {
      if (!affected.has(consumer)) {
        affected.add(consumer);
        pending.push(consumer);
      }
    }
  }
  const packages = full ? graph.names : [...affected].sort(compareStrings);
  return {
    schemaVersion: 1,
    scope: full ? "workspace" : packages.length ? "affected" : "none",
    packages,
    cargoArgs: packages.flatMap((name) => ["--package", name]),
    changedPackages: [...changed].sort(compareStrings),
    reasons: [...reasons].sort(compareStrings),
    excludedPaths: excludedPaths.sort(compareStrings),
  };
}

export function readCargoMetadata(cwd = process.cwd()) {
  return JSON.parse(
    execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1", "--locked"], {
      cwd,
      encoding: "utf8",
      maxBuffer: 32 * 1024 * 1024,
    }),
  );
}

export function main(argv = process.argv.slice(2)) {
  const [base, head, eventName, outputPath] = argv;
  if (
    argv.length !== 4 ||
    !/^[0-9a-f]{40}$/.test(base ?? "") ||
    !/^[0-9a-f]{40}$/.test(head ?? "")
  ) {
    throw new Error("expected base SHA, head SHA, event context and JSON output path");
  }
  const paths = /^0+$/.test(base) ? ["Cargo.toml"] : changedPaths(base, head);
  const plan = planAffectedRust(readCargoMetadata(), paths, eventName);
  const json = JSON.stringify(plan);
  writeFileSync(outputPath, `${json}\n`);
  if (process.env.GITHUB_OUTPUT) {
    appendFileSync(
      process.env.GITHUB_OUTPUT,
      `rust-plan=${json}\nrust-packages=${JSON.stringify(plan.packages)}\nrust-scope=${plan.scope}\n`,
    );
  }
  if (process.env.GITHUB_STEP_SUMMARY) {
    appendFileSync(
      process.env.GITHUB_STEP_SUMMARY,
      `### Affected Rust crates\n\nScope: ${plan.scope}; ${plan.packages.length} packages.\n\n\`\`\`json\n${json}\n\`\`\`\n`,
    );
  }
  process.stdout.write(`${json}\n`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) main();
