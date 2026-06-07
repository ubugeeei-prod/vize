import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

interface ExportEntry {
  default?: string;
  import?: string;
  types?: string;
}

interface BundlerPluginManifest {
  engines?: Record<string, string>;
  exports?: Record<string, ExportEntry | string>;
  name?: string;
  peerDependencies?: Record<string, string>;
}

function readManifest(packageDir: string): BundlerPluginManifest {
  return JSON.parse(
    fs.readFileSync(path.join(root, "npm", packageDir, "package.json"), "utf-8"),
  ) as BundlerPluginManifest;
}

function readWorkspaceYaml(): string {
  return fs.readFileSync(path.join(root, "pnpm-workspace.yaml"), "utf-8");
}

function catalogPin(workspaceYaml: string, catalog: string, name: string): string | undefined {
  const lines = workspaceYaml.split("\n");
  const header = `  ${catalog}:`;
  const start = lines.findIndex((line) => line === header);
  if (start === -1) return undefined;

  for (const line of lines.slice(start + 1)) {
    if (/^\s{0,2}\S/.test(line) && !/^\s{4}/.test(line)) break;
    const match = line.match(/^\s{4}"?([^":]+)"?:\s*"([^"]+)"\s*$/);
    if (match && match[1] === name) {
      return match[2];
    }
  }
  return undefined;
}

function leadingMajor(pin: string): number {
  const match = pin.match(/(\d+)\./);
  assert.ok(match, `expected a leading major number in catalog pin "${pin}"`);
  return Number(match[1]);
}

test("@vizejs/unplugin exposes per-bundler subpath entries wired at mjs/d.mts", () => {
  const manifest = readManifest("unplugin-vize");
  assert.equal(manifest.name, "@vizejs/unplugin");

  for (const subpath of ["./esbuild", "./rollup", "./rolldown", "./webpack", "./babel"]) {
    const entry = manifest.exports?.[subpath];
    assert.ok(entry, `missing export subpath ${subpath}`);
    assert.equal(typeof entry, "object", `export subpath ${subpath} must be a conditions object`);

    const conditions = entry as ExportEntry;
    assert.ok(
      conditions.import?.endsWith(".mjs"),
      `${subpath} import should point at a .mjs file, got ${conditions.import}`,
    );
    assert.ok(
      conditions.types?.endsWith(".d.mts"),
      `${subpath} types should point at a .d.mts file, got ${conditions.types}`,
    );
  }
});

test("@vizejs/unplugin declares the webpack peer range and the workspace pins webpack 5.x", () => {
  const manifest = readManifest("unplugin-vize");
  assert.equal(manifest.peerDependencies?.webpack, "^4.46.0 || ^5.0.0");

  const webpackPin = catalogPin(readWorkspaceYaml(), "bundlers", "webpack");
  assert.ok(webpackPin, "webpack catalog pin (bundlers)");
  assert.equal(leadingMajor(webpackPin), 5, `webpack catalog pin ${webpackPin} should be 5.x`);
});

test("@vizejs/rspack-plugin peer range matches the catalog @rspack/core major", () => {
  const manifest = readManifest("rspack-vize-plugin");
  assert.equal(manifest.name, "@vizejs/rspack-plugin");
  assert.equal(manifest.peerDependencies?.["@rspack/core"], "^1.0.0 || ^2.0.0");

  const rspackPin = catalogPin(readWorkspaceYaml(), "bundlers", "@rspack/core");
  assert.ok(rspackPin, "@rspack/core catalog pin (bundlers)");
  const major = leadingMajor(rspackPin);
  assert.ok(
    major === 1 || major === 2,
    `@rspack/core catalog pin ${rspackPin} major ${major} should be 1 or 2`,
  );
});

test("@vizejs/vite-plugin peers vite ^8 while the workspace resolves vite via the VoidZero proxy", () => {
  const manifest = readManifest("vite-plugin-vize");
  assert.equal(manifest.name, "@vizejs/vite-plugin");
  assert.equal(manifest.peerDependencies?.vite, "^8.0.0");

  const vitePin = catalogPin(readWorkspaceYaml(), "vite-stack", "vite");
  assert.ok(vitePin, "vite catalog pin (vite-stack)");
  assert.ok(
    vitePin.startsWith("npm:@voidzero-dev/vite-plus-core@"),
    `vite catalog pin should alias the VoidZero proxy, got ${vitePin}`,
  );
});

test("all three bundler-plugin packages require Node >= 22", () => {
  for (const packageDir of ["unplugin-vize", "rspack-vize-plugin", "vite-plugin-vize"]) {
    const manifest = readManifest(packageDir);
    const nodeEngine = manifest.engines?.node;
    assert.ok(nodeEngine, `${packageDir} engines.node`);
    assert.match(
      nodeEngine,
      /22/,
      `${packageDir} engines.node ${nodeEngine} should reference Node 22`,
    );
    assert.match(
      nodeEngine,
      /^>=?\s*22/,
      `${packageDir} engines.node ${nodeEngine} should be a >= form`,
    );
  }
});
