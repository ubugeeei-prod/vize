import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

const proof = process.env.VIZE_NATIVE_SFC_VERSION_PROOF;
const required = process.env.VIZE_NATIVE_SFC_VERSION_REQUIRED === "1";
const selected = Boolean(proof) || required;
const require = createRequire(import.meta.url);
const rows = [];
let native;
let fixtures;
if (selected) {
  assert.ok(proof, "the required source-native campaign must retain full observations");
  mkdirSync(proof, { recursive: true });
  fixtures = mkdtempSync(join(tmpdir(), "vize-sfc-runtime-雪-"));
  native = require("../../npm/native/index.js");
  const { getBindingIdentity } = require("../../npm/native/native-binding-identity.js");
  const identity = getBindingIdentity(native);
  assert.equal(identity.kind, "native");
  assert.equal(native.getNativeBuildToken(), identity.token);
  const receipt = JSON.parse(
    readFileSync(
      new URL("../../npm/native/.artifacts/native/build-token-receipt.json", import.meta.url),
    ),
  );
  assert.equal(receipt.source, process.env.SOURCE_SHA);
  assert.equal(receipt.identity.token, identity.token);
  const binary = new URL(`../../npm/native/${receipt.binary.filename}`, import.meta.url);
  assert.equal(
    createHash("sha256").update(readFileSync(binary)).digest("hex"),
    receipt.binary.sha256,
  );
  writeFileSync(join(proof, "source.json"), JSON.stringify(receipt, null, 2) + "\n");
}
function persist() {
  if (selected)
    writeFileSync(join(proof, "whole-observations.json"), JSON.stringify(rows, null, 2) + "\n");
}
process.once("exit", persist);
const observe = (name, options, operation) => {
  const started = performance.now();
  let result;
  try {
    result = operation();
  } catch (error) {
    rows.push({
      name,
      options,
      durationMs: performance.now() - started,
      error: {
        name: error.name,
        ...Object.fromEntries(Object.getOwnPropertyNames(error).map((key) => [key, error[key]])),
      },
    });
    persist();
    throw error;
  }
  rows.push({ name, options, durationMs: performance.now() - started, result });
  persist();
  return result;
};
const options = { skip: !selected };
const legacy = ["2", "2.7", "v2", " V2.7 ", "1", "0.11", "0.10"];
const invalid = ["", "2.6", "2.6.14", "2.7.16", "3.5", "4", "0", "Vue2", "2.7-beta"];
const source =
  '<template><div id="雪">Hello {{ count }}</div></template><script>export default { data() { return { count: 1 } } }</script>';

for (const [name, invoke] of Object.entries({
  single: (configuration) => native.compileSfc(source, configuration),
  files: (configuration) =>
    native.compileSfcBatchWithResults([{ path: "雪.vue", source }], configuration),
  stats: (configuration) => native.compileSfcBatch(join(fixtures, "*.vue"), configuration),
})) {
  await test(`${name} rejects an explicit unavailable runtime across target modes`, options, () => {
    for (const version of legacy) {
      for (const target of [{}, { ssr: true }, { vapor: true }]) {
        const configuration = { vueVersion: version, ...target };
        assert.throws(
          () => observe(name, configuration, () => invoke(configuration)),
          (error) => {
            assert.equal(error.code, "InvalidArg");
            assert.match(error.message, /native SFC compiler currently targets the Vue 3 runtime/);
            assert.match(error.message, /framework compiler enabled/);
            return true;
          },
        );
      }
    }
  });
  await test(`${name} rejects invalid or ambiguous selectors`, options, () => {
    for (const version of invalid) {
      const configuration = { vueVersion: version };
      assert.throws(
        () => observe(name, configuration, () => invoke(configuration)),
        (error) => {
          assert.equal(error.code, "InvalidArg");
          assert.match(error.message, /vue\.version/);
          return true;
        },
      );
    }
  });
}

