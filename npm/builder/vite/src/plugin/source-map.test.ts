/**
 * Source maps for compiled `.vue` SFCs (#3399).
 *
 * Before this, `compileSfc` ignored `sourceMap` and the plugin returned
 * `map: null` for every SFC module, so a production build with
 * `build.sourcemap` on resolved stack frames to the virtual `<file>.vue.ts`
 * module — a path that does not exist on disk — instead of the authored `.vue`.
 *
 * These tests decode the emitted mappings rather than merely asserting a map
 * exists: a map that resolves to the wrong place is the failure mode that
 * matters here.
 */

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { compileFile } from "../compiler.ts";
import { generateOutputWithMap } from "../utils/index.ts";
import { MappedModule, parseSourceMap, shiftMappedLines } from "../utils/source-map.ts";
import type { SourceMapV3 } from "../utils/source-map.ts";
import { toVirtualId } from "../virtual.ts";
import { loadHook } from "./load.ts";
import { getCompileOptionsForRequest, type VizePluginState } from "./state.ts";
import type { CompiledModule } from "../types.ts";

const BASE64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

interface DecodedSegment {
  generatedLine: number;
  generatedColumn: number;
  sourceIndex: number;
  sourceLine: number;
  sourceColumn: number;
}

/** Decode a v3 `mappings` string into absolute segments. */
function decodeMappings(mappings: string): DecodedSegment[] {
  const decoded: DecodedSegment[] = [];
  let sourceIndex = 0;
  let sourceLine = 0;
  let sourceColumn = 0;

  mappings.split(";").forEach((group, generatedLine) => {
    let generatedColumn = 0;
    for (const field of group.split(",")) {
      if (field === "") continue;
      let cursor = 0;
      const next = (): number => {
        let result = 0;
        let shift = 0;
        for (;;) {
          const digit = BASE64.indexOf(field[cursor++]);
          result |= (digit & 0b11111) << shift;
          shift += 5;
          if ((digit & 0b100000) === 0) break;
        }
        const magnitude = result >>> 1;
        return result & 1 ? -magnitude : magnitude;
      };
      generatedColumn += next();
      sourceIndex += next();
      sourceLine += next();
      sourceColumn += next();
      decoded.push({ generatedLine, generatedColumn, sourceIndex, sourceLine, sourceColumn });
    }
  });

  return decoded;
}

/** The 0-based generated line whose body is exactly `text` once trimmed. */
function generatedLineOf(code: string, text: string): number {
  const line = code.split("\n").findIndex((candidate) => candidate.trim() === text);
  assert.notEqual(line, -1, `emitted module should contain the line ${JSON.stringify(text)}`);
  return line;
}

function segmentForLine(map: SourceMapV3, code: string, text: string): DecodedSegment | undefined {
  const generatedLine = generatedLineOf(code, text);
  return decodeMappings(map.mappings).find((segment) => segment.generatedLine === generatedLine);
}

// ---------------------------------------------------------------------------
// shiftMappedLines / MappedModule
// ---------------------------------------------------------------------------

const threeMappedLines: SourceMapV3 = {
  version: 3,
  sources: ["/a.vue"],
  names: [],
  mappings: "AACA;AAEA;AAGA",
};

assert.equal(
  shiftMappedLines(threeMappedLines, 0, 2).mappings,
  ";;AACA;AAEA;AAGA",
  "prepending two lines pushes every mapped line down by two",
);
assert.equal(
  shiftMappedLines(threeMappedLines, 1, 1).mappings,
  "AACA;;AAEA;AAGA",
  "inserting a line mid-module moves only the lines after it",
);
assert.equal(
  shiftMappedLines(threeMappedLines, 3, 5).mappings,
  "AACA;AAEA;AAGA",
  "an insertion past the last mapped line changes nothing",
);
assert.equal(
  shiftMappedLines(threeMappedLines, 0, 0).mappings,
  "AACA;AAEA;AAGA",
  "a zero-line insertion is a no-op",
);

const appended = new MappedModule("a\nb\nc", { ...threeMappedLines });
appended.edit("a\nb\nc\ntail");
assert.equal(appended.map?.mappings, "AACA;AAEA;AAGA", "appending never moves existing mappings");

const prepended = new MappedModule("a\nb\nc", { ...threeMappedLines });
prepended.edit("x\ny\na\nb\nc");
assert.equal(prepended.map?.mappings, ";;AACA;AAEA;AAGA", "a prepend shifts by the lines it adds");

const sameLine = new MappedModule("export default {}\nb\nc", { ...threeMappedLines });
sameLine.edit("const _sfc_main = {}\nb\nc");
assert.equal(
  sameLine.map?.mappings,
  "AACA;AAEA;AAGA",
  "a same-line substitution leaves the line layout, and the mappings, alone",
);

const removedLines = new MappedModule("a\nb\nc", { ...threeMappedLines });
removedLines.edit("a\nc");
assert.equal(removedLines.map, null, "an edit that removes lines drops the map instead of lying");

assert.equal(parseSourceMap(undefined), null, "a missing map parses to null");
assert.equal(parseSourceMap("{"), null, "malformed JSON parses to null");
assert.equal(parseSourceMap('{"version":2}'), null, "a non-v3 document parses to null");

// ---------------------------------------------------------------------------
// compileFile: the napi boundary now carries the map
// ---------------------------------------------------------------------------

