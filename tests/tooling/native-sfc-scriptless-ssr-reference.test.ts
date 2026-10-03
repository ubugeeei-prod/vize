import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  checkMap,
  checkVersions,
  directModule,
  hash,
  renderModule,
  stockSfc,
} from "./support/native-sfc-ssr-reference.ts";

const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native-sfc-ssr-vue-3.5.35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const refusalIds = [
  "ordinary-empty",
  "setup-empty",
  "setup-constant",
  "global-style",
  "scoped-style",
  "custom",
  "external",
  "missing",
  "search",
  "component",
  "slot",
  "binding",
  "interpolation",
  "for",
  "class",
  "nested-whitespace",
  "handler-global",
  "handler-syntax",
];

test("fourteen official complete SFC transforms retain primary modules, maps and SSR context", async () => {
  checkVersions();
  assert.equal(pack.schema, "vize.native-sfc.scriptless-ssr-reference");
  assert.equal(pack.version, 1);
  assert.deepEqual(pack.compiler, { name: "@vitejs/plugin-vue", version: "6.0.7", vue: "3.5.35" });
  assert.equal(pack.fixtures.length, 14);
  assert.equal(new Set(pack.fixtures.map((row: any) => row.id)).size, 14);
  for (const row of pack.fixtures) {
    assert.equal(row.filename, `/native-ssr/${row.id}.vue`);
    assert.equal(hash(row.source), row.sourceSha256);
    assert.equal(hash(row.referenceCode), row.referenceCodeSha256);
    assert.equal(hash(JSON.stringify(row.referenceMap)), row.referenceMapSha256);
    assert.equal(hash(row.referenceHelper), row.referenceHelperSha256);
    assert.deepEqual(row.referenceMap.sourcesContent, [row.source]);
    for (let repeat = 0; repeat < 2; repeat++) {
      const actual = await stockSfc(row.source, row.filename);
      assert.equal(actual.code, row.referenceCode);
      assert.equal(JSON.stringify(actual.map), JSON.stringify(row.referenceMap));
      assert.equal(actual.helper, row.referenceHelper);
    }
    const executions = await renderModule(row.referenceCode, row.referenceHelper);
    assert.deepEqual(executions, row.executions);
    assert(
      executions.every(
        (execution) => execution.modules.length === 1 && execution.modules[0] === `${row.id}.vue`,
      ),
    );
    assert.deepEqual(
      await directModule(row.referenceCode, row.referenceHelper),
      executions.map((execution) => execution.html),
    );
    if (row.id.startsWith("handler-")) {
      assert(
        !row.referenceCode.includes("$event") &&
          !row.referenceCode.includes("__vize_ssr_event_probe"),
      );
    }
  }
});

test("primary judge detects missing actual SSR-context registration and evaluated handlers", async () => {
  const row = pack.fixtures.find((row: any) => row.id === "handler-nested");
  const stripped = row.referenceCode.replace(
    '  ;(ssrContext.modules || (ssrContext.modules = new Set())).add("handler-nested.vue")\n',
    "",
  );
  assert.notEqual(stripped, row.referenceCode);
  assert.notDeepEqual(await renderModule(stripped, row.referenceHelper), row.executions);
  const evaluated = row.referenceCode.replace(
    "function _sfc_ssrRender(_ctx, _push, _parent, _attrs) {",
    "function _sfc_ssrRender(_ctx, _push, _parent, _attrs) { _ctx.missingHandler();",
  );
  assert.notEqual(evaluated, row.referenceCode);
  await assert.rejects(
    directModule(evaluated, row.referenceHelper),
    /static SSR read missingHandler/,
  );
});

