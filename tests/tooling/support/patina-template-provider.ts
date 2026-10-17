import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { gunzipSync } from "node:zlib";
import provenance from "../../_fixtures/differential/lint/legacy-instance-apis-9217/template-compiler-provenance.json" with { type: "json" };

type CompilerDom = typeof import("@vue/compiler-dom");
type CompilerSfc = typeof import("vue-legacy-instance-oracle/compiler-sfc");
interface CompilerProvider {
  vue: string;
  compiler: string;
  typescript?: string;
  sources: { packageName: string; path: string; sha256: string }[];
}
interface Providers<Compiler> {
  compiler: Compiler;
  vueRuntime: typeof import("vue-legacy-instance-oracle");
  typescript: typeof import("typescript");
  requireVuePackage: ReturnType<typeof createRequire>;
}
const root = path.resolve(import.meta.dirname, "../../..");
const fixtures = path.join(root, "tests/_fixtures/differential/lint/legacy-instance-apis-9217");
const hash = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");

function version(manifest: string): string {
  const value: unknown = JSON.parse(fs.readFileSync(manifest, "utf8"));
  assert.ok(value !== null && typeof value === "object" && "version" in value);
  assert.ok(typeof value.version === "string");
  return value.version;
}

function declaredDependency(requireTests: ReturnType<typeof createRequire>, name: string): string {
  const value: unknown = requireTests("./package.json");
  assert.ok(value !== null && typeof value === "object" && "devDependencies" in value);
  const dependencies = value.devDependencies;
  assert.ok(dependencies !== null && typeof dependencies === "object" && name in dependencies);
  const dependency: unknown = Reflect.get(dependencies, name);
  assert.ok(typeof dependency === "string");
  return dependency;
}

/** Preserve historical source custody while loading only the actual pinned compiler closure. */
export function loadFrozenTemplateCompiler(
  file: string,
  provider: CompilerProvider,
  kind: "dom",
): Providers<CompilerDom>;
export function loadFrozenTemplateCompiler(
  file: string,
  provider: CompilerProvider,
  kind: "sfc",
): Providers<CompilerSfc>;
export function loadFrozenTemplateCompiler(
  file: string,
  provider: CompilerProvider,
  kind: "dom" | "sfc",
): Providers<CompilerDom | CompilerSfc> {
  assert.equal(provenance.historicalCompiledClosure, "unknown");
  const frozen = provenance.frozenPackets.find((packet) => packet.file === file);
  assert.ok(frozen, "only the five immutable compiler cohorts are reviewed");
  const bytes = fs.readFileSync(path.join(fixtures, file));
  assert.equal(bytes.length, frozen.bytes);
  assert.equal(hash(bytes), frozen.sha256);
  let recorded: unknown = JSON.parse(
    (file.endsWith(".gz") ? gunzipSync(bytes) : bytes).toString("utf8"),
  );
  for (const segment of frozen.providerPath) {
    assert.ok(recorded !== null && typeof recorded === "object");
    recorded = Reflect.get(recorded, segment);
  }
  assert.deepEqual(provider, recorded, "the provider must belong to these exact frozen bytes");

  const requireTests = createRequire(path.join(root, "tests/package.json"));
  const historical = provenance.recordedArtifact;
  assert.equal(
    declaredDependency(requireTests, historical.dependency),
    `npm:${historical.packageName}@${historical.version}`,
  );
  const historicalManifest = requireTests.resolve(`${historical.dependency}/package.json`);
  assert.equal(version(historicalManifest), historical.version);
  const historicalSource = path.join(path.dirname(historicalManifest), historical.path);
  assert.equal(hash(fs.readFileSync(historicalSource)), historical.sha256);
  // Read this artifact as bytes; never require or compile its implementation.

  const current = provenance.currentReplay;
  assert.equal(provider.vue, current.vue);
  assert.equal(provider.compiler, current.vue);
  if (provider.typescript !== undefined) assert.equal(provider.typescript, current.typescript);
  const externalManifest = process.env.VIZE_LEGACY_API_VUE_MANIFEST;
  const requireVue = createRequire(externalManifest ?? path.join(root, "tests/package.json"));
  const vuePackage = externalManifest ? "vue" : current.dependency;
  if (!externalManifest) {
    assert.equal(declaredDependency(requireTests, vuePackage), `npm:vue@${current.vue}`);
  }
  const vueManifest = requireVue.resolve(`${vuePackage}/package.json`);
  assert.equal(version(vueManifest), current.vue);
  const requireVuePackage = createRequire(vueManifest);
  const sfcManifest = requireVuePackage.resolve("@vue/compiler-sfc/package.json");
  const requireSfc = createRequire(sfcManifest);
  const domManifest = requireSfc.resolve("@vue/compiler-dom/package.json");
  const requireDom = createRequire(domManifest);
  const coreManifest = requireDom.resolve("@vue/compiler-core/package.json");
  assert.equal(requireSfc.resolve("@vue/compiler-core"), requireDom.resolve("@vue/compiler-core"));
  const manifests = new Map([
    ["@vue/compiler-sfc", sfcManifest],
    ["@vue/compiler-dom", domManifest],
    ["@vue/compiler-core", coreManifest],
  ]);
  const implementations = new Map<string, string>();
  for (const source of current.sources) {
    const manifest = manifests.get(source.packageName);
    assert.ok(manifest);
    assert.equal(version(manifest), source.version);
    const implementation = path.join(path.dirname(manifest), source.path);
    assert.equal(hash(fs.readFileSync(implementation)), source.sha256);
    implementations.set(source.packageName, implementation);
  }
  // Every original recorded hash remains checked, with its honest artifact role.
  for (const source of provider.sources) {
    const implementation =
      source.packageName === historical.packageName
        ? historicalSource
        : implementations.get(source.packageName);
    assert.ok(implementation);
    assert.equal(hash(fs.readFileSync(implementation)), source.sha256);
  }

  const compilerSfc: CompilerSfc = requireSfc("@vue/compiler-sfc");
  const compilerDom: CompilerDom = requireDom("@vue/compiler-dom");
  const coreEntry = requireDom.resolve("@vue/compiler-core");
  const domEntry = requireSfc.resolve("@vue/compiler-dom");
  for (const [owner, child] of [
    [implementations.get("@vue/compiler-sfc"), coreEntry],
    [implementations.get("@vue/compiler-sfc"), domEntry],
    [implementations.get("@vue/compiler-dom"), coreEntry],
    [coreEntry, implementations.get("@vue/compiler-core")],
  ]) {
    assert.ok(owner && child);
    assert.ok(requireSfc.cache[owner]?.children.some((module) => module.filename === child));
  }
  const typescript: typeof import("typescript") = requireVuePackage("typescript");
  assert.equal(typescript.version, current.typescript);
  const vueRuntime: typeof import("vue-legacy-instance-oracle") = requireVue(vuePackage);
  assert.equal(vueRuntime.version, current.vue);
  assert.equal(
    requireVuePackage("vue"),
    vueRuntime,
    "compiled require uses the same real Vue instance",
  );
  return {
    compiler: kind === "dom" ? compilerDom : compilerSfc,
    vueRuntime,
    typescript,
    requireVuePackage,
  };
}
