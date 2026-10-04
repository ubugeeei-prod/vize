import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { checkMap, checkVersions, hash } from "./support/native-sfc-ssr-reference.ts";
import {
  checkOfficialSetupMap,
  checkSetupCustody,
  executeExternalSetup,
  executeInlineSetup,
  primarySetupBindings,
  removeSetupMarker,
  insertSetupRead,
  insertConstSetter,
  stockSetupSfc,
} from "./support/native-sfc-setup-ssr-reference.ts";

const fixturesRoot = new URL("../../crates/vize_atelier_sfc/tests/fixtures/", import.meta.url);
const corpus = JSON.parse(
  fs.readFileSync(new URL("native_sfc_setup_ssr_sources.json", fixturesRoot), "utf8"),
);
const primaryFile = new URL("native-setup-ssr-vue-3.5.35.json", fixturesRoot);
const nativeFile = new URL("native-setup-ssr-output.json", fixturesRoot);
const requireNative = process.env.VIZE_L4_SSR_REQUIRE_NATIVE === "1";
const capturePath = process.env.VIZE_NATIVE_SETUP_SSR_CAPTURE;
const graphHashes = (graph: any) => ({
  code: hash(graph.code),
  map: hash(JSON.stringify(graph.map)),
  helper: hash(graph.helper),
  dependencies: hash(JSON.stringify(graph.dependencies)),
});

let primaryPromise: Promise<any> | undefined;
function primary() {
  primaryPromise ??= (async () => {
    checkVersions();
    assert.equal(corpus.modules.length, 15);
    assert.equal(new Set(corpus.modules.map((row: any) => row.name)).size, 15);
    const fixtures = [];
    for (const row of corpus.modules) {
      const filename = `/native-ssr/${row.filename}`;
      const bindings = primarySetupBindings(row.source, filename);
      const inline = await stockSetupSfc(row.source, filename, false);
      const external = await stockSetupSfc(row.source, filename, true);
      assert.deepEqual(await stockSetupSfc(row.source, filename, false), inline);
      assert.deepEqual(await stockSetupSfc(row.source, filename, true), external);
      checkOfficialSetupMap(inline, row.source, filename);
      checkOfficialSetupMap(external, row.source, filename, true);
      const inlineRuntime = await executeInlineSetup(inline);
      const externalRuntime = await executeExternalSetup(
        external,
        bindings,
        row.updates,
        false,
        row.handler ? ["value"] : [],
      );
      assert.deepEqual(
        inlineRuntime.map((run) => run.html),
        externalRuntime.map((run) => run.initial.html),
      );
      for (const run of [
        ...inlineRuntime,
        ...externalRuntime.flatMap((run) => Object.values(run)),
      ] as any[]) {
        assert.deepEqual(run.modules, [row.filename]);
      }
      if (row.handler) {
        assert(!inline.code.includes("$event") && !external.code.includes("$event"));
        assert(externalRuntime.every((run) => run.initial.html === run.updated.html));
      }
      fixtures.push({
        ...row,
        sourceSha256: hash(row.source),
        bindings,
        inline,
        external,
        inlineSha256: graphHashes(inline),
        externalSha256: graphHashes(external),
        inlineRuntime,
        externalRuntime,
      });
    }
    return {
      schema: "vize.native-sfc.setup-ssr-reference",
      version: 1,
      compiler: { name: "@vitejs/plugin-vue", version: "6.0.7", vue: "3.5.35" },
      fixtures,
    };
  })();
  return primaryPromise;
}

void test("fifteen official whole setup SFC graphs preserve source, TS query maps and both SSR modes", async () => {
  const actual = await primary();
  if (fs.existsSync(primaryFile))
    assert.equal(
      JSON.stringify(actual),
      JSON.stringify(JSON.parse(fs.readFileSync(primaryFile, "utf8"))),
    );
  else assert(!requireNative, "mandatory SSR needs the genuine pinned primary packet");
  if (process.env.VIZE_NATIVE_SETUP_SSR_PRIMARY_CAPTURE)
    fs.writeFileSync(
      process.env.VIZE_NATIVE_SETUP_SSR_PRIMARY_CAPTURE,
      JSON.stringify(actual, null, 2) + "\n",
    );
});

