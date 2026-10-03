import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import { gunzipSync } from "node:zlib";

const root = new URL("../../_fixtures/reference/vue2/runtime/", import.meta.url);
type Original = { path: string; bytes: number; sha256: string; gitBlobSha1: string };
type SourceFile = Original & {
  source: {
    originalPath?: string;
    originalBytes?: number;
    originalSha256?: string;
    originalGitBlobSha1?: string;
    encoding?: string;
  };
};
const provenance = JSON.parse(readFileSync(new URL("provenance.json", root), "utf8")) as {
  package: string;
  version: string;
  actualPackageMain: string;
  publishedIntegrity: string;
  sourceFiles: SourceFile[];
  fullPublishedInventory: {
    count: number;
    shards: Array<{ path: string; bytes: number; sha256: string; fileCount: number }>;
  };
};
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const blob = (bytes: Uint8Array) =>
  createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
assert.equal(provenance.package, "vue");
assert.equal(provenance.version, "2.7.16");
assert.equal(provenance.actualPackageMain, "dist/vue.runtime.common.js");
const inventory: Original[] = [];
for (const shard of provenance.fullPublishedInventory.shards) {
  const bytes = readFileSync(new URL(shard.path, root));
  assert.equal(bytes.length, shard.bytes);
  assert.equal(hash(bytes), shard.sha256);
  const data = JSON.parse(bytes.toString("utf8")) as {
    package: string;
    version: string;
    count: number;
    files: Original[];
  };
  assert.equal(data.package, "vue");
  assert.equal(data.version, "2.7.16");
  assert.equal(data.count, shard.fileCount);
  assert.equal(data.files.length, shard.fileCount);
  inventory.push(...data.files);
}
assert.equal(inventory.length, 228);
assert.equal(provenance.fullPublishedInventory.count, 228);
assert.equal(new Set(inventory.map(({ path }) => path)).size, 228);
const originals = new Map<string, Buffer>();
for (const file of provenance.sourceFiles) {
  const stored = readFileSync(new URL(file.path, root));
  assert.equal(stored.length, file.bytes, file.path);
  assert.equal(hash(stored), file.sha256, file.path);
  assert.equal(blob(stored), file.gitBlobSha1, file.path);
  const source =
    file.source.encoding === "gzip-original-complete-bytes" ? gunzipSync(stored) : stored;
  if (file.source.originalPath) {
    const entry = inventory.find(({ path }) => path === file.source.originalPath);
    assert.ok(entry, file.path);
    assert.equal(source.length, entry.bytes, file.path);
    assert.equal(hash(source), entry.sha256, file.path);
    assert.equal(blob(source), entry.gitBlobSha1, file.path);
    assert.equal(entry.bytes, file.source.originalBytes);
    assert.equal(entry.sha256, file.source.originalSha256);
    assert.equal(entry.gitBlobSha1, file.source.originalGitBlobSha1);
  }
  originals.set(file.path, source);
}
const pkg = JSON.parse(originals.get("package.json")!.toString("utf8"));
const registry = JSON.parse(originals.get("registry-original.json")!.toString("utf8"));
assert.equal(pkg.name, "vue");
assert.equal(pkg.version, "2.7.16");
assert.equal(pkg.main, provenance.actualPackageMain);
assert.equal(registry.name, "vue");
assert.equal(registry.version, "2.7.16");
const integrity =
  "sha512-4gCtFXaAA3zYZdTp5s4Hl2sozuySsgz4jy1EnpBHNfpMa9dK1ZCG7viqBPCwXtmgc8nHqUsAu3G4gtmXkkY3Sw==";
assert.equal(registry.dist.integrity, integrity);
assert.equal(provenance.publishedIntegrity, integrity);
const pins = {
  "dist/vue.runtime.common.js": "88f7fdf8e822f50d0446dcceb543df06db44653ed219bde5ffd2c095da832664",
  "dist/vue.runtime.common.dev.js":
    "1febb8ca712a76ec434096fc013edbd3883337a84225859f91a968ee315dbeb2",
  "dist/vue.runtime.common.prod.js":
    "58aedd341cf0d836b554b2a521b0d4c913dbff818fffa0660ad494e84980dc35",
} as const;
type Runtime = {
  new (options: {
    data: () => object;
    filters: Record<string, (...args: unknown[]) => unknown>;
    render: (...args: unknown[]) => unknown;
    staticRenderFns: Array<(...args: unknown[]) => unknown>;
  }): { _render(): { tag: string; children: Array<{ text: string }> } };
  version: string;
  config: { silent: boolean };
};

const loadRuntime = (environment: "test" | "production"): Runtime => {
  const loaded = new Map<string, unknown>();
  const load = (name: keyof typeof pins): unknown => {
    if (loaded.has(name)) return loaded.get(name);
    const original = originals.get(`${name}.gz`);
    assert.ok(original, name);
    assert.equal(hash(original), pins[name], name);
    const module = { exports: {} };
    const require = (id: string) => {
      assert.ok(name === pkg.main, "runtime body has no dependency imports");
      assert.ok(["./vue.runtime.common.dev.js", "./vue.runtime.common.prod.js"].includes(id));
      return load(`dist/${id.slice(2)}` as keyof typeof pins);
    };
    // Execute the literal npm main wrapper and its complete unchanged CJS body.
    vm.runInNewContext(
      original.toString("utf8"),
      {
        module,
        exports: module.exports,
        require,
        process: { env: { NODE_ENV: environment } },
        console,
        setTimeout,
        clearTimeout,
      },
      { timeout: 1000, filename: `actual-vue-2.7.16-${name}` },
    );
    loaded.set(name, module.exports);
    return module.exports;
  };
  const runtime = load(pkg.main) as Runtime;
  assert.equal(runtime.version, "2.7.16");
  assert.deepEqual(
    [...loaded.keys()].sort((left, right) => left.localeCompare(right)),
    [pkg.main, `dist/vue.runtime.common.${environment === "production" ? "prod" : "dev"}.js`].sort(
      (left, right) => String(left).localeCompare(String(right)),
    ),
  );
  runtime.config.silent = true;
  return runtime;
};

export { loadRuntime };
