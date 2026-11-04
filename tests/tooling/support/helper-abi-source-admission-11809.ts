import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type Contract = {
  schema: number;
  modules: { subpath: string; exports: unknown; files: { path: string; sha256: string }[] }[];
};
const contractFile = fileURLToPath(
  new URL("../../../crates/vize/src/commands/content_mapper/helper-abi.json", import.meta.url),
);
const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest();

/** Closed makeSourceProject profile; the published before oracle remains separate. */
export function assertSourceProtocolAdmission(responses: readonly unknown[], configFile: string) {
  const config = fs.realpathSync(configFile);
  const project = path.dirname(configFile);
  assert.equal(path.resolve(project), fs.realpathSync(project), "physical source fixture root");
  const settings = JSON.parse(fs.readFileSync(config, "utf8")) as Record<string, unknown>;
  assert.deepEqual(settings.compilerOptions, {
    module: "preserve",
    moduleResolution: "bundler",
    strict: true,
    noEmit: true,
    jsx: "preserve",
    jsxImportSource: "vue",
    lib: ["ESNext", "DOM"],
    types: [],
  });
  assert.equal(settings.extends, undefined);
  assert.equal(settings.vueCompilerOptions, undefined);
  const sdk = path.join(project, "node_modules/vize");
  assert.equal(fs.realpathSync(sdk), sdk, "owned source SDK directory");
  const manifestFile = path.join(sdk, "package.json");
  const manifestBytes = fs.readFileSync(manifestFile);
  const manifest = JSON.parse(manifestBytes.toString()) as {
    name: string;
    exports: Record<string, unknown>;
    typesVersions?: unknown;
  };
  assert.equal(manifest.name, "vize");
  assert.equal(manifest.typesVersions, undefined);
  const contract = JSON.parse(fs.readFileSync(contractFile, "utf8")) as Contract;
  assert.equal(contract.schema, 1);
  assert.deepEqual(
    contract.modules.map((row) => row.subpath),
    ["./content-mapper-helpers", "./virtual-ts", "./virtual-ts-checker"],
  );

  const watched = new Set([config]);
  const fingerprint = createHash("sha256");
  function selectedPackage(logical: string) {
    const metadata = fs.realpathSync(logical);
    const directory = fs.realpathSync(path.dirname(logical));
    watched.add(logical);
    watched.add(metadata);
    fingerprint.update(logical + "\0");
    fingerprint.update(metadata + "\0");
    fingerprint.update(directory + "\0");
    fingerprint.update(digest(fs.readFileSync(metadata)));
    return directory;
  }
  function absent(logical: string) {
    assert.equal(fs.existsSync(logical), false, "declared missing source lookup");
    watched.add(logical);
    fingerprint.update(logical + "\0missing\0");
  }
  assert.equal(selectedPackage(manifestFile), sdk);
  fingerprint.update(sdk);
  fingerprint.update(manifestBytes);
  const declarations = new Set<string>();
  for (const module of contract.modules) {
    assert.deepEqual(manifest.exports[module.subpath], module.exports);
    for (const declaration of module.files) {
      if (declarations.has(declaration.path)) continue;
      declarations.add(declaration.path);
      const file = path.join(sdk, declaration.path);
      const bytes = digest(fs.readFileSync(file));
      assert.equal(bytes.toString("hex"), declaration.sha256, "qualified source declaration");
      watched.add(file);
      fingerprint.update(declaration.path);
      fingerprint.update(bytes);
    }
  }
  const vue = path.join(project, "node_modules/vue/package.json");
  const projectVue = selectedPackage(vue);
  absent(path.join(sdk, "node_modules/vue/package.json"));
  absent(path.join(project, "node_modules/node_modules/vue/package.json"));
  assert.equal(selectedPackage(vue), projectVue);
  // The seeded flat profile owns all three helper imports and the same Vue.
  // Selection is fixed by those inputs, never borrowed from the mapper reply.
  fingerprint.update(new Uint8Array([1, 1]));
  const base = createHash("sha256")
    .update(JSON.stringify([{}, [config]]))
    .digest("hex");
  assert.deepEqual(responses[0], {
    id: 1,
    jsonrpc: "2.0",
    result: { diagnosticSource: "vize", positionEncoding: "utf-8" },
  });
  assert.deepEqual(responses[1], {
    id: 2,
    jsonrpc: "2.0",
    result: {
      configIdentity: `vize-content-mapper-v1:${base}:sdk:${fingerprint.digest("hex")}`,
      watchedFiles: [...watched].sort(),
    },
  });
}