void test("actual primary controls reject _ctx reads and missing generated script-setup marker", async () => {
  const row = (await primary()).fixtures.find((row: any) => row.name === "root-let-js");
  assert.equal(row.external.code.split("$setup.value").length, 2);
  await assert.rejects(
    executeExternalSetup(
      { ...row.external, code: row.external.code.replace("$setup.value", "_ctx.value") },
      row.bindings,
    ),
    /SSR setup _ctx fallback value/,
  );
  await assert.rejects(
    executeExternalSetup(
      { ...row.external, code: removeSetupMarker(row.external.code) },
      row.bindings,
    ),
    /__isScriptSetup|undefined/,
  );
  const on = (await primary()).fixtures.find((row: any) => row.handler);
  await assert.rejects(
    executeExternalSetup(
      { ...on.external, code: insertSetupRead(on.external.code, "value") },
      on.bindings,
      on.updates,
      false,
      ["value"],
    ),
    /SSR evaluated ignored handler binding value/,
  );
});

function completePacket(capture: any) {
  assert.equal(capture.schema, 1);
  assert.equal(capture.complete, true, "partial packets cannot grant native runtime credit");
  assert.equal(Object.hasOwn(capture, "suiteCompletion"), false);
  assert.deepEqual(
    capture.modules.map((row: any) => row.name),
    corpus.modules.map((row: any) => row.name),
  );
  assert.equal(corpus.refusals.length, 20);
  assert.deepEqual(
    capture.refusals.map(({ name, source }: any) => ({ name, source })),
    corpus.refusals,
  );
  for (const row of capture.refusals) {
    assert.equal(row.completeModule, false);
    assert.equal(typeof row.reason, "string");
    assert.deepEqual(row.diagnostics, []);
  }
}

