import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import YAML from "yaml";
import { packageDigest } from "./node-forge-proof.ts";
import { verifyBracesDepth, type Braces } from "./braces-laws.ts";
import { assertNoDirectBraces, protectedBracesModule } from "./braces-consumers.ts";
import { assertBracesLock, bracesPatchHash, bracesPatchPath, record } from "./braces-lock.ts";

const packageHash = "74958865c440ebdccbe18322a4b9a2d6468368e5a3c975a761c0752d8ff2c0cc";
const parents = {
  chokidar: {
    version: "3.6.0",
    hash: "1761c1f15d0293005713982750e15c043d27b9c04e406b53d6ef44697c5198dc",
  },
  micromatch: {
    version: "4.0.8",
    hash: "cf80b3ae96130204d9ef457a21f30c8d97035fe7de3eb78a044f0a9d543c31a9",
  },
};
const rangeChain = [
  ["fill-range", "7.1.1", "c3cb9715ced30dbe6b71e095dc2bc98222177343a294f0ca48c68dc84ad6e93c"],
  ["to-regex-range", "5.0.1", "599ca2aebf139e16e866a4240dd820abdba387b62d82504f10926f3b241762ef"],
  ["is-number", "7.0.0", "061308ec61c779f202e94f37f58f1f033ca2cd6cc8f67132c8ad668bdd6c2f2d"],
];
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const manifest = (root: string) =>
  record(JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8")));

export function verifyBracesPatch(root: string): void {
  assert.equal(hash(fs.readFileSync(path.join(root, bracesPatchPath))), bracesPatchHash);
}

export function assertBracesPackage(root: string): void {
  root = fs.realpathSync(root);
  const pkg = manifest(root),
    require = createRequire(path.join(root, "package.json"));
  assert.equal(pkg.name, "braces");
  assert.equal(pkg.version, "3.0.3");
  assert.equal(pkg.main, "index.js");
  for (const alternate of ["browser", "module", "exports"])
    assert.equal(pkg[alternate], undefined, "alternate Braces entry");
  assert.equal(require.resolve(root), path.join(root, "index.js"));
  assert.equal(packageDigest(root), packageHash, "unpatched or modified Braces source");
}

function assertRangeDependencies(bracesRoot: string): void {
  let owner = bracesRoot;
  for (const [name, version, digest] of rangeChain) {
    const require = createRequire(path.join(owner, "package.json"));
    const next = fs.realpathSync(path.dirname(require.resolve(name + "/package.json")));
    const pkg = manifest(next);
    assert.equal(pkg.name, name);
    assert.equal(pkg.version, version);
    assert.equal(pkg.main, "index.js");
    for (const alternate of ["browser", "module", "exports"])
      assert.equal(pkg[alternate], undefined);
    assert.equal(require.resolve(name), path.join(next, "index.js"));
    assert.equal(packageDigest(next), digest, "unreviewed range dependency source");
    owner = next;
  }
}

export function assertBracesConsumer(root: string): string {
  root = fs.realpathSync(root);
  const pkg = manifest(root),
    name = String(pkg.name);
  assert.ok(Object.hasOwn(parents, name), "unreviewed installed Braces consumer");
  const pin = parents[name as keyof typeof parents];
  assert.equal(pkg.version, pin.version);
  assert.equal(packageDigest(root), pin.hash, "modified Braces consumer source");
  const require = createRequire(path.join(root, "package.json"));
  const bracesRoot = fs.realpathSync(path.dirname(require.resolve("braces/package.json")));
  assertBracesPackage(bracesRoot);
  const braces = require(bracesRoot) as Braces;
  verifyBracesDepth(braces);
  if (name === "micromatch") {
    const consumer = require(root);
    assert.deepEqual(consumer.braceExpand("src/{app,lib}/{1..3}.ts"), [
      "src/app/1.ts",
      "src/app/2.ts",
      "src/app/3.ts",
      "src/lib/1.ts",
      "src/lib/2.ts",
      "src/lib/3.ts",
    ]);
    assert.throws(
      () => consumer.braces("{".repeat(101) + "a,b" + "}".repeat(101)),
      /exceeds max depth/,
    );
  } else {
    // Build its actual glob helper without starting filesystem watchers.
    const consumer = require(root),
      watcher = new consumer.FSWatcher({ persistent: false });
    const helper = watcher._getWatchHelpers("src/{app,lib}/*.ts", 0);
    assert.deepEqual(helper.dirParts, [["app"], ["lib"]]);
    assert.throws(
      () =>
        watcher._getWatchHelpers("src/" + "{".repeat(101) + "a,b" + "}".repeat(101) + "/*.ts", 0),
      /exceeds max depth/,
    );
  }
  return bracesRoot;
}

// Enumerate real installed manifests, including aliases, scoped entries and the
// virtual hoist. Every discovered Braces visibility must resolve to patched bytes.
export function installedBracesCensus(root: string): {
  instances: Set<string>;
  consumers: Set<string>;
} {
  const instances = new Set<string>(),
    consumers = new Set<string>(),
    seen = new Set<string>();
  const inspect = (lexical: string, alias: string): void => {
    const pkgRoot = fs.realpathSync(lexical);
    const manifestPath = path.join(pkgRoot, "package.json");
    if (!fs.existsSync(manifestPath)) return;
    const pkg = manifest(pkgRoot);
    if (pkg.name === "braces") {
      assert.equal(alias, "braces", "aliased installed Braces entry");
      instances.add(pkgRoot);
    }
    for (const field of ["dependencies", "optionalDependencies", "devDependencies"]) {
      for (const [name, value] of Object.entries(record(pkg[field] ?? {}))) {
        if (!protectedBracesModule(name) && !protectedBracesModule(String(value))) continue;
        assert.equal(name, "braces", "aliased installed Braces dependency");
        assert.ok(Object.hasOwn(parents, String(pkg.name)), "new installed Braces consumer");
        consumers.add(pkgRoot);
      }
    }
    scan(path.join(pkgRoot, "node_modules"));
  };
  const scan = (modules: string): void => {
    if (!fs.existsSync(modules)) return;
    const canonical = fs.realpathSync(modules);
    if (seen.has(canonical)) return;
    seen.add(canonical);
    for (const name of fs.readdirSync(modules).sort()) {
      if (name === ".bin" || name === ".pnpm") continue;
      const lexical = path.join(modules, name);
      if (name.startsWith("@")) {
        for (const child of fs.readdirSync(lexical).sort())
          inspect(path.join(lexical, child), name + "/" + child);
      } else if (!name.startsWith(".")) inspect(lexical, name);
    }
  };
  scan(path.join(root, "node_modules"));
  const store = path.join(root, "node_modules/.pnpm");
  scan(path.join(store, "node_modules"));
  for (const directory of fs.readdirSync(store).sort()) {
    if (directory === "node_modules" || directory.startsWith(".")) continue;
    if (!fs.lstatSync(path.join(store, directory)).isDirectory()) continue;
    scan(path.join(store, directory, "node_modules"));
  }
  assert.ok(instances.size > 0 && consumers.size > 0, "missing Braces installation");
  assert.deepEqual([...consumers].map((directory) => String(manifest(directory).name)).sort(), [
    "chokidar",
    "micromatch",
  ]);
  return { instances, consumers };
}

export function verifyInstalledBraces(root: string): {
  instances: number;
  patch: string;
  source: string;
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
  assertBracesLock(workspace.toJS(), documents[1].toJS());
  verifyBracesPatch(root);
  const tracked = spawnSync("git", ["ls-files", "-z"], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 8 * 1024 * 1024,
  });
  assert.equal(tracked.status, 0, tracked.stderr);
  for (const relative of tracked.stdout
    .split("\0")
    .filter((file) => /\.(?:[cm]?js|[cm]?ts|jsx|tsx|vue|html)$/.test(file)))
    assertNoDirectBraces(fs.readFileSync(path.join(root, relative), "utf8"));
  const { instances, consumers } = installedBracesCensus(root);
  let negativeChecks = 0,
    positiveChecks = 0;
  for (const pkgRoot of instances) {
    assertBracesPackage(pkgRoot);
    assertRangeDependencies(pkgRoot);
    const require = createRequire(path.join(pkgRoot, "package.json"));
    const laws = verifyBracesDepth(require(pkgRoot) as Braces);
    negativeChecks += laws.negativeChecks;
    positiveChecks += laws.positiveChecks;
  }
  for (const consumer of consumers) assert.ok(instances.has(assertBracesConsumer(consumer)));
  return {
    instances: instances.size,
    patch: bracesPatchHash,
    source: packageHash,
    negativeChecks,
    positiveChecks,
  };
}