const SFC_SOURCE = `<template>
  <p @click="bump">{{ message }}</p>
</template>

<script setup>
import { ref } from 'vue'

const message = ref('from the sfc')
function bump() {
  message.value = 'bumped'
}
</script>
`;

const workspaceRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
const testRoot = path.join(
  workspaceRoot,
  "target",
  "vize-tests",
  "tests",
  "vite-plugin-vize",
  "map",
);
fs.mkdirSync(testRoot, { recursive: true });
const projectRoot = fs.mkdtempSync(path.join(testRoot, "sfc-"));
const sfcPath = path.join(projectRoot, "Counter.vue");
fs.writeFileSync(sfcPath, SFC_SOURCE);

const withoutMap = compileFile(sfcPath, new Map(), { sourceMap: false, ssr: false, vapor: false });
assert.equal(withoutMap.map ?? null, null, "sourceMap: false must not produce a map");

const compiled = compileFile(sfcPath, new Map(), { sourceMap: true, ssr: false, vapor: false });
const compiledMap = parseSourceMap(compiled.map);
assert.ok(compiledMap, "sourceMap: true must produce a parseable v3 map");
assert.equal(compiledMap.version, 3);
assert.equal(compiledMap.file, sfcPath);
assert.deepEqual(compiledMap.sources, [sfcPath], "sources must name the authored .vue file");
assert.deepEqual(compiledMap.sourcesContent, [SFC_SOURCE]);
assert.deepEqual(compiledMap.names, []);

// `const message = ref('from the sfc')` is authored line 7 (0-based), column 0.
assert.deepEqual(
  segmentForLine(compiledMap, compiled.code, "const message = ref('from the sfc')"),
  {
    generatedLine: generatedLineOf(compiled.code, "const message = ref('from the sfc')"),
    generatedColumn: 0,
    sourceIndex: 0,
    sourceLine: 7,
    sourceColumn: 0,
  },
  "the copied statement resolves to its authored line and column",
);

// ---------------------------------------------------------------------------
// generateOutputWithMap: the map survives the module rewrites
// ---------------------------------------------------------------------------

const styled: CompiledModule = { ...compiled, css: ".a{color:red}", hasScoped: true };
const generated = generateOutputWithMap(styled, {
  isProduction: false,
  isDev: false,
  filePath: sfcPath,
});
assert.ok(generated.map, "generateOutput must carry the map through its rewrites");
assert.deepEqual(generated.map.sources, [sfcPath]);
assert.deepEqual(
  segmentForLine(generated.map, generated.code, "const message = ref('from the sfc')"),
  {
    generatedLine: generatedLineOf(generated.code, "const message = ref('from the sfc')"),
    generatedColumn: 0,
    sourceIndex: 0,
    sourceLine: 7,
    sourceColumn: 0,
  },
  "prepending the inline <style> injection moves the mapping with the code",
);

// ---------------------------------------------------------------------------
// loadHook: a production build with build.sourcemap resolves to the .vue file
// ---------------------------------------------------------------------------

const productionState: VizePluginState = {
  cache: new Map(),
  ssrCache: new Map(),
  collectedCss: new Map(),
  precompileMetadata: new Map(),
  pendingHmrUpdateTypes: new Map(),
  isProduction: true,
  viteBuildSourcemap: true,
  root: projectRoot,
  clientViteBase: "/",
  serverViteBase: "/",
  server: null,
  filter: () => true,
  scanPatterns: ["**/*.vue"],
  precompileBatchSize: 128,
  ignorePatterns: [],
  mergedOptions: {},
  initialized: true,
  dynamicImportAliasRules: [],
  cssAliasRules: [],
  extractCss: false,
  componentsCssFileName: "assets/vize-components.css",
  clientViteDefine: {},
  serverViteDefine: {},
  logger: { log() {}, info() {}, warn() {}, error() {} } as never,
};

assert.equal(
  getCompileOptionsForRequest(productionState, false).sourceMap,
  true,
  "build.sourcemap must override the production-off default",
);
assert.equal(
  getCompileOptionsForRequest({ ...productionState, viteBuildSourcemap: false }, false).sourceMap,
  false,
  "a production build without build.sourcemap keeps maps off",
);
assert.equal(
  getCompileOptionsForRequest({ ...productionState, isProduction: false }, false).sourceMap,
  true,
  "development keeps maps on",
);
assert.equal(
  getCompileOptionsForRequest({ ...productionState, mergedOptions: { sourceMap: false } }, false)
    .sourceMap,
  false,
  "an explicit compiler option still wins over build.sourcemap",
);

const loaded = loadHook(productionState, toVirtualId(sfcPath), { ssr: false });
assert.ok(loaded && typeof loaded === "object", "the virtual SFC module should load");
const loadedMap = loaded.map;
assert.ok(loadedMap, "the plugin must stop returning map: null for SFC modules");
assert.deepEqual(
  loadedMap.sources,
  [sfcPath],
  "the loaded module's map must name the authored .vue file, not the virtual .vue.ts id",
);
assert.deepEqual(
  segmentForLine(loadedMap, loaded.code, "const message = ref('from the sfc')"),
  {
    generatedLine: generatedLineOf(loaded.code, "const message = ref('from the sfc')"),
    generatedColumn: 0,
    sourceIndex: 0,
    sourceLine: 7,
    sourceColumn: 0,
  },
  "a frame in the loaded module resolves to the authored .vue line",
);

fs.rmSync(projectRoot, { recursive: true, force: true });

console.log("vite-plugin-vize source map tests passed!");