await test("runtime validation precedes worker allocation and glob reads", options, () => {
  for (const operation of [
    () => native.compileSfcBatch("[", { vueVersion: "2.7", threads: 0 }),
    () => native.compileSfcBatchWithResults([], { vueVersion: "2.7", threads: 0 }),
  ]) {
    assert.throws(
      () => observe("preflight", { vueVersion: "2.7", threads: 0 }, operation),
      /native SFC compiler currently targets the Vue 3 runtime/,
    );
  }
  assert.throws(
    () =>
      observe("modern-worker-control", { vueVersion: "3", threads: 0 }, () =>
        native.compileSfcBatch("[", { vueVersion: "3", threads: 0 }),
      ),
    /thread/i,
  );
});

await test("default and explicit Vue3 preserve complete single and batch modules", options, () => {
  const sources = [
    source,
    '<template><p class="雪">😀 &amp; {{ 1 + 2 }}</p></template>',
    '<template><button @click="count++">{{ count }}</button></template><script setup>let count = 1</script>',
    '<template><div :title="label">{{ label }}</div></template><script setup lang="ts">const label: string = "雪"</script><style scoped>div { color: red }</style>',
  ];
  const files = sources.map((source, index) => {
    const path = join(fixtures, `entry-${index}.vue`);
    writeFileSync(path, source);
    return { path, source };
  });
  for (const target of [{}, { ssr: true }, { vapor: true }]) {
    const baseline = observe("default-files", target, () =>
      native.compileSfcBatchWithResults(files, target),
    );
    assert.equal(baseline.successCount, sources.length);
    assert.equal(baseline.failedCount, 0);
    for (const version of ["3", "v3", " V3 "]) {
      const configuration = { ...target, vueVersion: version };
      const explicit = observe("explicit-files", configuration, () =>
        native.compileSfcBatchWithResults(files, configuration),
      );
      const { timeMs: beforeTime, ...before } = baseline;
      const { timeMs: afterTime, ...after } = explicit;
      assert.equal(typeof beforeTime, "number");
      assert.equal(typeof afterTime, "number");
      assert.deepEqual(after, before);
      for (const [index, file] of files.entries()) {
        const ordinary = observe("default-single", { target, index }, () =>
          native.compileSfc(file.source, { ...target, filename: file.path }),
        );
        const selected = observe("explicit-single", { configuration, index }, () =>
          native.compileSfc(file.source, { ...configuration, filename: file.path }),
        );
        assert.deepEqual(selected, ordinary);
        assert.deepEqual(selected.errors, []);
        assert.ok(selected.code.length > 0);
        const checked = execFileSync(process.execPath, ["--input-type=module", "--check"], {
          input: selected.code,
        });
        assert.equal(checked.length, 0);
      }
    }
    const stats = observe("default-stats", target, () =>
      native.compileSfcBatch(join(fixtures, "*.vue"), target),
    );
    const explicitStats = observe("explicit-stats", { ...target, vueVersion: "3" }, () =>
      native.compileSfcBatch(join(fixtures, "*.vue"), { ...target, vueVersion: "3" }),
    );
    const { timeMs: beforeTime, ...before } = stats;
    const { timeMs: afterTime, ...after } = explicitStats;
    assert.equal(before.failed, 0);
    assert.equal(before.success, files.length);
    assert.equal(typeof beforeTime, "number");
    assert.equal(typeof afterTime, "number");
    assert.deepEqual(after, before);
  }
});

await test("explicit Vue3 emits its actual DOM and server runtime vocabulary", options, () => {
  const dom = observe("dom-vocabulary", { vueVersion: "3" }, () =>
    native.compileSfc(source, { vueVersion: "3" }),
  );
  assert.deepEqual(dom.errors, []);
  assert.match(dom.code, /from ["']vue["']/);
  assert.match(dom.code, /createElementBlock|createElementVNode/);
  const ssr = observe("ssr-vocabulary", { vueVersion: "3", ssr: true }, () =>
    native.compileSfc(source, { vueVersion: "3", ssr: true }),
  );
  assert.deepEqual(ssr.errors, []);
  assert.match(ssr.code, /from ["']@vue\/server-renderer["']/);
  assert.match(ssr.code, /ssrRender|ssrInterpolate/);
  assert.equal(readdirSync(fixtures).filter((name) => name.endsWith(".vue")).length, 4);
});

persist();
