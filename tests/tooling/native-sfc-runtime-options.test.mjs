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
import { closedSetupReference } from "./support/script-setup-default-expose.ts";

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
const authoredStandalone = [];
let pinnedModule;
let parcelPacket;
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
    const authoredSource = String.raw`<script>import { ref as authoredRef } from '@runtime/quote\'雪';
export default { data() { return { n: 1 } } }</script><template><button @click="n++">{{ n }}</button></template>`;
    const authoredFile = {
      path: "./Authored.vue",
      source: authoredSource,
      scopeId: "runtime-options",
    };
    writeFileSync(authoredFile.path, authoredSource);
    for (const [target, targetOptions] of Object.entries(targets)) {
      for (const runtimeModuleName of [undefined, "@runtime/quote'雪"]) {
        const options = {
          ...targetOptions,
          filename: authoredFile.path,
          scopeId: authoredFile.scopeId,
          mode: "function",
          sourceMap: true,
          runtimeGlobalName: "SelectedVue",
          ...(runtimeModuleName === undefined ? {} : { runtimeModuleName }),
        };
        const single = observe("authored-standalone-single", { target, ...options }, () =>
          native.compileSfc(authoredSource, options),
        );
        const batch = observe("authored-standalone-files", { target, ...options }, () =>
          native.compileSfcBatchWithResults([authoredFile], {
            ...options,
            includeSourceMap: true,
            includeHashes: true,
            includeStyles: true,
            includeCustomBlocks: true,
            includeMacroArtifacts: true,
          }),
        );
        const stats = observe("authored-standalone-stats", { target, ...options }, () =>
          native.compileSfcBatch(authoredFile.path, options),
        );
        authoredStandalone.push({ target, options, source: authoredSource, single, batch, stats });
      }
    }
    // Preserve the discovered adjacent-statement input: a line replacement
    // cannot own its component declaration, so the existing external-import
    // contract must keep the complete statement and report the remaining import.
    for (const [control, source, runtimeModuleName, authoredModule] of [
      [
        "same-line-quoted",
        authoredSource.replace(";\nexport", "; export"),
        "@runtime/quote'雪",
        "@runtime/quote'雪",
      ],
      [
        "same-line-unicode",
        String.raw`<script>import { ref as authoredRef } from 'v\u0075e'; export default { data() { return { n: 1 } } }</script><template><button @click="n++">{{ n }}</button></template>`,
        undefined,
        "vue",
      ],
      [
        "decoded-source-near-miss",
        String.raw`<script>import { ref as authoredRef } from 'pkg\n';
export default { data() { return { n: 1 } } }</script><template><button @click="n++">{{ n }}</button></template>`,
        String.raw`pkg\n`,
        "pkg\n",
      ],
      [
        "decoded-source-literal-backslash",
        String.raw`<script>import { ref as authoredRef } from 'pkg\\n';
export default { data() { return { n: 1 } } }</script><template><button @click="n++">{{ n }}</button></template>`,
        String.raw`pkg\n`,
        String.raw`pkg\n`,
      ],
    ]) {
      const file = { path: `./${control}.vue`, source, scopeId: "runtime-options" };
      writeFileSync(file.path, source);
      const options = {
        filename: file.path,
        scopeId: file.scopeId,
        mode: "function",
        sourceMap: true,
        ...(runtimeModuleName === undefined ? {} : { runtimeModuleName }),
      };
      const single = observe(control, options, () => native.compileSfc(source, options));
      const batch = observe(`${control}-files`, options, () =>
        native.compileSfcBatchWithResults([file], {
          ...options,
          includeSourceMap: true,
          includeHashes: true,
          includeStyles: true,
          includeCustomBlocks: true,
          includeMacroArtifacts: true,
        }),
      );
      const stats = observe(`${control}-stats`, options, () =>
        native.compileSfcBatch(file.path, options),
      );
      authoredStandalone.push({
        target: "dom",
        control,
        external: control !== "decoded-source-literal-backslash",
        authoredModule,
        options,
        source,
        single,
        batch,
        stats,
      });
    }
    const source =
      '<script setup lang="ts">import {ref} from "vue-vapor";const n=ref<number>(1)</script><template><button @click="n++">Vapor: {{ n }}</button></template>';
    const options = {
      filename: "App.vue",
      vapor: true,
      runtimeModuleName: "vue-vapor",
      sourceMap: true,
    };
    const file = { path: "App.vue", source };
    writeFileSync(file.path, source);
    const single = observe("parcel-reproduction-single", options, () =>
      native.compileSfc(source, options),
    );
    const batch = observe("parcel-reproduction-files", options, () =>
      native.compileSfcBatchWithResults([file], {
        ...options,
        includeSourceMap: true,
        includeHashes: true,
        includeStyles: true,
        includeCustomBlocks: true,
        includeMacroArtifacts: true,
      }),
    );
    const stats = observe("parcel-reproduction-stats", options, () =>
      native.compileSfcBatch("./App.vue", options),
    );
    parcelPacket = { source, options, single, batch, stats };
  } finally {
    process.chdir(previous);
  }
  writeFileSync(join(proof, "matrix.json"), JSON.stringify(cases, null, 2) + "\n");
  writeFileSync(
    join(proof, "authored-standalone.json"),
    JSON.stringify(authoredStandalone, null, 2) + "\n",
  );
  writeFileSync(
    join(proof, "parcel-runtime-module.json"),
    JSON.stringify(parcelPacket, null, 2) + "\n",
  );
});

