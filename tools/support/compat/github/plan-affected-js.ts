import { execFileSync } from "node:child_process";
import { appendFileSync, readFileSync } from "node:fs";
import { dirname, matchesGlob, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { testedPackageGroups } from "../../../config/vite-plus/tested-package-groups.ts";
import {
  changedPaths,
  isCensusToolingInput,
  isDocsQualityToolingInput,
  isSharedRustInput,
  isToolingOnlyInput,
} from "./plan-source-checks.ts";

export type JsPackage = {
  name: string;
  path: string;
  dependencies: string[];
};
export type RustSelection = { packages: readonly string[]; scope: string };

// Read only tracked workspace manifests. Examples and fixture copies outside
// pnpm's declared workspace must not introduce duplicate package identities.
export function readJsWorkspace(cwd = process.cwd()): JsPackage[] {
  const source = readFileSync(resolve(cwd, "pnpm-workspace.yaml"), "utf8");
  const section = /^packages:\s*\n((?:[ \t]+[^\n]*\n|\s*\n)*)/m.exec(source)?.[1];
  if (!section) throw new Error("missing pnpm workspace package paths");
  const patterns = section
    .split("\n")
    .filter((line) => line.trim() && !line.trim().startsWith("#"))
    .map((line) => {
      const pattern = /^\s+-\s+"([^"\\]+)"\s*(?:#.*)?$/.exec(line)?.[1];
      if (!pattern) throw new Error("unsupported pnpm package pattern; update the planner");
      return pattern;
    });
  const selected = (path: string) =>
    patterns.some((pattern) => !pattern.startsWith("!") && matchesGlob(path, pattern)) &&
    !patterns.some((pattern) => pattern.startsWith("!") && matchesGlob(path, pattern.slice(1)));
  const files = execFileSync("git", ["ls-files", "-z", "**/package.json"], {
    cwd,
    encoding: "utf8",
  })
    .split("\0")
    .filter(Boolean);
  return files
    .filter((file) => selected(dirname(file)))
    .map((file) => {
      const manifest = JSON.parse(readFileSync(resolve(cwd, file), "utf8"));
      if (typeof manifest.name !== "string") throw new Error(`missing package name: ${file}`);
      return {
        name: manifest.name,
        path: dirname(file),
        dependencies: Object.keys({
          ...manifest.dependencies,
          ...manifest.devDependencies,
          ...manifest.optionalDependencies,
          ...manifest.peerDependencies,
        }),
      };
    });
}

export function planAffectedJs(
  workspace: readonly JsPackage[],
  paths: readonly string[],
  rust?: RustSelection,
) {
  const byName = new Map(workspace.map((pkg) => [pkg.name, pkg]));
  if (byName.size !== workspace.length) throw new Error("duplicate workspace package name");
  const changed = new Set<string>();
  const reasons = new Set<string>();
  let full = paths.length === 0;
  const nativeBindings = new Map([
    ["vize_vitrine", "@vizejs/native"],
    ["vize_fresco", "@vizejs/fresco-native"],
  ]);
  for (const [crate, pkg] of nativeBindings) {
    if (rust?.packages.includes(crate)) changed.add(pkg);
  }
  for (const path of paths) {
    if (isCensusToolingInput(path) || isDocsQualityToolingInput(path)) continue;
    if (/^(?:docs\/|\.changeset\/).*\.(?:md|mdx)$/.test(path) && !isSharedRustInput(path)) continue;
    if (/(?:^|\/)(?:README|AGENTS)\.md$/.test(path)) continue;
    // Fixture ownership is not a package dependency: Rust include_str! and
    // package tests read these shared files directly from the repository.
    if (/^tests\/(?:_fixtures|fixtures|expected|_helpers)\//.test(path)) {
      full = true;
      reasons.add(`shared product fixture: ${path}`);
      continue;
    }
    const owner = workspace
      .filter((pkg) => path.startsWith(`${pkg.path}/`))
      .sort((a, b) => b.path.length - a.path.length)[0];
    if (owner) {
      // Docs have their own scheduled build and do not consume product exports
      // in a way that can affect another workspace package.
      if (owner.path !== "docs") changed.add(owner.name);
      continue;
    }
    if (/^(?:crates\/|davinci\/|vendor\/)/.test(path) && rust) continue;
    if (isToolingOnlyInput(path)) continue;
    if (path.startsWith("editors/")) continue;
    full = true;
    reasons.add(`shared or unknown input: ${path}`);
  }
  const affected = new Set(changed);
  for (let previous = -1; previous !== affected.size;) {
    previous = affected.size;
    for (const pkg of workspace) {
      if (pkg.dependencies.some((name) => affected.has(name))) affected.add(pkg.name);
    }
  }
  const packages = workspace
    .filter((pkg) => full || affected.has(pkg.name))
    .map((pkg) => pkg.path)
    .sort();
  const groups = testedPackageGroups
    .filter((group) => group.packages.some((pkg) => packages.includes(pkg.slice(2))))
    .map(({ name }) => name);
  return {
    schemaVersion: 1,
    scope: full ? "workspace" : packages.length ? "affected" : "none",
    packages,
    groups,
    matrix: { group: groups.length ? groups : ["none"] },
    conformance: {
      fresco: packages.includes("npm/fresco-native"),
      ui: packages.includes("npm/ui"),
    },
    native: full || affected.has("@vizejs/native"),
    reasons: [...reasons].sort(),
  };
}

export function main(argv = process.argv.slice(2)) {
  const [base, head, rustFile] = argv;
  if (
    argv.length < 2 ||
    argv.length > 3 ||
    ![base, head].every((sha) => /^[0-9a-f]{40}$/.test(sha ?? ""))
  )
    throw new Error("expected base SHA, head SHA and optional Rust plan path");
  const paths = /^0+$/.test(base) ? [] : changedPaths(base, head);
  const rust = rustFile ? (JSON.parse(readFileSync(rustFile, "utf8")) as RustSelection) : undefined;
  const plan = planAffectedJs(readJsWorkspace(), paths, rust);
  if (process.env.GITHUB_OUTPUT)
    appendFileSync(
      process.env.GITHUB_OUTPUT,
      `js-plan=${JSON.stringify(plan)}\njs=${plan.groups.length > 0 || plan.conformance.fresco || plan.conformance.ui}\n`,
    );
  process.stdout.write(`${JSON.stringify(plan)}\n`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) main();
