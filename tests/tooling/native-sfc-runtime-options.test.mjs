import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  appendFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import vm from "node:vm";

const proof = process.env.VIZE_NATIVE_SFC_RUNTIME_OPTIONS_PROOF;
if (process.env.VIZE_NATIVE_SFC_RUNTIME_OPTIONS_REQUIRED === "1") assert.ok(proof);
const selected = Boolean(proof);
const require = createRequire(import.meta.url);
let native;
const observations = [];
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const sourceFiles = [
  '<template><div id="雪">Hello 😀</div></template>',
  '<script>export default { data() { return { n: 1 } } }</script><template><button @click="n++">{{ n }}</button></template>',
  '<script setup lang="ts">import { ref } from "author-runtime";const n=ref<number>(1)</script><template><button @click="n++">{{ n }}</button></template>',
];
sourceFiles.push(
  '<script setup>import { ref as authoredRef } from "vue"; const n=authoredRef(1)</script><template><button @click="n++">{{ n }}</button></template>',
  "<script setup>const n=await Promise.resolve(1)</script><template><div>{{ n }}</div></template>",
  '<script setup>const { title = "Hi", ...rest } = defineProps(["title", "other"]); const model=defineModel(); const slots=defineSlots()</script><template><div>{{ title }}{{ rest.other }}{{ model }}</div></template>',
  '<script setup>const color="red"</script><template><div/></template><style module>.x { color: v-bind(color) }</style>',
  '<script setup>const Lazy=defineLazyHydrationComponent("visible", () => import("./Lazy.vue"))</script><template><Lazy/></template>',
);
const targets = { dom: {}, vapor: { vapor: true }, ssr: { ssr: true } };
const runtimeModules = [
  undefined,
  "vue",
  "@runtime/雪",
  "@runtime/quote'雪",
  '@runtime/quote"雪',
  "@runtime/back\\雪",
];
function observe(name, configuration, operation) {
  const start = performance.now();
  let result;
  try {
    result = operation();
  } catch (error) {
    const row = {
      name,
      configuration,
      durationMs: performance.now() - start,
      error: Object.fromEntries(Object.getOwnPropertyNames(error).map((key) => [key, error[key]])),
    };
    observations.push(row);
    appendFileSync(join(proof, "whole-observations.jsonl"), JSON.stringify(row) + "\n");
    throw error;
  }
  const row = { name, configuration, durationMs: performance.now() - start, result };
  observations.push(row);
  appendFileSync(join(proof, "whole-observations.jsonl"), JSON.stringify(row) + "\n");
  return result;
}
const skip = { skip: !selected };
if (selected) {
  mkdirSync(proof, { recursive: true });
  assert.equal(existsSync(join(proof, "whole-observations.jsonl")), false);
  native = require("../../npm/native/index.js");
  const identity = require("../../npm/native/native-binding-identity.js").getBindingIdentity(
    native,
  );
  assert.equal(identity.kind, "native");
  const source = JSON.parse(
    readFileSync(
      new URL("../../npm/native/.artifacts/native/build-token-receipt.json", import.meta.url),
    ),
  );
  assert.equal(source.source, process.env.SOURCE_SHA);
  assert.equal(source.identity.token, native.getNativeBuildToken());
  assert.equal(
    hash(readFileSync(new URL(`../../npm/native/${source.binary.filename}`, import.meta.url))),
    source.binary.sha256,
  );
  writeFileSync(join(proof, "source.json"), JSON.stringify(source, null, 2) + "\n");
  assert.equal(typeof vm.SourceTextModule, "function", "use the real V8 module parser");
}
const cases = [];
await test("capture all native runtime configurations before semantic assertions", skip, () => {
  const fixtures = mkdtempSync(join(tmpdir(), "vize-runtime-options-"));
  const previous = process.cwd();
  try {
    process.chdir(fixtures);
    const files = sourceFiles.map((source, index) => {
      const path = `./Case-${index}.vue`;
      writeFileSync(path, source);
      return { path, source, scopeId: "runtime-options" };
    });
    for (const [target, targetOptions] of Object.entries(targets)) {
      for (const sourceMap of [false, true]) {
        for (const runtimeModuleName of runtimeModules) {
          const options = {
            ...targetOptions,
            sourceMap,
            ...(runtimeModuleName === undefined ? {} : { runtimeModuleName }),
          };
          const batchOptions = {
            ...options,
            includeSourceMap: sourceMap,
            includeHashes: true,
            includeStyles: true,
            includeCustomBlocks: true,
            includeMacroArtifacts: true,
          };
          const batch = observe("files", { target, ...batchOptions }, () =>
            native.compileSfcBatchWithResults(files, batchOptions),
          );
          const single = files.map((file) =>
            observe(
              "single",
              { target, ...options, filename: file.path, scopeId: file.scopeId },
              () =>
                native.compileSfc(file.source, {
                  ...options,
                  filename: file.path,
                  scopeId: file.scopeId,
                }),
            ),
          );
          const syntax = batch.results.map((result) => {
            try {
              const parsed = new vm.SourceTextModule(result.code);
              return { valid: true, imports: parsed.dependencySpecifiers };
            } catch (error) {
              return {
                valid: false,
                error: { name: error.name, message: error.message, stack: error.stack },
              };
            }
          });
          cases.push({ target, options, batch, single, syntax });
          if (!sourceMap) {
            const stats = observe("stats", { target, ...options }, () =>
              native.compileSfcBatch("./*.vue", options),
            );
            cases.at(-1).stats = stats;
          }
        }
      }
      for (const runtimeModuleName of runtimeModules) {
        for (const runtimeGlobalName of [undefined, "Vue", "AlternateVue", "RuntimeHolder.雪Vue"]) {
          const options = {
            ...targetOptions,
            mode: "function",
            sourceMap: true,
            filename: files[0].path,
            scopeId: files[0].scopeId,
            ...(runtimeModuleName === undefined ? {} : { runtimeModuleName }),
            ...(runtimeGlobalName === undefined ? {} : { runtimeGlobalName }),
          };
          const result = observe("function", { target, ...options }, () =>
            native.compileSfc(sourceFiles[0], options),
          );
          const functionBatch = observe("function-files", { target, ...options }, () =>
            native.compileSfcBatchWithResults([files[0]], {
              ...options,
              includeSourceMap: true,
              includeHashes: true,
              includeStyles: true,
              includeCustomBlocks: true,
              includeMacroArtifacts: true,
            }),
          );
          const functionStats = observe("function-stats", { target, ...options }, () =>
            native.compileSfcBatch("./Case-0.vue", options),
          );
          cases.push({ target, options, function: result, functionBatch, functionStats });
        }
      }
    }
  } finally {
    process.chdir(previous);
  }
  writeFileSync(join(proof, "matrix.json"), JSON.stringify(cases, null, 2) + "\n");
});