function completeCases() {
  assert.equal(cases.length, 144, "all 72 module and 72 standalone configurations were captured");
  assert.equal(observations.length, 934, "every native API return was retained");
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
    const rendered = component.render(state, [], {}, {}, state, component);
    assert.equal(rendered.type, "button");
    assert.equal(rendered.children, "1");
    rendered.props.onClick({});
    const updated = component.render(state, [], {}, {}, state, component);
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

await test(
  "whole default packets equal the before-source native plus closed setup exposure",
  skip,
  () => {
    assert.ok(process.env.VIZE_NATIVE_SFC_RUNTIME_OPTIONS_BASELINE);
    const before = JSON.parse(readFileSync(process.env.VIZE_NATIVE_SFC_RUNTIME_OPTIONS_BASELINE));
    assert.equal(before.length, 144);
    const defaults = (rows) =>
      rows.filter(
        (row) =>
          row.options.runtimeModuleName === undefined &&
          row.options.runtimeGlobalName === undefined,
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
    const advance = (row) => {
      const out = structuredClone(row);
      if (out.single)
        out.single = out.single.map((packet, index) =>
          closedSetupReference(packet, sourceFiles[index]),
        );
      if (out.batch)
        out.batch.results = out.batch.results.map((packet, index) =>
          closedSetupReference(packet, sourceFiles[index]),
        );
      if (out.stats) {
        out.stats.outputBytes += out.single.reduce(
          (bytes, packet, index) =>
            bytes + Buffer.byteLength(packet.code) - Buffer.byteLength(row.single[index].code),
          0,
        );
      }
      return out;
    };
    assert.deepEqual(
      defaults(completeCases()).map(comparable),
      defaults(before).map(advance).map(comparable),
    );
  },
);

await test("pinned SSR module follows the closed setup exposure contract", skip, () => {
  completeCases();
  const fixture = new URL(
    "../_fixtures/differential/compiler/sfc-runtime-options-native/",
    import.meta.url,
  );
  assert.equal(readFileSync(new URL("Layout.vue.txt", fixture), "utf8"), sourceFiles[4]);
  const preserved = { code: readFileSync(new URL("module.expected.txt", fixture), "utf8") };
  assert.equal(pinnedModule.code, closedSetupReference(preserved, sourceFiles[4]).code);
  assert.deepEqual(pinnedModule.errors, []);
  assert.deepEqual(pinnedModule.warnings, []);
});

await test(
  "standalone recognizes escaped authored runtime sources and warns for external imports",
  skip,
  () => {
    completeCases();
    assert.equal(authoredStandalone.length, 10);
    const Vue = require(join(process.env.VIZE_NATIVE_SFC_RUNTIME_OPTIONS_VUE, "node_modules/vue"));
    for (const row of authoredStandalone) {
      const { path, scopeId, ...file } = row.batch.results[0];
      assert.equal(path, row.options.filename);
      assert.equal(scopeId, row.options.scopeId);
      assert.deepEqual(file, row.single);
      assert.deepEqual(JSON.parse(file.map).sourcesContent, [row.source]);
      assert.equal(row.stats.failed, 0);
      assert.equal(row.stats.success, 1);
      assert.equal(row.stats.outputBytes, Buffer.byteLength(file.code));
      const external =
        row.external ?? (row.target === "ssr" || row.options.runtimeModuleName === undefined);
      if (row.control) {
        const { parse } = require(
          join(process.env.VIZE_NATIVE_SFC_RUNTIME_OPTIONS_VUE, "node_modules/@babel/parser"),
        );
        const parsed = parse(file.code, { sourceType: "module", allowReturnOutsideFunction: true });
        assert.deepEqual(
          parsed.program.body
            .filter((node) => node.type === "ImportDeclaration")
            .map((node) => node.source.value),
          external ? [row.authoredModule] : [],
        );
        assert.equal(
          parsed.program.body.some(
            (node) =>
              node.type === "VariableDeclaration" &&
              node.declarations.some(
                (declaration) =>
                  declaration.id.type === "Identifier" && declaration.id.name === "_sfc_main",
              ),
          ),
          true,
        );
      }
      assert.deepEqual(
        file.warnings,
        external
          ? [
              "Standalone SFC output still contains non-Vue ES module imports; CDN evaluation requires those dependencies to be provided separately.",
            ]
          : [],
      );
      if (!external) new vm.Script(`(function(){${file.code}\n})`);
      if (!external && row.target === "dom") {
        const component = vm.runInNewContext(`(function(){${file.code}\n})()`, {
          [row.options.runtimeGlobalName ?? "Vue"]: Vue,
        });
        const state = component.data();
        const first = component.render(state, [], {}, {}, state, component);
        assert.equal(first.children, "1");
        first.props.onClick({});
        assert.equal(component.render(state, [], {}, {}, state, component).children, "2");
      }
    }
  },
);

await test(
  "the original custom Vapor runtime reproduction uses only its selected module",
  skip,
  () => {
    completeCases();
    const { path, scopeId: _scopeId, ...file } = parcelPacket.batch.results[0];
    assert.equal(path, parcelPacket.options.filename);
    assert.deepEqual(file, parcelPacket.single);
    assert.deepEqual(file.errors, []);
    assert.deepEqual(file.warnings, []);
    assert.deepEqual(new vm.SourceTextModule(file.code).dependencySpecifiers, ["vue-vapor"]);
    assert.deepEqual(JSON.parse(file.map).sourcesContent, [parcelPacket.source]);
    assert.equal(parcelPacket.stats.success, 1);
    assert.equal(parcelPacket.stats.failed, 0);
    assert.equal(parcelPacket.stats.outputBytes, Buffer.byteLength(file.code));
  },
);
