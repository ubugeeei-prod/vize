// Audited fact tables persist independently of the rule diagnostic cache.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const child = `
  import { createRequire } from "node:module";
  import { defineFactProvider, definePlugin } from ${JSON.stringify(pathToFileURL(path.join(root, "npm/plugin-sdk/index.js")).href)};
  const native = createRequire(import.meta.url)(${JSON.stringify(path.join(root, "npm/native/index.js"))});
  const color = process.argv[2];
  let calls = 0;
  const provider = defineFactProvider({
    name: "tokens", version: "disk-provider-1", visit: ["ui.element"],
    provides: ["tokens/colors"], cacheInputs: [{name:"palette", value:color}],
    provide(batch) { calls += 1; return {"tokens/colors": batch.nodes.map(node => [node.id, color])}; }
  });
  const rule = definePlugin({
    name: "disk-fact-rule", version: "1", visit: ["ui.element"],
    demands: ["tokens/colors"], cacheInputs: [],
    rules: { contrast(ctx) {
      const colors = ctx.facts("tokens/colors");
      for (const node of ctx.nodes) ctx.report(node, colors.get(node.id));
    }}
  });
  const result = native.lintWithPlugins("<template><button>Go</button></template>", [rule], {
    filename:"DiskProvider.vue", cache:true, cacheDir:process.argv[1], factProviders:[provider]
  });
  process.stdout.write(JSON.stringify({calls, diagnostics:result.diagnostics,
    provider:result.factProviders[0], rule:result.plugins[0]}));
`;

function run(dir: string, color: string) {
  const result = spawnSync(process.execPath, ["--input-type=module", "-e", child, dir, color], {
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}

test("providers and their consumer rules persist across Node processes under complete manifests", () => {
  const dir = mkdtempSync(path.join(tmpdir(), "vize-fact-provider-cache-"));
  try {
    const first = run(dir, "red");
    const warm = run(dir, "red");
    const changed = run(dir, "blue");
    const changedWarm = run(dir, "blue");
    assert.deepEqual([first.calls, warm.calls, changed.calls, changedWarm.calls], [2, 0, 2, 0]);
    assert.deepEqual(
      [
        first.provider.cached,
        warm.provider.cached,
        changed.provider.cached,
        changedWarm.provider.cached,
      ],
      [false, true, false, true],
    );
    assert.deepEqual(
      [first.rule.cached, warm.rule.cached, changed.rule.cached, changedWarm.rule.cached],
      [false, true, false, true],
    );
    assert.equal(first.provider.contentKey, warm.provider.contentKey);
    assert.equal(first.rule.contentKey, warm.rule.contentKey);
    assert.notEqual(first.provider.contentKey, changed.provider.contentKey);
    assert.notEqual(first.rule.contentKey, changed.rule.contentKey);
    assert.deepEqual(first.diagnostics, warm.diagnostics);
    assert.equal(changed.diagnostics[0].message, "blue");
    assert.equal(warm.provider.jsNs, 0);
    assert.equal(warm.provider.audited, false);
    assert.equal(readdirSync(dir).filter((file) => file.startsWith("plugin-facts-")).length, 2);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("a corrupted provider table is recomputed before a consumer diagnostic cache lookup", () => {
  const dir = mkdtempSync(path.join(tmpdir(), "vize-fact-provider-corrupt-"));
  try {
    const first = run(dir, "red");
    const file = readdirSync(dir).find((name) => name.startsWith("plugin-facts-"));
    assert.ok(file);
    const at = path.join(dir, file);
    const entry = JSON.parse(readFileSync(at, "utf8"));
    entry.values["tokens/colors"] = [[0, "tampered"]];
    writeFileSync(at, JSON.stringify(entry));
    const recovered = run(dir, "red");
    assert.equal(recovered.calls, 2);
    assert.equal(recovered.provider.cached, false);
    assert.equal(recovered.rule.cached, true);
    assert.equal(recovered.rule.contentKey, first.rule.contentKey);
    assert.deepEqual(recovered.diagnostics, first.diagnostics);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("canonical provider result identity rejects a legacy digest with unchanged fact bytes", () => {
  const baseline = JSON.parse(
    readFileSync(
      path.join(root, "crates/vize_vitrine/tests/fixtures/hash-migration-v1.json"),
      "utf8",
    ),
  );
  assert.equal(baseline.state, "captured-old-source");
  assert.match(baseline.providerResult, /^s0\.v1:[a-f0-9]{32}$/);
  const dir = mkdtempSync(path.join(tmpdir(), "vize-fact-provider-hash-migration-"));
  try {
    const first = run(dir, "red");
    assert.match(first.provider.contentKey, /^l0\.v2:[a-f0-9]{32}$/);
    assert.match(first.rule.contentKey, /^l0\.v2:[a-f0-9]{32}$/);
    const file = readdirSync(dir).find((name) => name.startsWith("plugin-facts-"));
    assert.ok(file);
    const at = path.join(dir, file);
    const entry = JSON.parse(readFileSync(at, "utf8"));
    assert.deepEqual(entry.values, baseline.providerValues);
    assert.notEqual(entry.result_key, baseline.providerResult);
    const currentResult = entry.result_key;
    entry.result_key = baseline.providerResult;
    writeFileSync(at, JSON.stringify(entry));
    const recovered = run(dir, "red");
    assert.equal(recovered.calls, 2);
    assert.equal(recovered.provider.cached, false);
    assert.equal(recovered.rule.cached, true);
    assert.equal(recovered.provider.contentKey, first.provider.contentKey);
    assert.equal(recovered.rule.contentKey, first.rule.contentKey);
    assert.deepEqual(recovered.diagnostics, first.diagnostics);
    const rewritten = JSON.parse(readFileSync(at, "utf8"));
    assert.deepEqual(rewritten.values, entry.values);
    assert.equal(rewritten.result_key, currentResult);
    const freshProcess = run(dir, "red");
    assert.equal(freshProcess.calls, 0);
    assert.equal(freshProcess.provider.cached, true);
    assert.equal(freshProcess.rule.cached, true);
    assert.deepEqual(freshProcess.diagnostics, first.diagnostics);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
