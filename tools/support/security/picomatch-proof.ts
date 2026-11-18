import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import YAML from "yaml";
import { packageDigest } from "./node-forge-proof.ts";
import { verifyPicomatchExtglob, type Picomatch } from "./picomatch-laws.ts";
import {
  assertPicomatchLock,
  picomatchPatches,
  picomatchVersions,
  record,
  type PicomatchVersion,
} from "./picomatch-lock.ts";

const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const manifest = (root: string) =>
  record(JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8")));

export function verifyPicomatchPatches(root: string): void {
  for (const version of picomatchVersions) {
    const patch = picomatchPatches[version];
    assert.equal(hash(fs.readFileSync(path.join(root, patch.path))), patch.hash);
  }
}

export function assertPicomatchPackage(root: string): PicomatchVersion {
  root = fs.realpathSync(root);
  const pkg = manifest(root),
    require = createRequire(path.join(root, "package.json"));
  assert.equal(pkg.name, "picomatch");
  const version = String(pkg.version) as PicomatchVersion;
  assert.ok(Object.hasOwn(picomatchPatches, version), "unreviewed picomatch version");
  assert.equal(pkg.main, "index.js");
  assert.equal(require.resolve(root), path.join(root, "index.js"));
  assert.equal(
    packageDigest(root),
    picomatchPatches[version].source,
    "unpatched or modified picomatch source",
  );
  return version;
}

// Every installed picomatch visibility: the root and importer node_modules,
// the virtual hoist and each store entry's sibling links (what a consumer's
// `require("picomatch")` resolves to). Aliases are refused.
export function installedPicomatchCensus(root: string): Set<string> {
  const instances = new Set<string>(),
    seen = new Set<string>();
  const inspect = (lexical: string, alias: string): void => {
    const manifestPath = path.join(lexical, "package.json");
    if (!fs.existsSync(manifestPath)) return;
    const pkg = manifest(fs.realpathSync(lexical));
    if (pkg.name === "picomatch") {
      assert.equal(alias, "picomatch", "aliased installed picomatch entry");
      instances.add(fs.realpathSync(lexical));
    } else assert.notEqual(alias, "picomatch", "foreign package installed as picomatch");
  };
  const scan = (modules: string): void => {
    if (!fs.existsSync(modules)) return;
    const canonical = fs.realpathSync(modules);
    if (seen.has(canonical)) return;
    seen.add(canonical);
    for (const name of fs.readdirSync(modules).sort()) {
      if (name === ".bin" || name === ".pnpm" || name.startsWith(".")) continue;
      const lexical = path.join(modules, name);
      if (name.startsWith("@")) {
        for (const child of fs.readdirSync(lexical).sort())
          inspect(path.join(lexical, child), name + "/" + child);
      } else inspect(lexical, name);
    }
  };
  const lock = YAML.parseAllDocuments(fs.readFileSync(path.join(root, "pnpm-lock.yaml"), "utf8"));
  for (const importer of Object.keys(record(record(lock[1].toJS()).importers)))
    scan(path.join(root, importer, "node_modules"));
  const store = path.join(root, "node_modules/.pnpm");
  scan(path.join(store, "node_modules"));
  for (const directory of fs.readdirSync(store).sort()) {
    if (directory === "node_modules" || directory.startsWith(".")) continue;
    if (!fs.lstatSync(path.join(store, directory)).isDirectory()) continue;
    scan(path.join(store, directory, "node_modules"));
  }
  assert.ok(instances.size > 0, "missing picomatch installation");
  return instances;
}

export function verifyInstalledPicomatch(root: string): {
  instances: number;
  versions: string[];
  edges: number;
  negativeChecks: number;
  positiveChecks: number;
} {
  const workspace = YAML.parseDocument(
    fs.readFileSync(path.join(root, "pnpm-workspace.yaml"), "utf8"),
  );
  const documents = YAML.parseAllDocuments(
    fs.readFileSync(path.join(root, "pnpm-lock.yaml"), "utf8"),
  );
  assert.equal(workspace.errors.length, 0);
  assert.equal(documents.length, 2);
  assert.ok(documents.every((document) => document.errors.length === 0));
  const edges = assertPicomatchLock(workspace.toJS(), documents[1].toJS());
  verifyPicomatchPatches(root);
  const instances = installedPicomatchCensus(root),
    versions = new Set<string>();
  let negativeChecks = 0,
    positiveChecks = 0;
  for (const packageRoot of instances) {
    versions.add(assertPicomatchPackage(packageRoot));
    const require = createRequire(path.join(packageRoot, "package.json"));
    const laws = verifyPicomatchExtglob(require(packageRoot) as Picomatch);
    negativeChecks += laws.negativeChecks;
    positiveChecks += laws.positiveChecks;
  }
  assert.deepEqual([...versions].sort(), picomatchVersions, "missing patched picomatch line");
  return {
    instances: instances.size,
    versions: [...versions].sort(),
    edges,
    negativeChecks,
    positiveChecks,
  };
}
