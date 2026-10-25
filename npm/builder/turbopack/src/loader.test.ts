import assert from "node:assert/strict";
import { test } from "node:test";
import { performance } from "node:perf_hooks";
import { originalPositionFor, TraceMap } from "@jridgewell/trace-mapping";
import { compileTurbopackSfc } from "./loader.ts";
import { withVize, vizeTurbopack } from "./index.ts";
import styleLoader from "./style-loader.ts";
import scopeLoader from "./scope-loader.ts";
import { createGeneratedNameAllocator } from "@vizejs/builder-shared/module-output";
import type { LoaderContext, SfcLoaderOptions } from "./types.ts";

const filename = "/fixture/组件 space.vue";
const options: SfcLoaderOptions = {
  root: "/fixture",
  isProduction: false,
  ssr: false,
  styleLanguages: ["css"],
  sourceMap: true,
};

function trace(code: string, map: object, marker: string) {
  const offset = code.indexOf(marker);
  assert.notEqual(offset, -1);
  const prefix = code.slice(0, offset);
  return originalPositionFor(new TraceMap(JSON.stringify(map)), {
    line: prefix.split("\n").length,
    column: offset - prefix.lastIndexOf("\n") - 1,
  });
}

function runStyle(
  loader: Function,
  source: string,
  query: string,
  inputMap?: object,
): Promise<{ code: string; map: object | null }> {
  return new Promise((resolve, reject) => {
    const context: LoaderContext<{}> = {
      resourcePath: filename,
      resourceQuery: query,
      rootContext: "/fixture",
      getOptions: () => ({}),
      addDependency() {},
      emitWarning(error) {
        reject(error);
      },
      async: () => (error, code, map) => {
        if (error) reject(error);
        else
          resolve({ code: code!, map: typeof map === "string" ? JSON.parse(map) : (map ?? null) });
      },
    };
    loader.call(context, source, inputMap);
  });
}

void test("native export byte geometry stays correct after astral and non-ASCII authored text", () => {
  const source =
    '<script setup lang="ts">\r\nconst emoji: string = "🐱";\r\nconst answer = 42;\r\n</script>\r\n<template>{{emoji}} {{answer}}</template>\r\n<style scoped>.answer{color:red}</style>';
  const output = compileTurbopackSfc(filename, source, options);
  assert.match(output.code, /const emoji = "🐱"/);
  assert.match(output.code, /const __vize_component__ =/);
  assert.match(output.code, /export default __vize_component__;/);
  assert.match(output.code, /__scopeId = "data-v-[a-f0-9]+"/);
  assert.ok(output.map);
  assert.equal(output.map.file, `${filename}.js`);
  assert.deepEqual(trace(output.code, output.map, "const answer"), {
    source: filename,
    line: 3,
    column: 0,
    name: null,
  });
  assert.equal(trace(output.code, output.map, "import.meta.turbopackHot").source, null);
  assert.equal(trace(output.code, output.map, 'import "./组件 space.vue?').source, null);
});

void test("authored names and fake export text cannot capture generated bindings", () => {
  const output = compileTurbopackSfc(
    filename,
    `<script setup>
const __vize_component__ = '🐱 export default fake';
const _sfc_main = 1;
const _imports_0 = 'user';
</script><template><img src="./pixel.png">{{__vize_component__}} {{_sfc_main}} {{_imports_0}}</template>`,
    options,
  );
  assert.match(output.code, /const __vize_component___ =/);
  assert.match(output.code, /const _imports_0_ = typeof/);
  assert.match(output.code, /export default __vize_component___;/);
  assert.ok(output.code.includes("🐱 export default fake"));
});

