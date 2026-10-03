import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import { test } from "node:test";
import { officialCompilerVapor, vueVaporVersion } from "./support/vue-vapor-release.mjs";

const pack = JSON.parse(readFileSync(new URL("../../davinci/vize_l4/tests/fixtures/native-vapor-vue-3.6.0-rc.9.json", import.meta.url), "utf8"));
const required = process.env.VIZE_NATIVE_VAPOR_REQUIRE_CAPTURE === "1";
assert.ok(!required || process.env.VIZE_NATIVE_VAPOR_CAPTURE, "mandatory original Rust module capture path");
const captures = process.env.VIZE_NATIVE_VAPOR_CAPTURE
  ? JSON.parse(readFileSync(process.env.VIZE_NATIVE_VAPOR_CAPTURE, "utf8")) : null;
if (required) assert.equal(captures?.length, 12, "mandatory original Rust module denominator");
const ui = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const vue = createRequire(ui.resolve("vue-vapor-runtime/package.json"));
const { SourceMapConsumer } = createRequire(vue.resolve("@vue/compiler-sfc"))("source-map-js");

function position(text, bytes) {
  const prefix = Buffer.from(text).subarray(0, bytes).toString("utf8");
  const lines = prefix.split(/\r\n|[\r\n\u2028\u2029]/u);
  return { line: lines.length, column: lines.at(-1).length };
}

test("frozen complete modules retain exact official rc.9 output and independent maps", () => {
  assert.equal(vueVaporVersion, pack.version);
  assert.equal(pack.fixtures.length, 12);
  for (const fixture of pack.fixtures) {
    const upstream = officialCompilerVapor.compile(fixture.template, { mode: "module", sourceMap: true, filename: "NativeVapor.vue" });
    assert.equal(upstream.code, fixture.upstreamCode, fixture.id);
    assert.deepEqual(upstream.map, fixture.upstreamMap, fixture.id);
    const consumer = new SourceMapConsumer(fixture.map);
    const decoded = [];
    consumer.eachMapping((mapping) => decoded.push(mapping));
    assert.equal(decoded.length, fixture.anchors.length, fixture.id);
    for (const anchor of fixture.anchors) {
      const actual = consumer.originalPositionFor(position(fixture.code, anchor.generated));
      assert.deepEqual(actual, { source: "NativeVapor.vue", ...position(fixture.source, anchor.source), name: null }, fixture.id);
    }
    assert.deepEqual(fixture.map.sourcesContent, [fixture.source]);
  }
});

test("genuine captured modules and maps equal the entire frozen output", () => {
  if (!captures) { assert.ok(!required); return; }
  assert.equal(captures.length, pack.fixtures.length);
  for (const [index, fixture] of pack.fixtures.entries()) {
    const actual = captures[index];
    assert.equal(actual.id, fixture.id);
    assert.equal(actual.source, fixture.source);
    assert.equal(actual.code, fixture.code);
    assert.deepEqual(actual.map, fixture.map);
    assert.ok(Number.isInteger(actual.nodes));
    assert.ok(Number.isInteger(actual.roots));
  }
});

test("whole modules execute with real rc.9 nodes, comments, attrs and clone disposal", () => {
  const inputs = pack.fixtures.map((fixture, index) => {
    const roots = officialCompilerVapor.parse(fixture.template).children.filter((node) => node.type !== 3);
    return { id: fixture.id, code: captures?.[index].code ?? fixture.code,
      upstreamCode: fixture.upstreamCode, rootElement: roots.length === 1 && roots[0].type === 1 };
  });
  const runtime = spawnSync(process.execPath, [new URL("./support/native-vapor-runtime.mjs", import.meta.url).pathname], {
    input: JSON.stringify(inputs), encoding: "utf8", timeout: 60_000, maxBuffer: 8 * 1024 * 1024,
  });
  assert.equal(runtime.status, 0, runtime.stderr);
  const traces = JSON.parse(runtime.stdout);
  assert.equal(traces.length, 12);
  for (const trace of traces) assert.ok(trace.traces.length > 0);
  if (process.env.VIZE_NATIVE_VAPOR_RUNTIME_CAPTURE)
    writeFileSync(process.env.VIZE_NATIVE_VAPOR_RUNTIME_CAPTURE, JSON.stringify({ capturedFromRust: !!captures, version: vueVaporVersion, traces }, null, 2));
});
