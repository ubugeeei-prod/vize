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
  const workflow = read(".github/workflows/check.yml");
  const job = workflow.match(/\n  cloudflare-worker:\n([\s\S]*?)\n  playground-test:/)?.[1];

  assert.ok(job, "cloudflare-worker job should exist");
  assert.match(job, /targets: wasm32-unknown-unknown/);
  assert.match(job, /moon run --target native tools\/moon\/cmd\/build_vize_wasm_package --/);
  assert.match(job, /node tools\/npm\/smoke-wasm-package\.mjs npm\/wasm/);
  assert.match(job, /vp run --filter vize-cloudflare-worker-example check/);
  assert.match(workflow, /\n      - cloudflare-worker\n      - playground-test/);
});