void test("hot reload preserves authored import names, requests and style ownership", () => {
  const output = compileTurbopackSfc(
    filename,
    `<script setup>
import { label as _sfc_main } from './_sfc_main.js';
</script><template>{{_sfc_main}}</template><style src="./_sfc_main.css"></style>`,
    options,
  );
  assert.ok(output.code.includes('["./_sfc_main.js:_sfc_main", _sfc_main]'));
  assert.ok(output.code.includes('"./_sfc_main.css?vue&'));
  assert.ok(output.code.includes('src\\\":\\\"./_sfc_main.css'));
  assert.match(output.code, /__vize_component__.__hmrId/);
});

void test("generated bindings reserve escaped names and keep ordinary modules off the AST path", () => {
  let parses = 0;
  const plain = createGeneratedNameAllocator("const value = 1;", () => parses++);
  assert.equal(plain("__vize_component__"), "__vize_component__");
  assert.equal(parses, 0);
  const escape = String.fromCharCode(92) + "u005f";
  const source = `<script>const ${escape}_vize_component__ = 1; export default { name: "FixtureCounter" };</script><template><div>ok</div></template>`;
  const output = compileTurbopackSfc(filename, source, options);
  assert.match(output.code, /const __vize_component___ =/);
  const collisions = Array.from(
    { length: 512 },
    (_, index) => `const __vize_name_${index} = ${index};`,
  ).join("\n");
  const reserved = createGeneratedNameAllocator(collisions, () => parses++);
  for (let index = 0; index < 512; index++)
    assert.equal(reserved(`__vize_name_${index}`), `__vize_name_${index}_`);
  assert.equal(parses, 1);
});

void test("server and production compilation have deterministic scoped identities and no HMR", () => {
  const source =
    '<template><div class="box">hello</div></template><style scoped>.box{color:red}</style>';
  const one = compileTurbopackSfc(filename, source, { ...options, isProduction: true });
  const two = compileTurbopackSfc("/other/组件 space.vue", source, {
    ...options,
    root: "/other",
    isProduction: true,
    ssr: true,
  });
  const scope = (code: string) => /__scopeId = "([^"]+)"/.exec(code)?.[1];
  assert.equal(scope(one.code), scope(two.code));
  assert.ok(scope(one.code));
  assert.doesNotMatch(one.code, /turbopackHot/);
  assert.doesNotMatch(two.code, /turbopackHot/);
  assert.match(two.code, /ssrRender/);
});

void test("unsupported preprocessing and block modules fail with actionable errors", () => {
  assert.throws(
    () =>
      compileTurbopackSfc(
        filename,
        '<template><div/></template><style lang="scss">$c:red;.a{color:$c}</style>',
        options,
      ),
    /configure styleLoaders.scss/,
  );
  assert.throws(
    () => compileTurbopackSfc(filename, '<template lang="pug">div</template>', options),
    /template preprocessor/,
  );
  assert.throws(
    () => compileTurbopackSfc(filename, '<script lang="coffee">x = 1</script><template/>', options),
    /script language/,
  );
  assert.throws(
    () => compileTurbopackSfc(filename, '<template src="./view.html"></template>', options),
    /external script\/template/,
  );
  assert.throws(
    () => compileTurbopackSfc(filename, "<template><div/></template><i18n>{}</i18n>", options),
    /custom block/,
  );
  assert.throws(
    () =>
      compileTurbopackSfc(
        filename,
        "<template><div/></template><style module>.a{}</style><style module>.b{}</style>",
        options,
      ),
    /duplicate CSS module/,
  );
  assert.throws(
    () => compileTurbopackSfc(filename, "<template><div></template>", options),
    /\[vize\]/,
  );
});

