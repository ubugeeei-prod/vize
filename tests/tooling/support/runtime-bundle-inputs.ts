import { createRequire } from "node:module";
import { existsSync, readFileSync, readdirSync, realpathSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { sha256 } from "./runtime-bundle-cache.ts";

type Manifest = {
  name: string;
  version: string;
  dependencies?: Record<string, string>;
  optionalDependencies?: Record<string, string>;
  peerDependencies?: Record<string, string>;
};

function packageManifest(entry: string): string {
  let directory = dirname(realpathSync(entry));
  for (;;) {
    const path = join(directory, "package.json");
    if (existsSync(path) && JSON.parse(readFileSync(path, "utf8")).name) return path;
    const parent = dirname(directory);
    if (parent === directory) throw new Error(`No package manifest for ${entry}`);
    directory = parent;
  }
}

/** Includes installed package bytes, optional/peer presence and transitive resolution. */
export function packageInputFiles(entries: string[]) {
  const files = new Set<string>();
  const packages = new Map<string, Manifest>();
  const missing: string[] = [];
  const visited = new Set<string>();
  function walk(directory: string) {
    const actual = realpathSync(directory);
    if (visited.has(actual)) return;
    visited.add(actual);
    for (const item of readdirSync(actual, { withFileTypes: true })) {
      if (item.name === "node_modules") continue;
      const path = join(actual, item.name);
      if (item.isDirectory()) walk(path);
      else if (item.isFile()) files.add(path);
      else if (item.isSymbolicLink()) {
        const target = realpathSync(path);
        if (statSync(target).isDirectory()) walk(target);
        else files.add(target);
      } else throw new Error(`Unsupported package input: ${path}`);
    }
  }
  function visit(path: string) {
    path = realpathSync(path);
    if (packages.has(path)) return;
    const manifest = JSON.parse(readFileSync(path, "utf8")) as Manifest;
    if (!manifest.name || !manifest.version)
      throw new Error(`Incomplete package manifest: ${path}`);
    packages.set(path, manifest);
    walk(dirname(path));
    const fromPackage = createRequire(path);
    const optional = { ...manifest.optionalDependencies, ...manifest.peerDependencies };
    const dependencies = { ...manifest.dependencies, ...optional };
    for (const name of Object.keys(dependencies).sort()) {
      let entry: string;
      try {
        try {
          entry = fromPackage.resolve(`${name}/package.json`);
        } catch {
          // Type-only packages may hide both their manifest and default entry.
          // Node's ordered package search paths still identify their installed bytes.
          const manifestPath = fromPackage.resolve
            .paths(name)
            ?.map((directory) => join(directory, name, "package.json"))
            .find((candidate) => existsSync(candidate));
          entry = manifestPath ?? packageManifest(fromPackage.resolve(name));
        }
      } catch (error) {
        if (!Object.hasOwn(optional, name)) throw error;
        missing.push(`${path}:${name}`);
        continue;
      }
      visit(entry);
    }
  }
  for (const entry of entries) visit(packageManifest(resolve(entry)));
  return {
    files: [...files].sort(),
    missing: missing.sort(),
    packages: [...packages]
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([path, manifest]) => ({
        path,
        name: manifest.name,
        version: manifest.version,
      })),
  };
}

export function fingerprintFiles(files: string[]) {
  return [...new Set(files.map((file) => resolve(file)))].sort().map((path) => ({
    path,
    sha256: existsSync(path) ? sha256(readFileSync(path)) : null,
  }));
}
