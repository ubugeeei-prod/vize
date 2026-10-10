import assert from "node:assert/strict";
import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { runtimeGraph } from "../../../../performance/support/warm-type-backed-runtime.ts";

export const originalHead = "c2bc3a55a2b70dc4dd6c4e43ee0737c347476398";
export const originalFiles = Object.freeze({
  "pnpm-lock.yaml": "6c311bea2e8c89e938f1fe4f4c3c87e08d5776022a316070538178e61ef2ea86",
  "playground/package.json": "67a5b2a66ab71004c6440696ae5c1a3824705c6a629433afd5ad462d44abda6e",
  "crates/vize/tests/support/lsp_vue_project.rs":
    "352aa0542cbfa01407eded744e207957484fd5ab0c0a4a9736f71c8dd869f168",
});
type LockedPackage = {
  name: string;
  version: string;
  key: string;
  integrity: string;
  dependencies: Record<string, string>;
};
export interface VueFixtureAuthority {
  schema: "vize-stock-alias-vue-install-v1";
  source: { head: string; files: Record<string, string> };
  installRoot: string;
  packageManifestPath: string;
  packageManifestSha256: string;
  packageLockPath: string;
  packageLockSha256: string;
  vueManifestPath: string;
  node: { path: string; version: string; sha256: string };
  npm: { path: string; version: string; sha256: string; arguments: string[]; exitCode: 0 };
  lockedPackages: LockedPackage[];
  registry: Array<{ name: string; version: string; resolved: string; integrity: string }>;
  runtimeGraph: ReturnType<typeof runtimeGraph>;
  runtimeGraphSha256: string;
}
const digest = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
const owned = (file: string, root: string) => {
  assert.ok(path.isAbsolute(file));
  const physical = fs.realpathSync(file);
  const relative = path.relative(root, physical);
  assert.ok(relative !== "" && !relative.startsWith("..") && !path.isAbsolute(relative));
  return physical;
};
const exact = (file: string, sha: string, root?: string) => {
  assert.match(sha, /^[a-f0-9]{64}$/u);
  if (root) owned(file, root);
  assert.ok(fs.statSync(file).isFile());
  assert.equal(digest(fs.readFileSync(file)), sha, `fixture custody changed: ${file}`);
};

