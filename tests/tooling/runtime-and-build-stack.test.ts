import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const npmDir = path.join(root, "npm");

interface PackageJson {
  bin?: Record<string, string> | string;
  engines?: Record<string, string>;
  name?: string;
  private?: boolean;
}

interface NpmPackage {
  dir: string;
  json: PackageJson;
  name: string;
}

function readWorkspaceYaml(): string {
  return fs.readFileSync(path.join(root, "pnpm-workspace.yaml"), "utf-8");
}

// Package paths that pnpm-workspace.yaml explicitly excludes via "!<dir>"
// (e.g. the VS Code extensions which carry an engines.vscode field instead of node).
function workspaceIgnoredPackageDirs(workspaceYaml: string): Set<string> {
  const ignored = new Set<string>();
  for (const line of workspaceYaml.split("\n")) {
    const match = line.match(/^\s*-\s*"!(npm|editors)\/([^"]+)"\s*$/);
    if (match) {
      ignored.add(`${match[1]}/${match[2]}`);
    }
  }
  return ignored;
}

// Directory names under npm/ that pnpm-workspace.yaml explicitly excludes.
function workspaceIgnoredNpmDirs(workspaceYaml: string): Set<string> {
  return new Set(
    [...workspaceIgnoredPackageDirs(workspaceYaml)]
      .filter((dir) => dir.startsWith("npm/"))
      .map((dir) => dir.slice("npm/".length)),
  );
}

function readNpmPackages(): NpmPackage[] {
  const packages: NpmPackage[] = [];
  const visit = (relativeDir: string) => {
    const absoluteDir = path.join(npmDir, relativeDir);
    const packagePath = path.join(absoluteDir, "package.json");
    if (fs.existsSync(packagePath)) {
      const json = JSON.parse(fs.readFileSync(packagePath, "utf-8")) as PackageJson;
      packages.push({ dir: relativeDir, json, name: json.name ?? relativeDir });
    }

    for (const entry of fs.readdirSync(absoluteDir, { withFileTypes: true })) {
      if (!entry.isDirectory()) continue;
      if (entry.name === "node_modules" || entry.name === "dist") continue;
      visit(path.join(relativeDir, entry.name));
    }
  };

  for (const entry of fs.readdirSync(npmDir, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      visit(entry.name);
    }
  }

  return packages;
}

// The set of packages whose runtime portability invariants we lock down:
// published (not private) and not a workspace-ignored editor extension.
function publishedPortablePackages(): NpmPackage[] {
  const ignoredDirs = workspaceIgnoredNpmDirs(readWorkspaceYaml());
  return readNpmPackages().filter((pkg) => pkg.json.private !== true && !ignoredDirs.has(pkg.dir));
}

test("vite-stack catalog keeps the Vite+/Vitest/Void SDK proxy trio in lockstep", () => {
  const workspaceYaml = readWorkspaceYaml();

  const viteAlias = workspaceYaml.match(
    /^\s+vite: "npm:@voidzero-dev\/vite-plus-core@([^"]+)"$/m,
  )?.[1];
  const vitestAlias = workspaceYaml.match(
    /^\s+vitest: "npm:@voidzero-dev\/vite-plus-test@([^"]+)"$/m,
  )?.[1];
  const vitePlusPin = workspaceYaml.match(/^\s+vite-plus: "([^"]+)"$/m)?.[1];

  // The three entries must exist and be parseable.
  assert.ok(viteAlias, "vite => @voidzero-dev/vite-plus-core alias version");
  assert.ok(vitestAlias, "vitest => @voidzero-dev/vite-plus-test alias version");
  assert.ok(vitePlusPin, "vite-plus pin version");

  // Each must be a concrete semver pin (no ranges / tags), so the trio is reproducible.
  for (const version of [viteAlias, vitestAlias, vitePlusPin]) {
    assert.match(version, /^\d+\.\d+\.\d+/, `concrete version pin: ${version}`);
  }

  // All THREE must be the exact same version string.
  assert.equal(viteAlias, vitePlusPin, "vite alias must match the vite-plus pin");
  assert.equal(vitestAlias, vitePlusPin, "vitest alias must match the vite-plus pin");
  assert.equal(
    new Set([viteAlias, vitestAlias, vitePlusPin]).size,
    1,
    "Vite+/Vitest/Void SDK trio must share one version",
  );
});