await test(
  "default and explicit default runtime names preserve entire native returns",
  skip,
  () => {
    for (const target of Object.keys(targets)) {
      for (const sourceMap of [false, true]) {
        const baseline = cases.find(
          (row) =>
            row.target === target &&
            row.options.sourceMap === sourceMap &&
            row.batch &&
            row.options.runtimeModuleName === undefined,
        );
        const explicit = cases.find(
          (row) =>
            row.target === target &&
            row.options.sourceMap === sourceMap &&
            row.batch &&
            row.options.runtimeModuleName === "vue",
        );
        assert.deepEqual(explicit.single, baseline.single);
        const { timeMs: beforeTime, ...before } = baseline.batch;
        const { timeMs: afterTime, ...after } = explicit.batch;
        assert.equal(typeof beforeTime, "number");
        assert.equal(typeof afterTime, "number");
        assert.deepEqual(after, before);
      }
    }
  },
);

await test("single and batch code, maps and all shared metadata remain equal", skip, () => {
  const keys = [
    "code",
    "map",
    "css",
    "errors",
    "warnings",
    "templateHash",
    "styleHash",
    "scriptHash",
    "styles",
    "customBlocks",
    "macroArtifacts",
    "moduleShape",
    "hasScoped",
  ];
  for (const row of cases.filter((row) => row.batch)) {
    assert.equal(row.batch.failedCount, 0);
    assert.equal(row.batch.successCount, sourceFiles.length);
    for (const [index, result] of row.single.entries()) {
      assert.deepEqual(
        Object.fromEntries(keys.map((key) => [key, result[key]])),
        Object.fromEntries(keys.map((key) => [key, row.batch.results[index][key]])),
      );
      if (row.options.sourceMap)
        assert.deepEqual(JSON.parse(result.map).sourcesContent, [sourceFiles[index]]);
    }
    if (row.stats) {
      assert.equal(row.stats.failed, 0);
      assert.equal(row.stats.success, sourceFiles.length);
      const baseline = cases.find(
        (entry) =>
          entry.batch &&
          entry.target === row.target &&
          entry.options.sourceMap === false &&
          entry.options.runtimeModuleName === undefined,
      );
      const bytes = (entry) =>
        entry.batch.results.reduce((sum, result) => sum + Buffer.byteLength(result.code), 0);
      // The statistics API owns its scope IDs. Runtime-option deltas must agree
      // while keeping both APIs' independently owned whole outputs in evidence.
      assert.equal(
        row.stats.outputBytes - baseline.stats.outputBytes,
        bytes(row) - bytes(baseline),
      );
    }
  }
});

