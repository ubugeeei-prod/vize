import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function read(relativePath: string): string {
  return fs.readFileSync(path.join(root, relativePath), "utf8");
}

test("workerd build keeps the focused artifact separate from the browser artifact", () => {
  const manifest = read("crates/vize_vitrine/Cargo.toml");
  const wasmModule = read("crates/vize_vitrine/src/wasm.rs");
  const build = read("tools/moon/cmd/build_vize_wasm_package/main.mbt");

  assert.match(manifest, /^workerd = \[/m);
  assert.match(manifest, /^wasm = \["workerd", "glyph"\]$/m);
  assert.match(wasmModule, /#\[cfg\(feature = "wasm"\)\]\nmod lint;/);
  assert.match(wasmModule, /#\[cfg\(feature = "wasm"\)\]\nmod musea;/);
  assert.match(build, /"--features",\n\s+"wasm"/);
  assert.match(build, /"--features",\n\s+"workerd"/);
  assert.match(build, /"--out-name",\n\s+"vize_workerd"/);
});

test("Cloudflare Worker lazily instantiates the compiled module and caches warm requests", () => {
  const entry = read("examples/cloudflare-worker/src/index.js");
  const config = read("examples/cloudflare-worker/wrangler.jsonc");

  assert.match(entry, /import vizeWasm from "@vizejs\/wasm\/wasm\.wasm";/);
  assert.match(entry, /import \{ instantiate \} from "@vizejs\/wasm\/workerd";/);
  assert.match(entry, /bindingPromise \?\?= instantiate\(vizeWasm\);/);
  assert.match(config, /"type": "CompiledWasm"/);
  assert.match(config, /"globs": \["\*\*\/\*\.wasm"\]/);
});

test("PR CI builds, smoke-tests, and bundles the Cloudflare Worker", () => {
  const workflow = read(".github/workflows/cloudflare-worker.yml");

  assert.match(workflow, /^ {2}pull_request:\n {4}branches: \[main\]$/m);
  assert.match(workflow, /^jobs:\n {2}cloudflare-worker:$/m);
  assert.match(workflow, /targets: wasm32-unknown-unknown/);
  assert.match(workflow, /moon run --target native tools\/moon\/cmd\/build_vize_wasm_package --/);
  assert.match(workflow, /node tools\/npm\/smoke-wasm-package\.mjs npm\/wasm/);
  assert.match(workflow, /vp run --filter vize-cloudflare-worker-example check/);
});

test("wasm package publishes the workerd entrypoint and focused artifact", () => {
  const wasmPackage = JSON.parse(read("npm/wasm/package.json")) as {
    exports?: Record<string, unknown>;
    files?: string[];
  };

  assert.deepEqual(wasmPackage.exports?.["./workerd"], {
    types: "./workerd.d.ts",
    import: "./workerd.js",
    default: "./workerd.js",
  });
  assert.equal(wasmPackage.exports?.["./wasm.wasm"], "./vize_workerd_bg.wasm");

  for (const file of [
    "README.md",
    "workerd.js",
    "workerd.d.ts",
    "vize_workerd.js",
    "vize_workerd.d.ts",
    "vize_workerd_bg.wasm",
  ]) {
    assert.ok(wasmPackage.files?.includes(file), `@vizejs/wasm files include ${file}`);
  }
});