const capturePath = process.env.VIZE_NATIVE_SFC_SSR_CAPTURE;
test(
  "fresh original SFC products execute entire components and every complete map anchor",
  { skip: !capturePath && process.env.VIZE_L4_SSR_REQUIRE_NATIVE !== "1" },
  async () => {
    assert(capturePath, "hosted SSR requires fresh whole-SFC source-built modules");
    const capture = JSON.parse(fs.readFileSync(capturePath, "utf8"));
    assert.equal(capture.custody, "once_selected_scriptless_sfc");
    assert.deepEqual(
      capture.modules.map((row: any) => row.id),
      pack.fixtures.map((row: any) => row.id),
    );
    const runtime = [];
    for (const [index, expected] of pack.fixtures.entries()) {
      const actual = capture.modules[index];
      assert.equal(actual.source, expected.source);
      assert.equal(actual.filename, expected.filename);
      assert.equal(actual.outcome, "complete_original_sfc_module");
      assert.deepEqual(JSON.parse(actual.mapText), actual.map);
      assert(actual.code.includes("export default _sfc_main"));
      assert(!actual.code.includes("useSSRContext"));
      const codec = checkMap(actual);
      const native = await renderModule(actual.code);
      const official = await renderModule(expected.referenceCode, expected.referenceHelper);
      assert.deepEqual(
        native.map((execution) => execution.html),
        official.map((execution) => execution.html),
      );
      assert(native.every((execution) => execution.modules.length === 0));
      const direct = await directModule(actual.code);
      assert.deepEqual(
        direct,
        native.map((execution) => execution.html),
      );
      if (actual.id.startsWith("handler-")) {
        assert.equal(
          Buffer.from(actual.source)
            .subarray(actual.handler.span.start, actual.handler.span.end)
            .toString("utf8"),
          actual.handler.raw,
        );
        assert.equal(
          actual.handler.raw,
          actual.id === "handler-root"
            ? "const unused=&quot;雪🌸&quot;; $event.count+=2;"
            : "$event.count += 2;",
        );
        assert.equal(
          actual.handler.decoded,
          actual.id === "handler-root"
            ? 'const unused="雪🌸"; $event.count+=2;'
            : "$event.count += 2;",
        );
        assert(!actual.code.includes("$event") && !actual.code.includes("__vize_ssr_event_probe"));
        const changed = actual.code.replace(
          "function ssrRender(_ctx, _push, _parent, _attrs) {",
          "function ssrRender(_ctx, _push, _parent, _attrs) { _ctx.missingHandler();",
        );
        assert.notEqual(changed, actual.code);
        await assert.rejects(directModule(changed), /static SSR read missingHandler/);
      }
      if (actual.id === "handler-root") {
        const invalid = structuredClone(actual);
        const flower = actual.source.indexOf("🌸");
        assert(flower >= 0 && invalid.links.length > 0);
        invalid.links[0].authored.end = Buffer.byteLength(actual.source.slice(0, flower)) + 3;
        assert.throws(() => checkMap(invalid), /UTF-8 boundaries/);
      }
      if (actual.id === "import-shaped-comment") {
        assert.equal(native[0].html, "<!--from \"vue\"; from 'vue/server-renderer'-->");
      }
      if (actual.id === "ordinary-props") {
        const missing = structuredClone(actual);
        const decoded = codec.decode(missing.map.mappings);
        const segments = decoded.find((line: number[][]) => line.length > 1);
        assert(segments);
        segments.splice(1, 1);
        missing.map.mappings = codec.encode(decoded);
        assert.throws(() => checkMap(missing), /complete start anchor/);
        const endOnly = structuredClone(actual);
        const link = endOnly.links.find(
          (link: any) => link.segment && link.generated.start !== link.generated.end,
        );
        assert(link);
        link.generated.start = link.generated.end;
        link.authored.start = link.authored.end;
        assert.throws(() => checkMap(endOnly), /complete start anchor/);
      }
      runtime.push({
        id: actual.id,
        sourceSha256: hash(actual.source),
        codeSha256: hash(actual.code),
        mapSha256: hash(actual.mapText),
        mapValueSha256: hash(JSON.stringify(actual.map)),
        linksSha256: hash(JSON.stringify(actual.links)),
        referenceCodeSha256: expected.referenceCodeSha256,
        referenceMapSha256: expected.referenceMapSha256,
        native,
        direct,
        official,
      });
    }
    assert.deepEqual(
      capture.refusals.map((row: any) => row.id),
      refusalIds,
    );
    for (const row of capture.refusals) {
      assert.equal(row.outcome, "whole_sfc_refusal");
      assert(
        typeof row.reason === "string" && /^(Lowering|Ssr|Analysis|Assembly)\(/.test(row.reason),
      );
      assert(typeof row.source === "string" && row.source.length > 0);
    }
    if (process.env.VIZE_NATIVE_SFC_SSR_RUNTIME_CAPTURE)
      fs.writeFileSync(
        process.env.VIZE_NATIVE_SFC_SSR_RUNTIME_CAPTURE,
        JSON.stringify(
          {
            custody: capture.custody,
            modules: 14,
            refusals: capture.refusals,
            nativeWholeExecutions: 42,
            nativeDirectExecutions: 42,
            officialWholeExecutions: 42,
            completeUpstreamMapParity: false,
            viteContextParity: false,
            runtime,
          },
          null,
          2,
        ) + "\n",
      );
  },
);