void test("stable compilation rejects the real parser's Vapor marker presence", () => {
  for (const script of [
    "<script vapor>export default {}</script>",
    '<script vapor="false">export default {}</script>',
    "<SCRIPT vapor>export default {}</SCRIPT>",
    "<script setup vapor>const n = 1;</script>",
    '<script setup vapor="">const n = 1;</script>',
    '<script setup="false" vapor="false">const n = 1;</script>',
  ]) {
    assert.throws(
      () => compileTurbopackSfc(filename, `${script}<template><div/></template>`, options),
      /Vapor SFC blocks are not supported/,
      script,
    );
  }
  for (const script of [
    "<script VAPOR>export default {}</script>",
    '<script VaPoR="false">export default {}</script>',
    '<script data-vapor nonvapor title="vapor">export default {}</script>',
    "<!-- <script vapor> --><script>export default {}</script>",
    '<script setup>const marker = "<script vapor>";</script>',
  ]) {
    assert.doesNotThrow(
      () => compileTurbopackSfc(filename, `${script}<template><div/></template>`, options),
      script,
    );
  }
});

void test("style extraction preserves authored CRLF and UTF-16 mapping after an astral prefix", async () => {
  const source = "<template>🐱</template>\r\n<style scoped>\r\n.box { color: red }\r\n</style>";
  const extracted = await runStyle(styleLoader, source, "?vue&type=style&index=0");
  assert.equal(extracted.code, "\r\n.box { color: red }\r\n");
  assert.ok(extracted.map);
  assert.deepEqual(trace(extracted.code, extracted.map, ".box"), {
    source: filename,
    line: 3,
    column: 0,
    name: null,
  });
  const scoped = await runStyle(
    scopeLoader,
    extracted.code,
    "?scoped=data-v-123abc",
    extracted.map!,
  );
  assert.match(scoped.code, /\.box\[data-v-123abc\]/);
  assert.ok(scoped.map);
  assert.equal(trace(scoped.code, scoped.map, ".box").source, filename);
  assert.equal(trace(scoped.code, scoped.map, ".box").line, 3);
  await assert.rejects(runStyle(styleLoader, source, "?index=1"), /missing/);
  assert.equal((await runStyle(styleLoader, source, "?index=1&dev")).code, "");
  await assert.rejects(runStyle(styleLoader, source, "?index=01"), /Invalid style block index/);
  await assert.rejects(runStyle(scopeLoader, ".box{}", "?scoped=forged"), /Invalid style scope/);
});

void test("configuration preserves host settings and explicit Vue flags", () => {
  const original = {
    compiler: { define: { __VUE_OPTIONS_API__: false, MY_VALUE: "kept" } },
    turbopack: {
      resolveAlias: { local: "./local.js" },
      resolveExtensions: [".mdx", ".js"],
      rules: { "*": { loaders: ["raw-loader"], as: "*.js" } },
    },
  };
  const snapshot = JSON.stringify(original);
  const config = withVize(original, { root: "/fixture" });
  assert.equal(JSON.stringify(original), snapshot);
  assert.equal(config.compiler?.define?.__VUE_OPTIONS_API__, false);
  assert.equal(config.compiler?.define?.MY_VALUE, "kept");
  assert.deepEqual(config.turbopack?.resolveExtensions, [".vue", ".mdx", ".js"]);
  assert.deepEqual(config.turbopack?.resolveAlias, original.turbopack.resolveAlias);
  const rules = config.turbopack?.rules?.["*"];
  assert.ok(Array.isArray(rules));
  assert.deepEqual(rules[0], original.turbopack.rules["*"]);
  assert.throws(() => vizeTurbopack({ styleLoaders: { "bad/lang": [] } }), /language names/);
});

void test("record native adapter cold and repeated-source timings without a compile cache", (t) => {
  const source = "<script setup>const value=1</script><template>{{value}}</template>";
  const samples = Array.from({ length: 10 }, () => {
    const start = performance.now();
    const output = compileTurbopackSfc(filename, source, { ...options, isProduction: true });
    return { ms: performance.now() - start, code: output.code };
  });
  assert.equal(new Set(samples.map(({ code }) => code)).size, 1);
  t.diagnostic(
    JSON.stringify({
      samplesMs: samples.map(({ ms }) => ms),
      sourceBytes: Buffer.byteLength(source),
      resultBytes: Buffer.byteLength(samples[0].code),
    }),
  );
});
