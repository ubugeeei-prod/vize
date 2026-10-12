import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  checkMap,
  checkVersions,
  compilerSourceMapPrototype,
  hash,
} from "./support/native-sfc-ssr-reference.ts";
import {
  checkOfficialSetupMap,
  checkSetupCustody,
  executeExternalSetup,
  executeInlineSetup,
  primarySetupBindings,
  stockSetupSfc,
} from "./support/native-sfc-setup-ssr-reference.ts";
import {
  assertDirectChildRuntime,
  originalNestedSource,
} from "./support/native-ssr-direct-child-successor.ts";

const root = new URL("../../crates/vize_atelier_sfc/tests/fixtures/", import.meta.url);
const corpus = JSON.parse(
  fs.readFileSync(new URL("native-setup-ssr-direct-child-sources.json", root), "utf8"),
);
const golden = JSON.parse(
  fs.readFileSync(new URL("native-setup-ssr-direct-child-output.json", root), "utf8"),
);
const capturePath = process.env.VIZE_NATIVE_SETUP_SSR_CAPTURE;
const requireNative = process.env.VIZE_L4_SSR_REQUIRE_NATIVE === "1";

void test(
  "whole direct-child setup modules preserve native ownership, maps and actual SSR",
  {
    skip: !capturePath && !requireNative,
  },
  async () => {
    assert(capturePath, "fresh native source capture is mandatory");
    checkVersions();
    assert.equal(corpus.modules.length, 8);
    assert.equal(corpus.refusals.length, 15);
    assert.equal(corpus.modules[0].source, originalNestedSource);
    const capture = JSON.parse(fs.readFileSync(`${capturePath}.direct-child.json`, "utf8"));
    assert.equal(capture.schema, "vize.native-ssr.direct-child-capture");
    assert.equal(capture.complete, true);
    assert.deepEqual(
      capture.modules.map(({ name, source, filename }: any) => ({ name, source, filename })),
      corpus.modules.map(({ name, source, filename }: any) => ({ name, source, filename })),
    );
    assert.deepEqual(
      capture.refusals.map(({ name, source }: any) => ({ name, source })),
      corpus.refusals.map(({ name, source }: any) => ({ name, source })),
    );
    for (const row of capture.refusals) {
      assert.equal(row.completeModule, false);
      assert.equal(typeof row.reason, "string");
    }
    const rows = [];
    for (const [index, actual] of capture.modules.entries()) {
      const authored = corpus.modules[index],
        filename = `/native-ssr/${authored.filename}`;
      const bindings = primarySetupBindings(authored.source, filename);
      const inline = await stockSetupSfc(authored.source, filename, false);
      const external = await stockSetupSfc(authored.source, filename, true);
      checkOfficialSetupMap(inline, authored.source, filename);
      checkOfficialSetupMap(external, authored.source, filename, true);
      const officialInline = await executeInlineSetup(inline);
      const officialExternal = await executeExternalSetup(external, bindings, authored.updates);
      const expected = {
        ...authored,
        bindings,
        inline,
        external,
        inlineRuntime: officialInline,
        externalRuntime: officialExternal,
      };
      checkSetupCustody(actual, expected);
      checkMap({ ...actual, map: actual.mapValue });
      const native = await executeExternalSetup(
        { code: actual.code },
        bindings,
        authored.updates,
        true,
      );
      assert.deepEqual(
        native.map((run) => run.initial.html),
        officialInline.map((run) => run.html),
      );
      assert.equal(native[0].initial.html, authored.expectedPlainHtml);
      assert.equal(officialInline[0].html, authored.expectedPlainHtml);
      if (authored.expectedUpdatedPlainHtml)
        assert.equal(native[0].updated.html, authored.expectedUpdatedPlainHtml);
      for (const [context, run] of native.entries())
        for (const [phase, outcome] of Object.entries(run) as any[]) {
          const official = officialExternal[context][phase];
          assert.deepEqual({ ...outcome, modules: official.modules }, official);
          assert.deepEqual(outcome.modules, []);
        }
      if (authored.name === "nested-let-js") {
        assert.equal(actual.code.split("$setup.value").length, 2);
        await assert.rejects(
          executeExternalSetup(
            { code: actual.code.replace("$setup.value", "_ctx.value") },
            bindings,
            authored.updates,
            true,
          ),
          /SSR setup _ctx fallback value/,
        );
        const mutated = await executeExternalSetup(
          { code: actual.code.replace("$setup.value", "0") },
          bindings,
          authored.updates,
          true,
        );
        assert.notEqual(mutated[0].initial.html, authored.expectedPlainHtml);
        assert.notEqual(mutated[0].updated.html, authored.expectedUpdatedPlainHtml);
      }
      rows.push({
        name: actual.name,
        sourceSha256: hash(actual.source),
        codeSha256: hash(actual.code),
        mapSha256: hash(actual.mapText),
        mapValueSha256: hash(JSON.stringify(actual.mapValue)),
        linksSha256: hash(JSON.stringify(actual.links)),
        setupSha256: hash(JSON.stringify(actual.setup)),
        inline,
        external,
        bindings,
        native,
        officialInline,
        officialExternal,
      });
    }
    const runtime = {
      schema: "vize.native-ssr.direct-child-runtime",
      complete: true,
      rows,
      refusals: capture.refusals,
      completeUpstreamMapParity: false,
      viteContextParity: false,
    };
    if (process.env.VIZE_NATIVE_SETUP_SSR_RUNTIME_CAPTURE)
      fs.writeFileSync(
        `${process.env.VIZE_NATIVE_SETUP_SSR_RUNTIME_CAPTURE}.direct-child.json`,
        JSON.stringify(runtime, null, 2) + "\n",
      );
    // Retain the whole real outcome before a deliberately pending freeze fails closed.
    assert.deepEqual(capture, golden.capture);
    const compare = () =>
      assertDirectChildRuntime(runtime, golden.runtime, compilerSourceMapPrototype);
    compare();

    const map = runtime.rows[0].external.map;
    map.unknown = undefined;
    assert.throws(compare);
    delete map.unknown;
    Object.defineProperty(map, "hiddenUnknown", { value: undefined, configurable: true });
    assert.throws(compare);
    delete map.hiddenUnknown;
    Object.defineProperty(map, "file", { writable: false });
    assert.throws(compare);
    Object.defineProperty(map, "file", { writable: true });
    delete map.sourceRoot;
    assert.throws(compare);
    map.sourceRoot = undefined;
    map.file = "unexpected.vue";
    assert.throws(compare);
    map.file = undefined;
    const mappings = map.mappings;
    map.mappings += ";AAAA";
    assert.throws(compare);
    map.mappings = mappings;
    const originalMap = runtime.rows[2].external.dependencies[0].originalMap;
    assert.equal(Object.getPrototypeOf(originalMap), compilerSourceMapPrototype);
    Object.setPrototypeOf(originalMap, Object.prototype);
    assert.throws(compare);
    Object.setPrototypeOf(originalMap, compilerSourceMapPrototype);
    const outcome = runtime.rows[0].native[0].initial;
    const html = outcome.html;
    outcome.html += "unexpected";
    assert.throws(compare);
    outcome.html = html;
    compare();
  },
);
