import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import { gunzipSync } from "node:zlib";

const root = new URL("../../_fixtures/reference/vue2/", import.meta.url);
type PackageName = "vue-template-compiler" | "de-indent" | "he";
type OriginalFile = { path: string; bytes: number; sha256: string; gitBlobSha1: string };
type FixtureFile = OriginalFile & {
  source: {
    package: PackageName;
    path?: string;
    encoding?: string;
    originalBytes?: number;
    originalSha256?: string;
    originalGitBlobSha1?: string;
  };
};
type PackageReceipt = {
  name: PackageName;
  version: string;
  publishedIntegrity: string;
  fullPublishedFileInventoryFile: { path: string; bytes: number; sha256: string };
};
const provenance = JSON.parse(readFileSync(new URL("provenance.json", root), "utf8")) as {
  packages: PackageReceipt[];
  files: FixtureFile[];
};
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const blob = (bytes: Uint8Array) =>
  createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
const originals = new Map<string, Buffer>();
for (const pkg of provenance.packages) {
  const receipt = pkg.fullPublishedFileInventoryFile;
  const bytes = readFileSync(new URL(receipt.path, root));
  assert.equal(bytes.length, receipt.bytes);
  assert.equal(hash(bytes), receipt.sha256);
  const inventory = JSON.parse(bytes.toString("utf8")) as {
    name: string;
    version: string;
    files: OriginalFile[];
  };
  assert.equal(inventory.name, pkg.name);
  assert.equal(inventory.version, pkg.version);
  assert.equal(
    inventory.files.length,
    { "vue-template-compiler": 7, "de-indent": 4, he: 6 }[pkg.name],
  );
  for (const file of provenance.files.filter(
    (file) => file.source.package === pkg.name && file.source.path,
  )) {
    const original = inventory.files.find((entry) => entry.path === file.source.path);
    assert.ok(original, file.path);
    assert.equal(original.bytes, file.source.originalBytes);
    assert.equal(original.sha256, file.source.originalSha256);
    assert.equal(original.gitBlobSha1, file.source.originalGitBlobSha1);
  }
}
for (const file of provenance.files) {
  const stored = readFileSync(new URL(file.path, root));
  assert.equal(stored.length, file.bytes, file.path);
  assert.equal(hash(stored), file.sha256, file.path);
  assert.equal(blob(stored), file.gitBlobSha1, file.path);
  const source =
    file.source.encoding === "gzip-original-complete-bytes" ? gunzipSync(stored) : stored;
  if (file.source.originalBytes !== undefined) {
    assert.equal(source.length, file.source.originalBytes, file.path);
    assert.equal(hash(source), file.source.originalSha256, file.path);
    assert.equal(blob(source), file.source.originalGitBlobSha1, file.path);
  }
  originals.set(file.path, source);
}
const exact = (name: string, sha256: string) => {
  const source = originals.get(name);
  assert.ok(source, name);
  assert.equal(hash(source), sha256, name);
  return source.toString("utf8");
};
const source = exact(
  "build.js.gz",
  "ca057518797901b3879d7d47049eaed969fafe4d5700e271f870909290d00f90",
);
assert.equal(Buffer.byteLength(source), 227831);
assert.equal(blob(Buffer.from(source)), "eaabc56a0a31f434a4c9ac3374a2f3a208d822e2");
const dependencies = new Map([
  [
    "de-indent",
    exact(
      "dependencies/de-indent/index.js.gz",
      "6cfabc60c7a069b9ca720c15be6a33a05c7913655c837102e9f10c10c81880db",
    ),
  ],
  [
    "he",
    exact(
      "dependencies/he/he.js.gz",
      "76c554d5bbfd032fe620595076a50abea5124b9cbd4e9ffe6ac94a4f855aeceb",
    ),
  ],
]);
for (const [name, version, integrity, path] of [
  [
    "vue-template-compiler",
    "2.7.16",
    "sha512-AYbUWAJHLGGQM7+cNTELw+KsOG9nl2CnSv467WobS5Cv9uk3wFcnr1Etsz2sEIHEZvw1U+o9mRlEO6QbZvUPGQ==",
    "",
  ],
  [
    "de-indent",
    "1.0.2",
    "sha512-e/1zu3xH5MQryN2zdVaF0OrdNLUbvWxzMbi+iNA6Bky7l1RoP8a2fIbRocyHclXt/arDrrR6lL3TqFD9pMQTsg==",
    "dependencies/de-indent/",
  ],
  [
    "he",
    "1.2.0",
    "sha512-F/1DnUGPopORZi0ni+CvrCgHQ5FyEAHRLSApuYWMmrbSwoN2Mn/7k+Gl38gJnR7yyDZk6WLXwiGod1JOWNDKGw==",
    "dependencies/he/",
  ],
]) {
  const registry = JSON.parse(readFileSync(new URL(`${path}registry-original.json`, root), "utf8"));
  const pkg = JSON.parse(readFileSync(new URL(`${path}package.json`, root), "utf8"));
  assert.equal(pkg.name, name);
  assert.equal(pkg.version, version);
  assert.equal(registry.name, name);
  assert.equal(registry.version, version);
  assert.equal(registry.dist.integrity, integrity);
  assert.equal(
    provenance.packages.find((entry) => entry.name === name)?.publishedIntegrity,
    integrity,
  );
}
const modules = new Map<string, unknown>();
const loadDependency = (name: string) => {
  if (modules.has(name)) return modules.get(name);
  const bytes = dependencies.get(name);
  assert.ok(bytes, `unexpected oracle dependency ${name}`);
  const module = { exports: {} };
  vm.runInNewContext(
    bytes,
    { module, exports: module.exports },
    { timeout: 1000, filename: `actual-${name}.cjs` },
  );
  modules.set(name, module.exports);
  return module.exports;
};
const module = { exports: {} };
const context = vm.createContext({
  module,
  exports: module.exports,
  require: loadDependency,
  process: { env: { NODE_ENV: "test" } },
  console,
});
// Run every original byte. A separate read-only expression borrows the actual
// private upstream functions; no function body is extracted or rewritten.
vm.runInContext(source, context, {
  timeout: 1000,
  filename: "actual-vue-template-compiler-2.7.16.cjs",
});
const parser = vm.runInContext("({ parseText, parseFilters })", context) as {
  parseText(source: string): unknown;
  parseFilters(source: string): string;
};
const compiler = module.exports as {
  compile(
    source: string,
    options?: object,
  ): {
    ast: { children: Array<{ type: number; expression?: string; tokens?: unknown }> };
    errors: unknown[];
    render: string;
  };
  compileToFunctions(source: string): {
    render: (...args: unknown[]) => unknown;
    staticRenderFns: Array<(...args: unknown[]) => unknown>;
  };
};
const plain = (value: unknown) => (value === undefined ? null : JSON.parse(JSON.stringify(value)));

export { compiler, root, parser, plain };
