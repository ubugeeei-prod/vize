import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { TraceMap, originalPositionFor } from "@jridgewell/trace-mapping";
import vizeLoader from "./index.ts";
import type { VizeSfcLoaderOptions } from "../types/index.ts";
import type { SourceMapV3 } from "../shared/source-map.ts";

const source = '<script>export default { name: "Example" }</script><template><div /></template>';

function compileModule(
  options: VizeSfcLoaderOptions,
  mode = "development",
  context: { sourceMap?: boolean; source?: string; rootContext?: string } = { sourceMap: false },
) {
  const rootContext = context.rootContext ?? path.resolve("/project");
  let output: string | undefined;
  let outputMap: SourceMapV3 | undefined;
  vizeLoader.call(
    {
      resourcePath: path.join(rootContext, "src", "App.vue"),
      resourceQuery: "",
      rootContext,
      context: path.join(rootContext, "src"),
      mode,
      sourceMap: context.sourceMap,
      getOptions: () => ({ hotReload: false, ...options }),
      addDependency() {},
      emitWarning() {},
      emitError(error: Error) {
        throw error;
      },
      async: () => (error: Error | null, code: string, map?: string) => {
        if (error) throw error;
        output = code;
        outputMap = map ? (JSON.parse(map) as SourceMapV3) : undefined;
      },
    } as never,
    context.source ?? source,
  );
  assert.equal(typeof output, "string");
  return { code: output!, map: outputMap };
}

function compile(options: VizeSfcLoaderOptions, mode = "development"): string {
  return compileModule(options, mode).code;
}

void test("relative loader root controls development component paths", () => {
  assert.match(compile({ root: "src", isProduction: false }), /__file = "App.vue"/);
  assert.match(compile({ isProduction: false }), /__file = "src\/App.vue"/);
});

void test("loader production override controls component metadata independently of build mode", () => {
  assert.doesNotMatch(compile({ isProduction: true }), /__file/);
  assert.match(compile({ isProduction: false }, "production"), /__file/);
});

void test("the SFC loader forwards native maps through output assembly", () => {
  const { code, map } = compileModule({ sourceMap: true });
  assert.ok(map);
  const index = code.indexOf("name:");
  const prefix = code.slice(0, index);
  const original = originalPositionFor(new TraceMap(JSON.stringify(map)), {
    line: prefix.split("\n").length,
    column: index - prefix.lastIndexOf("\n") - 1,
  });
  assert.equal(original.line, 1);
  assert.equal(original.column, source.indexOf("name:"));
  assert.ok(original.source?.endsWith("/src/App.vue"));
  assert.deepEqual(map.sourcesContent, [source]);
});

void test("source-map options override nested aliases and Rspack context without reusing stale cache entries", () => {
  assert.ok(compileModule({ compilerOptions: { sourceMap: true } }).map);
  assert.equal(
    compileModule({ sourceMap: false, compilerOptions: { sourceMap: true } }).map,
    undefined,
  );
  assert.ok(compileModule({ sourceMap: true, compilerOptions: { sourceMap: false } }).map);
  assert.ok(compileModule({}, "development", { sourceMap: true }).map);
  assert.equal(compileModule({}, "development", { sourceMap: false }).map, undefined);
  assert.ok(compileModule({}, "development", {}).map);
  assert.equal(compileModule({}, "production", {}).map, undefined);
});

void test("SSR aliases reach native and top-level false disables SSR explicitly", () => {
  const server = compile({ compilerOptions: { ssr: true }, hotReload: true });
  assert.match(server, /ssrRender/);
  assert.doesNotMatch(server, /module\.hot/);
  const client = compile({ ssr: false, compilerOptions: { ssr: true } });
  assert.doesNotMatch(client, /ssrRender/);
  assert.match(client, /\.render\b/);
});

void test("Vapor aliases reach native and top-level false restores VDOM output", () => {
  assert.match(compile({ compilerOptions: { vapor: true } }), /__vapor/);
  assert.doesNotMatch(compile({ vapor: false, compilerOptions: { vapor: true } }), /__vapor/);
});

for (const externalScript of [false, true]) {
  void test(`source maps survive external ${externalScript ? "script" : "template"} inlining`, (t) => {
    const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-src-map-")));
    t.after(() => fs.rmSync(root, { recursive: true, force: true }));
    fs.mkdirSync(path.join(root, "src"));
    const script = 'export default { name: "ExternalProbe" };';
    fs.writeFileSync(path.join(root, "src", "script.js"), script);
    fs.writeFileSync(path.join(root, "src", "template.html"), "<div>\n<p>External</p>\n</div>");
    const sfc =
      '<template src="./template.html"></template>\n' +
      (externalScript ? '<script src="./script.js"></script>' : `<script>${script}</script>`);
    const { code, map } = compileModule({ sourceMap: true }, "development", {
      source: sfc,
      rootContext: root,
    });
    assert.ok(map);
    const offset = code.indexOf("name:");
    const prefix = code.slice(0, offset);
    const original = originalPositionFor(new TraceMap(JSON.stringify(map)), {
      line: prefix.split("\n").length,
      column: offset - prefix.lastIndexOf("\n") - 1,
    });
    assert.equal(original.source, path.join(root, "src", externalScript ? "script.js" : "App.vue"));
    assert.equal(original.line, externalScript ? 1 : 2);
    assert.equal(
      original.column,
      script.indexOf("name:") + (externalScript ? 0 : "<script>".length),
    );
    assert.ok(map.sourcesContent?.includes(externalScript ? script : sfc));
  });
}