void test(
  "fresh genuine setup products bind every complete module/map/owner to real Vue SSR setupState",
  {
    skip: !capturePath && !requireNative,
  },
  async () => {
    assert(capturePath, "mandatory SSR needs fresh source-built original setup modules");
    const capture = JSON.parse(fs.readFileSync(capturePath, "utf8"));
    completePacket(capture);
    assert(
      fs.existsSync(nativeFile),
      "native pins must come from the actual complete hosted artifact",
    );
    const golden = JSON.parse(fs.readFileSync(nativeFile, "utf8"));
    assert.equal(golden.schema, "vize.native-sfc.setup-ssr-output");
    assert.equal(golden.version, 1);
    assert.deepEqual(capture, golden.capture);
    assert.throws(() => completePacket({ ...capture, complete: false }), /partial packets/);
    const reference = await primary(),
      runtime = [];
    for (const [index, actual] of capture.modules.entries()) {
      const expected = reference.fixtures[index],
        codec = checkSetupCustody(actual, expected);
      const native = await executeExternalSetup(
        { code: actual.code },
        expected.bindings,
        expected.updates,
        true,
        expected.handler ? ["value"] : [],
      );
      assert.deepEqual(
        native.map((run) => run.initial.html),
        expected.inlineRuntime.map((run: any) => run.html),
      );
      for (const [context, run] of native.entries())
        for (const [phase, outcome] of Object.entries(run) as any[]) {
          const official = expected.externalRuntime[context][phase];
          assert.deepEqual({ ...outcome, modules: official.modules }, official);
          assert.deepEqual(outcome.modules, []);
        }
      if (actual.name === "root-let-js") {
        assert.equal(actual.code.split("$setup.value").length, 2);
        await assert.rejects(
          executeExternalSetup(
            { code: actual.code.replace("$setup.value", "_ctx.value") },
            expected.bindings,
            expected.updates,
            true,
          ),
          /SSR setup _ctx fallback value/,
        );
        await assert.rejects(
          executeExternalSetup(
            { code: removeSetupMarker(actual.code) },
            expected.bindings,
            expected.updates,
            true,
          ),
          /__isScriptSetup|undefined/,
        );
        assert.equal(actual.code.split(actual.setup.rawProgram).length, 2);
        await assert.rejects(
          executeExternalSetup(
            { code: actual.code.replace(actual.setup.rawProgram, "") },
            expected.bindings,
            expected.updates,
            true,
          ),
          /is not defined/,
        );
        const partial = structuredClone(actual),
          decoded = codec.decode(partial.mapValue.mappings);
        const line = decoded.find((segments: any[]) => segments.length > 1);
        assert(line);
        line.splice(1, 1);
        partial.mapValue.mappings = codec.encode(decoded);
        assert.throws(
          () => checkMap({ ...partial, map: partial.mapValue }),
          /complete start anchor/,
        );
        const endOnly = structuredClone(actual),
          link = endOnly.links.find(
            (link: any) =>
              link.segment &&
              link.generated.start < link.generated.end &&
              link.authored.start < link.authored.end,
          );
        assert(link);
        link.generated.start = link.generated.end;
        link.authored.start = link.authored.end;
        assert.throws(
          () => checkMap({ ...endOnly, map: endOnly.mapValue }),
          /complete start anchor/,
        );
        const invalid = structuredClone(actual),
          crab = actual.source.indexOf("🦀");
        assert(crab >= 0 && invalid.links.length > 0);
        invalid.links[0].authored.end = Buffer.byteLength(actual.source.slice(0, crab)) + 3;
        assert.throws(() => checkMap({ ...invalid, map: invalid.mapValue }), /UTF-8 boundaries/);
      }
      if (actual.name === "root-const-number")
        await assert.rejects(
          executeExternalSetup(
            { code: insertConstSetter(actual.code, "value") },
            expected.bindings,
            {},
            true,
          ),
          /undefined/,
        );
      if (actual.name === "root-ts-let") {
        assert.equal(actual.code.split("let value").length, 2);
        await assert.rejects(
          executeExternalSetup(
            { code: actual.code.replace("let value", "let value:string") },
            expected.bindings,
            expected.updates,
            true,
          ),
          (error: any) => error instanceof SyntaxError,
        );
      }
      if (expected.handler)
        await assert.rejects(
          executeExternalSetup(
            { code: insertSetupRead(actual.code, "value") },
            expected.bindings,
            expected.updates,
            true,
            ["value"],
          ),
          /SSR evaluated ignored handler binding value/,
        );
      runtime.push({
        name: actual.name,
        sourceSha256: hash(actual.source),
        codeSha256: hash(actual.code),
        mapSha256: hash(actual.mapText),
        mapValueSha256: hash(JSON.stringify(actual.mapValue)),
        linksSha256: hash(JSON.stringify(actual.links)),
        setupSha256: hash(JSON.stringify(actual.setup)),
        inlineSha256: expected.inlineSha256,
        externalSha256: expected.externalSha256,
        native,
        officialInline: expected.inlineRuntime,
        officialExternal: expected.externalRuntime,
      });
    }
    const executions = runtime.reduce(
      (sum, row) =>
        sum + row.native.reduce((count: number, run: any) => count + Object.keys(run).length, 0),
      0,
    );
    const output = {
      schema: "vize.native-sfc.setup-ssr-runtime",
      version: 1,
      modules: runtime.length,
      refusals: capture.refusals,
      nativeWholeExecutions: executions,
      nativeDirectExecutions: executions,
      officialExternalWholeExecutions: executions,
      officialInlineWholeExecutions: runtime.length * 3,
      completeUpstreamMapParity: false,
      viteContextParity: false,
      runtime,
    };
    assert.deepEqual(output, golden.runtime);
    if (process.env.VIZE_NATIVE_SETUP_SSR_RUNTIME_CAPTURE)
      fs.writeFileSync(
        process.env.VIZE_NATIVE_SETUP_SSR_RUNTIME_CAPTURE,
        JSON.stringify(output, null, 2) + "\n",
      );
  },
);
