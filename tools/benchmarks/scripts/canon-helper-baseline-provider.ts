/** Source-image and physical SDK custody for the untimed helper carrier. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { hash, identify, save } from "./canon-instance-directives-io.ts";
import { isRecord, jsonValue } from "./canon-instance-directives-model.ts";
import { nativeProxyLauncher } from "./canon-helper-native-proxy.ts";

export const sourceRevision = "fb550c424e4256860647dbf96ce32c31cea3774b";
export const binarySha256 = "cdf44149ddfe160339887a256c1bd0253fb34f1f7c774759e6c5382b42ed0cb9";
export const packageVersions = {
  vize: "0.602.0",
  vue: "3.5.42",
  typescript: "6.0.3",
  "typescript-native": "7.1.0-beta",
};
function readJson(file: string): Record<string, unknown> {
  const value = jsonValue(fs.readFileSync(file, "utf8"));
  assert.ok(isRecord(value));
  return value;
}
export async function prepareHelperProvider(provider: string, binary: string, out: string) {
  assert.equal(identify(binary).sha256, binarySha256);
  const sidecar = readJson(binary + ".differential-build.json");
  assert.equal(sidecar.sourceRevision, sourceRevision);
  save(path.join(out, "source-image-sidecar.json"), sidecar);
  const packages = [];
  for (const [name, version] of Object.entries(packageVersions)) {
    const file = path.join(provider, "node_modules", name, "package.json");
    const manifest = readJson(file);
    assert.equal(manifest.version, version, name);
    packages.push({ name, manifest: identify(file), version });
  }
  const getterFile = path.join(provider, "node_modules/typescript-native/lib/getExePath.js");
  const imported: unknown = await import(pathToFileURL(getterFile).href);
  assert.ok(isRecord(imported) && typeof imported.default === "function");
  const candidate: unknown = Reflect.apply(imported.default, imported, []);
  assert.equal(typeof candidate, "string");
  assert.ok(typeof candidate === "string");
  const checker = fs.realpathSync(candidate);
  const helper = path.join(provider, "node_modules/vize/content-mapper-helpers.d.ts");
  assert.ok(fs.statSync(helper).isFile());
  const generated = path.join(provider, "node_modules/vize/generated/content-mapper-types.d.ts");
  const vue = fs.realpathSync(path.join(provider, "node_modules/vue/package.json"));
  const receipt = {
    sourceRevision,
    sourceRootAttested: false,
    binary: identify(binary),
    runtimeVersion: "0.474.0-dev",
    imageRun: 40570360508,
    imageArtifact: 12765860439,
    sourceRole: "normal shipping-profile source image; not published 0.602 executable",
    packages,
    lock: identify(path.join(provider, "package-lock.json")),
    checker: identify(checker),
    getter: identify(getterFile),
    physicalVue: identify(vue),
    helperEntry: identify(helper),
    helperGenerated: identify(generated),
    helperCapability:
      "matched physical Vue and published SDK53; explicit source mapper exec override",
    normalCargoFeatures: ["default", "glyph", "maestro"],
    allocationCounts: null,
    allocationMeaning: "normal release allocator is not instrumented",
  };
  save(path.join(out, "provider.json"), receipt);
  return { checker, receipt };
}

/** Replace only this fresh project's generated dependency symlink; leave the provider untouched. */
export function sourceMapperSdk(project: string, provider: string, executable: string[]) {
  const modules = path.join(project, "node_modules");
  assert.ok(fs.lstatSync(modules).isSymbolicLink());
  assert.equal(fs.realpathSync(modules), fs.realpathSync(path.join(provider, "node_modules")));
  fs.unlinkSync(modules);
  fs.mkdirSync(modules);
  for (const entry of fs.readdirSync(path.join(provider, "node_modules"))) {
    if (entry === "vize") continue;
    fs.symlinkSync(path.join(provider, "node_modules", entry), path.join(modules, entry), "dir");
  }
  const original = path.join(provider, "node_modules/vize");
  const shadow = path.join(modules, "vize");
  fs.mkdirSync(shadow);
  for (const entry of fs.readdirSync(original)) {
    if (entry !== "package.json")
      fs.symlinkSync(path.join(original, entry), path.join(shadow, entry));
  }
  const manifestFile = path.join(original, "package.json");
  const before = hash(fs.readFileSync(manifestFile)),
    manifest = readJson(manifestFile);
  assert.ok(isRecord(manifest.typescript) && isRecord(manifest.typescript.contentMapper));
  const originalExec = manifest.typescript.contentMapper.exec;
  assert.ok(Array.isArray(originalExec) && originalExec.every((item) => typeof item === "string"));
  manifest.typescript.contentMapper.exec = executable;
  save(path.join(shadow, "package.json"), manifest);
  assert.equal(hash(fs.readFileSync(manifestFile)), before);
  const vueFromProject = fs.realpathSync(path.join(modules, "vue/package.json"));
  assert.equal(
    vueFromProject,
    fs.realpathSync(path.join(provider, "node_modules/vue/package.json")),
  );
  return {
    originalManifest: identify(manifestFile),
    shadowManifest: identify(path.join(shadow, "package.json")),
    originalExec,
    executable,
    physicalVue: vueFromProject,
  };
}
export function observerLauncher(out: string, binary: string, mode: "rpc" | "stats") {
  const executable = path.join(out, mode === "stats" ? "tsgo" : "mapper");
  fs.mkdirSync(out, { recursive: true });
  const module = path.resolve("tools/benchmarks/scripts/canon-helper-native-proxy.ts");
  fs.writeFileSync(
    executable,
    nativeProxyLauncher({
      node: process.execPath,
      module,
      out: path.join(out, "capture"),
      executable: binary,
      mode,
    }),
    { mode: 0o755 },
  );
  return executable;
}
