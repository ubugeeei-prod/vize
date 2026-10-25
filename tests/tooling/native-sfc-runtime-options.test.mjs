import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  appendFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  readlinkSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { test } from "node:test";
import vm from "node:vm";

const proof = process.env.VIZE_NATIVE_SFC_RUNTIME_OPTIONS_PROOF
  ? resolve(process.env.VIZE_NATIVE_SFC_RUNTIME_OPTIONS_PROOF)
  : undefined;
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
let pinnedModule;
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
    pinnedModule = observe(
      "pinned-ssr",
      { filename: "Layout.vue", ssr: true, sourceMap: false, scopeId: "runtime-options" },
      () =>
        native.compileSfc(sourceFiles[4], {
          filename: "Layout.vue",
          ssr: true,
          sourceMap: false,
          scopeId: "runtime-options",
        }),
    );
    for (const [target, targetOptions] of Object.entries(targets)) {
      for (const inlineTemplate of [undefined, true]) {
        for (const sourceMap of [false, true]) {
          for (const runtimeModuleName of runtimeModules) {
            const options = {
              ...targetOptions,
              sourceMap,
              ...(inlineTemplate === undefined ? {} : { inlineTemplate }),
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
      }
      for (const runtimeModuleName of runtimeModules) {
        for (const runtimeGlobalName of [undefined, "Vue", "AlternateVue", "RuntimeHolder.雪Vue"]) {
          const options = {
            ...targetOptions,
            mode: "function",
            sourceMap: true,
            filename: files[1].path,
            scopeId: files[1].scopeId,
            ...(runtimeModuleName === undefined ? {} : { runtimeModuleName }),
            ...(runtimeGlobalName === undefined ? {} : { runtimeGlobalName }),
          };
          const result = observe("function", { target, ...options }, () =>
            native.compileSfc(sourceFiles[1], options),
          );
          const functionBatch = observe("function-files", { target, ...options }, () =>
            native.compileSfcBatchWithResults([files[1]], {
              ...options,
              includeSourceMap: true,
              includeHashes: true,
              includeStyles: true,
              includeCustomBlocks: true,
              includeMacroArtifacts: true,
            }),
          );
          const functionStats = observe("function-stats", { target, ...options }, () =>
            native.compileSfcBatch("./Case-1.vue", options),
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

function completeCases() {
  assert.equal(cases.length, 144, "all 72 module and 72 standalone configurations were captured");
  assert.equal(observations.length, 901, "every native API return was retained");
  return cases;
}

await test(
  "default and explicit default runtime names preserve entire native returns",
  skip,
  () => {
    for (const target of Object.keys(targets)) {
      for (const inlineTemplate of [undefined, true]) {
        for (const sourceMap of [false, true]) {
          const baseline = completeCases().find(
            (row) =>
              row.target === target &&
              row.options.sourceMap === sourceMap &&
              row.options.inlineTemplate === inlineTemplate &&
              row.batch &&
              row.options.runtimeModuleName === undefined,
          );
          const explicit = completeCases().find(
            (row) =>
              row.target === target &&
              row.options.sourceMap === sourceMap &&
              row.options.inlineTemplate === inlineTemplate &&
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
  for (const row of completeCases().filter((row) => row.batch)) {
    assert.equal(row.batch.failedCount, 0);
    assert.equal(row.batch.successCount, sourceFiles.length);
    for (const [index, result] of row.single.entries()) {
      assert.deepEqual(
        Object.fromEntries(keys.map((key) => [key, result[key]])),
        Object.fromEntries(keys.map((key) => [key, row.batch.results[index][key]])),
      );
      if (row.options.sourceMap) {
        if (index === 0) {
          // Existing template-only static output has no authored map link. The
          // before-source comparison independently pins this absent-map DTO.
          assert.equal(result.map, undefined);
        } else assert.deepEqual(JSON.parse(result.map).sourcesContent, [sourceFiles[index]]);
      }
    }
    if (row.stats) {
      assert.equal(row.stats.failed, 0);
      assert.equal(row.stats.success, sourceFiles.length);
      const baseline = completeCases().find(
        (entry) =>
          entry.batch &&
          entry.target === row.target &&
          entry.options.sourceMap === false &&
          entry.options.inlineTemplate === row.options.inlineTemplate &&
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
    for (const row of completeCases().filter((row) => row.batch)) {
      const selectedModule = row.options.runtimeModuleName ?? "vue";
      for (const [index, syntax] of row.syntax.entries()) {
        assert.equal(
          syntax.valid,
          true,
          JSON.stringify({ target: row.target, options: row.options, syntax }),
        );
        const authored = { 2: "author-runtime", 3: "vue", 7: "#app/nuxt" }[index];
        if (authored)
          assert.deepEqual(
            syntax.imports.filter((name) => name === authored),
            [authored],
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
  const runtimeRoot = process.env.VIZE_NATIVE_SFC_RUNTIME_OPTIONS_VUE;
  const Vue = require(join(runtimeRoot, "node_modules/vue"));
  assert.equal(Vue.version, "3.5.44");
  const inventory = (directory, relative = "") =>
    readdirSync(directory, { withFileTypes: true })
      .sort((left, right) => left.name.localeCompare(right.name))
      .flatMap((entry) => {
        const absolute = join(directory, entry.name);
        const path = join(relative, entry.name);
        if (entry.isDirectory()) return inventory(absolute, path);
        if (entry.isSymbolicLink()) return [{ path, target: readlinkSync(absolute) }];
        assert.equal(entry.isFile(), true);
        const bytes = readFileSync(absolute);
        return [{ path, bytes: bytes.length, sha256: hash(bytes) }];
      });
  writeFileSync(
    join(proof, "actual-runtime-provider.json"),
    JSON.stringify(
      {
        root: runtimeRoot,
        version: Vue.version,
        node: process.version,
        lock: JSON.parse(readFileSync(join(runtimeRoot, "package-lock.json"))),
        files: inventory(join(runtimeRoot, "node_modules")),
      },
      null,
      2,
    ) + "\n",
  );
  for (const row of completeCases().filter((row) => row.function && row.target === "dom")) {
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
    const state = component.data();
    const rendered = component.render(state, []);
    assert.equal(rendered.type, "button");
    assert.equal(rendered.children, "1");
    rendered.props.onClick({});
    const updated = component.render(state, []);
    assert.equal(updated.children, "2");
    assert.deepEqual(reads, [chosen]);
    appendFileSync(
      join(proof, "runtime-globals.jsonl"),
      JSON.stringify({
        options: row.options,
        reads,
        type: rendered.type,
        children: rendered.children,
        updatedChildren: updated.children,
      }) + "\n",
    );
  }
});

await test("standalone single, files and statistics preserve code and maps", skip, () => {
  for (const row of completeCases().filter((entry) => entry.function)) {
    const { path, scopeId, ...file } = row.functionBatch.results[0];
    assert.equal(path, row.options.filename);
    assert.equal(scopeId, row.options.scopeId);
    assert.deepEqual(file, row.function);
    assert.deepEqual(JSON.parse(file.map).sourcesContent, [sourceFiles[1]]);
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

await test("whole default code, maps and DTOs equal the before-source native", skip, () => {
  assert.ok(process.env.VIZE_NATIVE_SFC_RUNTIME_OPTIONS_BASELINE);
  const before = JSON.parse(readFileSync(process.env.VIZE_NATIVE_SFC_RUNTIME_OPTIONS_BASELINE));
  assert.equal(before.length, 144);
  const defaults = (rows) =>
    rows.filter(
      (row) =>
        row.options.runtimeModuleName === undefined && row.options.runtimeGlobalName === undefined,
    );
  const withoutTiming = (row) => {
    if (!row) return row;
    const { timeMs: _timeMs, ...rest } = row;
    return rest;
  };
  const comparable = (row) => ({
    target: row.target,
    options: row.options,
    single: row.single,
    batch: withoutTiming(row.batch),
    stats: withoutTiming(row.stats),
    function: row.function,
    functionBatch: withoutTiming(row.functionBatch),
    functionStats: withoutTiming(row.functionStats),
    syntax: row.syntax,
  });
  assert.deepEqual(defaults(completeCases()).map(comparable), defaults(before).map(comparable));
});

await test("pinned default SSR module remains byte-identical", skip, () => {
  completeCases();
  const fixture = new URL(
    "../_fixtures/differential/compiler/sfc-runtime-options-native/",
    import.meta.url,
  );
  assert.equal(readFileSync(new URL("Layout.vue.txt", fixture), "utf8"), sourceFiles[4]);
  assert.equal(pinnedModule.code, readFileSync(new URL("module.expected.txt", fixture), "utf8"));
  assert.deepEqual(pinnedModule.errors, []);
  assert.deepEqual(pinnedModule.warnings, []);
});