await test(
  "custom runtime names produce valid whole modules and only selected generated imports",
  skip,
  () => {
    for (const row of cases.filter((row) => row.batch)) {
      const selectedModule = row.options.runtimeModuleName ?? "vue";
      for (const [index, syntax] of row.syntax.entries()) {
        assert.equal(
          syntax.valid,
          true,
          JSON.stringify({ target: row.target, options: row.options, syntax }),
        );
        assert.equal(
          syntax.imports.every(
            (name) =>
              name === selectedModule ||
              name === "@vue/server-renderer" ||
              name === "author-runtime" ||
              (index === 3 && name === "vue") ||
              (index === 7 && name === "#app/nuxt"),
          ),
          true,
          JSON.stringify({ target: row.target, options: row.options, imports: syntax.imports }),
        );
      }
    }
  },
);

await test("standalone DOM output reads the selected real runtime global", skip, () => {
  const Vue = require(join(process.env.VIZE_NATIVE_SFC_RUNTIME_OPTIONS_VUE, "node_modules/vue"));
  assert.equal(Vue.version, "3.5.44");
  for (const row of cases.filter((row) => row.function && row.target === "dom")) {
    const reads = [];
    const context = {};
    const chosen = row.options.runtimeGlobalName ?? "Vue";
    if (chosen === "RuntimeHolder.雪Vue") {
      context.RuntimeHolder = Object.defineProperty({}, "雪Vue", {
        get() {
          reads.push(chosen);
          return Vue;
        },
      });
    } else
      Object.defineProperty(context, chosen, {
        get() {
          reads.push(chosen);
          return Vue;
        },
      });
    const component = vm.runInNewContext(`(function(){${row.function.code}\n})()`, context);
    assert.equal(typeof component.render, "function");
    const rendered = component.render({}, []);
    assert.equal(rendered.type, "div");
    assert.equal(rendered.children, "Hello 😀");
    assert.deepEqual(reads, [chosen]);
    appendFileSync(
      join(proof, "runtime-globals.jsonl"),
      JSON.stringify({
        options: row.options,
        reads,
        type: rendered.type,
        children: rendered.children,
      }) + "\n",
    );
  }
});

await test("standalone single, files and statistics preserve code and maps", skip, () => {
  for (const row of cases.filter((entry) => entry.function)) {
    const { path, scopeId, ...file } = row.functionBatch.results[0];
    assert.equal(path, row.options.filename);
    assert.equal(scopeId, row.options.scopeId);
    assert.deepEqual(file, row.function);
    assert.deepEqual(JSON.parse(file.map).sourcesContent, [sourceFiles[0]]);
    assert.equal(row.functionStats.failed, 0);
    assert.equal(row.functionStats.success, 1);
    assert.equal(row.functionStats.outputBytes, Buffer.byteLength(file.code));
    if (row.target !== "ssr") {
      new vm.Script(`(function(){${file.code}\n})`);
      assert.deepEqual(file.warnings, []);
    } else {
      assert.deepEqual(file.warnings, [
        "Standalone SFC output still contains non-Vue ES module imports; CDN evaluation requires those dependencies to be provided separately.",
      ]);
    }
  }
});