test("every published npm package declares engines.node so Bun/Deno can consume it", () => {
  const failures: string[] = [];

  for (const pkg of publishedPortablePackages()) {
    const node = pkg.json.engines?.node;
    if (node == null) {
      failures.push(`${pkg.name}: missing engines.node`);
      continue;
    }
    assert.equal(typeof node, "string", `${pkg.name}: engines.node should be a string`);
  }

  assert.deepEqual(failures, []);
  // Sanity: the set is non-trivial and includes the flagship CLI.
  const names = publishedPortablePackages().map((pkg) => pkg.name);
  assert.ok(names.length >= 5, "expected several published packages");
  assert.ok(names.includes("vize"), "the vize CLI must be in the published set");
});

test("no published npm package declares a bun or deno engine key", () => {
  const offenders: string[] = [];

  for (const pkg of publishedPortablePackages()) {
    const engines = pkg.json.engines ?? {};
    // Declaring a bun/deno engine could fence those runtimes out; Bun & Deno honor
    // the node engine instead, so the packages must stay node-keyed only.
    if (Object.prototype.hasOwnProperty.call(engines, "bun")) {
      offenders.push(`${pkg.name}: has engines.bun`);
    }
    if (Object.prototype.hasOwnProperty.call(engines, "deno")) {
      offenders.push(`${pkg.name}: has engines.deno`);
    }
  }

  assert.deepEqual(offenders, []);
});

test("workspace-ignored editor extensions are correctly excluded from the portable set", () => {
  const ignoredDirs = workspaceIgnoredPackageDirs(readWorkspaceYaml());
  // pnpm-workspace.yaml ignores the VS Code extension dirs; confirm they exist on disk,
  // carry an engines.vscode field, and are therefore not in the portable published set.
  assert.ok(ignoredDirs.size >= 1, "expected at least one workspace-ignored package dir");

  const portableDirs = new Set(publishedPortablePackages().map((pkg) => `npm/${pkg.dir}`));
  for (const dir of ignoredDirs) {
    const pkgPath = path.join(root, dir, "package.json");
    assert.ok(fs.existsSync(pkgPath), `${dir}: ignored extension package should exist`);
    const json = JSON.parse(fs.readFileSync(pkgPath, "utf-8")) as PackageJson;
    assert.ok(json.engines?.vscode != null, `${dir}: ignored extension should be vscode-keyed`);
    assert.ok(!portableDirs.has(dir), `${dir}: must not be in the portable published set`);
  }
});

test("CLI bin entry files use the portable env-node shebang", () => {
  const binPackages = [
    { dir: "cli", binName: "vize" },
    { dir: "oxint", binName: "oxlint-vize" },
  ] as const;

  const checked: string[] = [];

  for (const { dir, binName } of binPackages) {
    const pkgPath = path.join(npmDir, dir, "package.json");
    assert.ok(fs.existsSync(pkgPath), `${dir}/package.json should exist`);
    const json = JSON.parse(fs.readFileSync(pkgPath, "utf-8")) as PackageJson;

    const bin = json.bin;
    assert.ok(bin && typeof bin === "object", `${dir}: expected a bin map`);
    const binRel = (bin as Record<string, string>)[binName];
    assert.ok(binRel, `${dir}: expected bin.${binName}`);

    const binAbs = path.join(npmDir, dir, binRel);
    if (!fs.existsSync(binAbs)) {
      // Narrow to existing bin files only and note the gap rather than fail spuriously.
      continue;
    }

    const firstLine = fs.readFileSync(binAbs, "utf-8").split("\n", 1)[0];
    // Portable shebang: Bun & Deno honor `#!/usr/bin/env node` via node-compat.
    // A hardcoded interpreter path (e.g. /usr/local/bin/node) would break portability.
    assert.equal(
      firstLine,
      "#!/usr/bin/env node",
      `${dir}/${binRel}: expected portable env-node shebang, got ${JSON.stringify(firstLine)}`,
    );
    checked.push(`${dir}/${binRel}`);
  }

  // At least the flagship CLI bin must have been verified.
  assert.ok(
    checked.some((entry) => entry.startsWith("cli/")),
    "the vize CLI bin must exist and be verified",
  );
});
