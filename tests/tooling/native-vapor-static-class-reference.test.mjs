import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { createRequire } from "node:module";
import { captureVaporProcess } from "./support/native-vapor-process-capture.mjs";

const fixture = new URL(
  "../../davinci/vize_l4/tests/fixtures/native-vapor-static-class-controls.json",
  import.meta.url,
);
const source = JSON.parse(readFileSync(fixture, "utf8"));
const required = process.env.VIZE_NATIVE_VAPOR_REQUIRE_CAPTURE === "1";
const capturePath = process.env.VIZE_NATIVE_VAPOR_CAPTURE
  ? `${process.env.VIZE_NATIVE_VAPOR_CAPTURE}.class-cases.json`
  : null;
assert(!required || capturePath, "actual supplemental class capture required");
const captures = capturePath ? JSON.parse(readFileSync(capturePath, "utf8")) : null;
const ui = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const vue = createRequire(ui.resolve("vue-vapor-runtime/package.json"));
const { SourceMapConsumer } = createRequire(vue.resolve("@vue/compiler-sfc"))("source-map-js");
function position(text, offset) {
  assert(Number.isInteger(offset) && offset >= 0 && offset <= Buffer.byteLength(text));
  const bytes = Buffer.from(text).subarray(0, offset);
  const prefix = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  assert(Buffer.from(prefix).equals(bytes));
  const lines = prefix.split(/\r\n|[\r\n\u2028\u2029]/u);
  return { line: lines.length, column: lines.at(-1).length };
}
function wholeMap(packet, original) {
  assert.deepEqual(JSON.parse(packet.mapText), packet.map);
  assert.deepEqual(packet.map.sourcesContent, [original]);
  assert.deepEqual(packet.map.sources, ["NativeVaporClass.vue"]);
  const consumer = new SourceMapConsumer(packet.map),
    decoded = [];
  consumer.eachMapping((mapping) => decoded.push(mapping));
  assert.equal(decoded.length, packet.links.length);
  for (const [index, link] of packet.links.entries()) {
    assert.equal(link.generated.start, link.generated.end);
    assert.equal(link.authored.start, link.authored.end);
    const generated = position(packet.code, link.generated.start),
      authored = position(original, link.authored.start);
    assert.deepEqual(decoded[index], {
      source: "NativeVaporClass.vue",
      generatedLine: generated.line,
      generatedColumn: generated.column,
      originalLine: authored.line,
      originalColumn: authored.column,
      name: null,
    });
  }
}
await test(
  "all16 class templates and14 genuine components retain whole independent VLQ/link custody",
  { skip: !captures },
  () => {
    assert.equal(captures.length, 16);
    let components = 0;
    for (const [index, row] of captures.entries()) {
      const expected = source.positive[index];
      assert.equal(row.id, expected.id);
      assert.equal(row.source, expected.source);
      assert.equal(row.code, expected.expectedCode);
      wholeMap(row, row.source);
      if (row.component.code === null) {
        assert(["multiple-roots", "comment-root"].includes(row.id));
        assert.match(row.component.error, /ComponentFragmentLifecycle/u);
      } else {
        components++;
        assert.equal(row.component.error, null);
        wholeMap(row.component, row.source);
      }
    }
    assert.equal(components, 14);
  },
);
await test(
  "genuine dev and production component clones hydrate with official DOM/SSR authority in all3 contexts",
  { skip: !captures },
  () => {
    const observed = [];
    for (const mode of ["development", "production"]) {
      const argv = [
        new URL("./support/native-vapor-static-class-runtime.ts", import.meta.url).pathname,
        fixture.pathname,
        capturePath,
      ];
      const actual = spawnSync(process.execPath, argv, {
        env: { ...process.env, NODE_ENV: mode },
        encoding: "utf8",
        timeout: 120_000,
        maxBuffer: 64 * 1024 * 1024,
      });
      captureVaporProcess(`static-class-${mode}`, { argv, source }, actual);
      if (process.env.VIZE_NATIVE_VAPOR_CLASS_RUNTIME_CAPTURE)
        writeFileSync(
          `${process.env.VIZE_NATIVE_VAPOR_CLASS_RUNTIME_CAPTURE}.${mode}.stdout`,
          actual.stdout ?? "",
        );
      observed.push({ mode, actual });
    }
    // Both complete child streams are retained before any mode/case assertions.
    for (const { mode, actual } of observed) {
      assert.equal(actual.signal, null);
      assert.equal(actual.error, undefined);
      assert.equal(actual.status, 0, actual.stderr);
      const packet = JSON.parse(actual.stdout);
      assert.equal(packet.mode, mode);
      assert.equal(packet.observations.length, 16);
      for (const row of packet.observations) {
        assert.equal(row.failure, null, row.id);
        assert(row.primary.dom.code && row.primary.ssr.code && row.primary.vapor.code);
        assert(row.primary.dom.map && row.primary.ssr.map && row.primary.vapor.map);
        if (["multiple-roots", "comment-root"].includes(row.id)) {
          assert.deepEqual(row.contexts, []);
          continue;
        }
        assert.equal(row.contexts.length, 3);
        for (const [index, context] of row.contexts.entries()) {
          assert.deepEqual(context.native.tree, context.officialDom.tree, row.id);
          assert.deepEqual(context.native.tree, context.officialSsr.tree, row.id);
          if (index !== 1) assert.deepEqual(context.native.tree, row.literalTree, row.id);
          assert.deepEqual(context.cloned.tree, context.native.tree);
          assert(context.cloned.distinctNodes);
          assert.deepEqual(context.hydrated.tree, context.native.tree);
          assert(context.hydrated.originalNodes > 0 && context.hydrated.retained.every(Boolean));
          for (const trace of [
            context.officialDom,
            context.native,
            context.cloned,
            context.hydrated,
          ]) {
            assert.equal(trace.renderIdentity, true);
            assert.deepEqual(trace.diagnostics, []);
            assert.deepEqual(trace.unmounted, []);
          }
          assert.deepEqual(context.officialSsr.diagnostics, []);
        }
      }
    }
  },
);