// Closed parsing of the authenticated stock lock's package/snapshot entries;
// this is not a general YAML loader and fails on every unknown graph shape.
function blocks(section: string): Map<string, string[]> {
  const result = new Map<string, string[]>();
  let current: string[] | undefined;
  for (const line of section.split("\n")) {
    if (/^  \S/u.test(line)) {
      const match = /^  (?:'([^']+)'|(.+)):(?: \{\})?\s*$/u.exec(line);
      assert.ok(match, `unrecognized stock lock entry: ${line}`);
      const key = match[1] ?? match[2];
      assert.ok(!result.has(key));
      current = [];
      result.set(key, current);
    } else if (current) current.push(line);
  }
  return result;
}
export function stockGraph(lock: string): LockedPackage[] {
  const document = lock.split("\n---\n").at(-1)!;
  const packages = blocks(document.split("\npackages:\n")[1].split("\nsnapshots:\n")[0]);
  const snapshots = blocks(document.split("\nsnapshots:\n")[1]);
  const pending = ["vue@3.6.0-rc.6(typescript@6.0.3)"];
  const seen = new Set<string>();
  const result: LockedPackage[] = [];
  while (pending.length) {
    const key = pending.pop()!;
    if (seen.has(key)) continue;
    seen.add(key);
    const base = key.split("(")[0];
    const at = base.lastIndexOf("@");
    const name = base.slice(0, at),
      version = base.slice(at + 1);
    const metadata = packages.get(base),
      snapshot = snapshots.get(key);
    assert.ok(metadata && snapshot, `missing stock dependency ${key}`);
    const integrity = /resolution: \{integrity: (sha512-[A-Za-z0-9+/]+=*)\}/u.exec(
      metadata.join("\n"),
    )?.[1];
    assert.ok(integrity);
    const dependencies: Record<string, string> = {};
    let active = false;
    for (const line of snapshot) {
      if (/^    \S/u.test(line)) {
        assert.ok(
          /^    (?:dependencies|optionalDependencies|transitivePeerDependencies):/u.test(line),
        );
        active = /^    (?:dependencies|optionalDependencies):/u.test(line);
      } else if (active && line.trim()) {
        const edge = /^      (?:'([^']+)'|([^:]+)): '?([^']+)'?$/u.exec(line);
        assert.ok(edge, `unrecognized stock dependency edge: ${line}`);
        const dependency = edge[1] ?? edge[2];
        dependencies[dependency] = edge[3];
        pending.push(`${dependency}@${edge[3]}`);
      }
    }
    result.push({ name, version, key, integrity, dependencies });
  }
  assert.equal(result.length, 26);
  return result.toSorted((left, right) =>
    left.name < right.name ? -1 : left.name > right.name ? 1 : 0,
  );
}

/** A caller-reviewed receipt digest freezes expectations before any provider runs. */
export function loadVueFixtureAuthority(options: {
  receiptPath: string;
  receiptSha256: string;
  sourceRoot: string;
  sourceHead: string;
}) {
  exact(options.receiptPath, options.receiptSha256);
  const receipt: VueFixtureAuthority = JSON.parse(fs.readFileSync(options.receiptPath, "utf8"));
  assert.equal(receipt.schema, "vize-stock-alias-vue-install-v1");
  assert.equal(options.sourceHead, originalHead);
  assert.equal(receipt.source.head, options.sourceHead);
  assert.deepEqual(receipt.source.files, originalFiles);
  assert.ok(path.isAbsolute(options.sourceRoot));
  assert.equal(fs.realpathSync(options.sourceRoot), options.sourceRoot);
  let lockSource = "";
  for (const [file, sha] of Object.entries(originalFiles)) {
    const read: SpawnSyncReturns<Buffer> = spawnSync(
      "git",
      ["show", `${options.sourceHead}:${file}`],
      {
        cwd: options.sourceRoot,
        maxBuffer: 16 * 1024 * 1024,
      },
    );
    assert.equal(read.error, undefined);
    assert.equal(read.status, 0);
    assert.equal(digest(read.stdout), sha);
    if (file === "pnpm-lock.yaml") lockSource = read.stdout.toString("utf8");
  }
  assert.deepEqual(receipt.lockedPackages, stockGraph(lockSource));
  assert.ok(path.isAbsolute(receipt.installRoot));
  assert.equal(fs.realpathSync(receipt.installRoot), receipt.installRoot);
  assert.equal(
    receipt.vueManifestPath,
    path.join(receipt.installRoot, "node_modules/vue/package.json"),
  );
  assert.equal(receipt.npm.exitCode, 0);
  assert.ok(receipt.npm.arguments.includes("--ignore-scripts"));
  assert.ok(receipt.npm.arguments.includes("--registry=https://registry.npmjs.org/"));
  const recheck = () => {
    exact(options.receiptPath, options.receiptSha256);
    exact(receipt.packageManifestPath, receipt.packageManifestSha256, receipt.installRoot);
    exact(receipt.packageLockPath, receipt.packageLockSha256, receipt.installRoot);
    exact(receipt.node.path, receipt.node.sha256);
    exact(receipt.npm.path, receipt.npm.sha256);
    const environment = {
      PATH: path.dirname(receipt.node.path),
      npm_config_userconfig: path.join(receipt.installRoot, "npm-empty.ini"),
      npm_config_globalconfig: path.join(receipt.installRoot, "npm-global-empty.ini"),
    };
    const node = spawnSync(receipt.node.path, ["--version"], {
      encoding: "utf8",
      env: environment,
    });
    assert.equal(node.error, undefined);
    assert.equal(node.status, 0);
    assert.equal(node.signal, null);
    assert.equal(node.stdout.trim(), receipt.node.version);
    const npm = spawnSync(receipt.node.path, [receipt.npm.path, "--version"], {
      cwd: receipt.installRoot,
      encoding: "utf8",
      env: environment,
    });
    assert.equal(npm.error, undefined);
    assert.equal(npm.status, 0);
    assert.equal(npm.signal, null);
    assert.equal(npm.stdout.trim(), receipt.npm.version);
    const manifest = JSON.parse(fs.readFileSync(receipt.packageManifestPath, "utf8"));
    assert.deepEqual(manifest.dependencies, { vue: "3.6.0-rc.6", typescript: "6.0.3" });
    assert.deepEqual(
      manifest.overrides,
      Object.fromEntries(receipt.lockedPackages.map((record) => [record.name, record.version])),
    );
    const lock = JSON.parse(fs.readFileSync(receipt.packageLockPath, "utf8"));
    const locked = new Map(receipt.lockedPackages.map((record) => [record.name, record]));
    assert.equal(Object.keys(lock.packages).length, locked.size + 1);
    assert.equal(receipt.registry.length, locked.size);
    const names = new Set<string>();
    for (const record of receipt.registry) {
      assert.ok(!names.has(record.name));
      names.add(record.name);
      const expected = locked.get(record.name);
      assert.ok(expected);
      assert.equal(record.version, expected.version);
      assert.equal(record.integrity, expected.integrity);
      assert.ok(record.resolved.startsWith("https://registry.npmjs.org/"));
      const installed = lock.packages[`node_modules/${record.name}`];
      assert.equal(installed.version, record.version);
      assert.equal(installed.resolved, record.resolved);
      assert.equal(installed.integrity, record.integrity);
      owned(
        path.join(receipt.installRoot, "node_modules", record.name, "package.json"),
        receipt.installRoot,
      );
    }
    const observed = runtimeGraph([receipt.vueManifestPath]);
    assert.equal(observed.length, 26);
    for (const record of observed) {
      const manifestPath = owned(String(record.manifestPath), receipt.installRoot);
      const manifest = record.manifest as { name: string; version: string };
      assert.equal(manifest.version, locked.get(manifest.name)?.version);
      for (const edge of record.edges as Array<{ state: string; manifestPath: string }>) {
        assert.equal(edge.state, "resolved");
        owned(edge.manifestPath, receipt.installRoot);
      }
      for (const payload of record.packagePayload as Array<{ path: string; kind: string }>) {
        owned(path.join(path.dirname(manifestPath), payload.path), receipt.installRoot);
      }
    }
    assert.equal(digest(JSON.stringify(receipt.runtimeGraph)), receipt.runtimeGraphSha256);
    assert.deepEqual(
      observed,
      receipt.runtimeGraph,
      "whole stock Vue runtime graph or payload changed",
    );
  };
  recheck();
  return { vuePath: path.dirname(fs.realpathSync(receipt.vueManifestPath)), receipt, recheck };
}
