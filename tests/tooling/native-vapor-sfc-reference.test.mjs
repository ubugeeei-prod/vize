import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import { test } from "node:test";
import { captureVaporProcess } from "./support/native-vapor-process-capture.mjs";
import {
  compileVaporSfcReference,
  compilerVersion,
  pluginVersion,
} from "./support/native-vapor-sfc-oracle.mjs";
import { officialCompilerVapor } from "./support/vue-vapor-release.mjs";

const pack = JSON.parse(
  readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native_vapor_sfc_vue_3_6_rc9.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const required = process.env.VIZE_NATIVE_VAPOR_REQUIRE_CAPTURE === "1";
assert.ok(
  !required || process.env.VIZE_NATIVE_VAPOR_SFC_CAPTURE,
  "mandatory genuine whole-SFC Rust capture path",
);
const captures = process.env.VIZE_NATIVE_VAPOR_SFC_CAPTURE
  ? JSON.parse(readFileSync(process.env.VIZE_NATIVE_VAPOR_SFC_CAPTURE, "utf8"))
  : null;
if (required) assert.equal(captures?.length, 11, "mandatory genuine whole-SFC denominator");
const ui = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const vue = createRequire(ui.resolve("vue-vapor-runtime/package.json"));
const { SourceMapConsumer } = createRequire(vue.resolve("@vue/compiler-sfc"))("source-map-js");
const position = (text, bytes) => {
  const lines = Buffer.from(text)
    .subarray(0, bytes)
    .toString("utf8")
    .split(/\r\n|[\r\n\u2028\u2029]/u);
  return { line: lines.length, column: lines.at(-1).length };
};

await test("eleven whole stock component modules/maps/helpers and native independent maps remain complete", async () => {
  assert.equal(pack.schema, "vize.native-vapor.scriptless-sfc");
  assert.equal(pack.version, compilerVersion);
  assert.equal(pack.pluginVersion, pluginVersion);
  assert.equal(pack.fixtures.length, 11);
  for (const fixture of [...pack.fixtures, ...pack.refusedReferences]) {
    assert.deepEqual(
      JSON.parse(JSON.stringify(await compileVaporSfcReference(fixture.source))),
      fixture.reference,
      fixture.id,
    );
    if (fixture.reference.multiRoot) {
      assert.match(fixture.reference.serverHtml, /^<!--\[-->[\s\S]*<!--\]-->$/u);
      continue;
    }
    const consumer = new SourceMapConsumer(fixture.map);
    const links = [];
    consumer.eachMapping((mapping) => links.push(mapping));
    assert.equal(links.length, fixture.anchors.length, fixture.id);
    for (const anchor of fixture.anchors)
      assert.deepEqual(
        consumer.originalPositionFor(position(fixture.code, anchor.generated)),
        { source: "NativeVapor.vue", ...position(fixture.source, anchor.source), name: null },
        fixture.id,
      );
    assert.deepEqual(fixture.map.sourcesContent, [fixture.source]);
    assert.ok(fixture.code.endsWith("_sfc_main.render = render\nexport default _sfc_main\n"));
  }
});

await test(
  "genuine captured whole-SFC modules/maps equal complete independent frozen outputs",
  { skip: !captures },
  () => {
    assert.equal(captures.length, pack.fixtures.length);
    for (const [index, fixture] of pack.fixtures.entries()) {
      const actual = captures[index];
      assert.equal(actual.id, fixture.id);
      assert.equal(actual.source, fixture.source);
      assert.equal(actual.code, fixture.code);
      assert.deepEqual(actual.map, fixture.map);
      assert.equal(actual.roots > 1, fixture.reference.multiRoot);
      assert.ok(Number.isInteger(actual.nodes));
    }
  },
);

function execute(inputs, expectedFailure = false) {
  return captureVaporProcess(
    "whole-component",
    inputs,
    spawnSync(
      process.execPath,
      [new URL("./support/native-vapor-runtime.mjs", import.meta.url).pathname],
      {
        input: JSON.stringify(inputs),
        encoding: "utf8",
        timeout: 60_000,
        maxBuffer: 8 * 1024 * 1024,
      },
    ),
    expectedFailure,
  );
}

await test("actual default components mount, inherit attrs, clone, hydrate stock SSR HTML and unmount", async () => {
  const inputs = [];
  for (const [index, fixture] of pack.fixtures.entries()) {
    const roots = officialCompilerVapor
      .parse(fixture.template)
      .children.filter((node) => node.type !== 3);
    const rootElement = roots.length === 1 && roots[0].type === 1;
    const configurations = [{ inheritAttrs: true, props: {} }];
    if (rootElement)
      configurations.push(
        { inheritAttrs: true, props: { title: "inherited", "data-owner": "caller" } },
        { inheritAttrs: false, props: { title: "inherited", "data-owner": "caller" } },
      );
    const serverHtmlByConfiguration = [];
    for (const configuration of configurations)
      serverHtmlByConfiguration.push(
        (await compileVaporSfcReference(fixture.source, undefined, configuration)).serverHtml,
      );
    inputs.push({
      id: fixture.id,
      component: true,
      code: captures?.[index].code ?? fixture.code,
      upstreamCode: fixture.reference.code,
      helperCode: fixture.reference.helperCode,
      multiRoot: fixture.reference.multiRoot,
      rootElement,
      hydrate: fixture.id !== "empty",
      configurations,
      serverHtmlByConfiguration,
    });
  }
  const runtime = execute(inputs);
  assert.equal(runtime.status, 0, runtime.stderr);
  const traces = JSON.parse(runtime.stdout);
  assert.equal(traces.length, 11);
  for (const trace of traces)
    assert.ok(
      trace.traces.length > 0 &&
        trace.traces.every((entry) => entry.hydrated === (trace.id !== "empty")),
    );
  if (process.env.VIZE_NATIVE_VAPOR_SFC_RUNTIME_CAPTURE)
    writeFileSync(
      process.env.VIZE_NATIVE_VAPOR_SFC_RUNTIME_CAPTURE,
      JSON.stringify(
        { capturedFromRust: !!captures, version: compilerVersion, pluginVersion, traces },
        null,
        2,
      ),
    );
});

await test("whole-component judge rejects changed source output and missing render installation", () => {
  const fixture = pack.fixtures.find((entry) => entry.id === "text");
  for (const code of [
    fixture.code.replace('"hello"', '"changed"'),
    fixture.code.replace("_sfc_main.render = render", "/* missing render */"),
  ]) {
    const runtime = execute(
      [
        {
          id: "negative-component",
          component: true,
          code,
          upstreamCode: fixture.reference.code,
          helperCode: fixture.reference.helperCode,
          multiRoot: fixture.reference.multiRoot,
          hydrate: true,
          serverHtml: fixture.reference.serverHtml,
        },
      ],
      true,
    );
    assert.notEqual(runtime.status, 0, "actual whole component changes must fail");
    assert.match(runtime.stderr, /AssertionError/u);
  }
});

await test("separate real primary lifecycle controls retain exact upstream fragment residue", async () => {
  const inputs = [];
  for (const fixture of [...pack.fixtures, ...pack.refusedReferences]) {
    const roots = officialCompilerVapor
      .parse(fixture.template)
      .children.filter((node) => node.type !== 3);
    const configurations = [{ inheritAttrs: true, props: {} }];
    if (roots.length === 1 && roots[0].type === 1)
      configurations.push(
        { inheritAttrs: true, props: { title: "inherited", "data-owner": "caller" } },
        { inheritAttrs: false, props: { title: "inherited", "data-owner": "caller" } },
      );
    const serverHtmlByConfiguration = [];
    for (const configuration of configurations)
      serverHtmlByConfiguration.push(
        (await compileVaporSfcReference(fixture.source, undefined, configuration)).serverHtml,
      );
    inputs.push({
      id: fixture.id,
      reference: fixture.reference,
      configurations,
      serverHtmlByConfiguration,
    });
  }
  const primary = spawnSync(
    process.execPath,
    [new URL("./support/native-vapor-primary-lifecycle.mjs", import.meta.url).pathname],
    {
      input: JSON.stringify(inputs),
      encoding: "utf8",
      timeout: 60_000,
      maxBuffer: 8 * 1024 * 1024,
    },
  );
  captureVaporProcess("primary-only-lifecycle", inputs, primary);
  assert.equal(primary.status, 0, primary.stderr);
  const observed = JSON.parse(primary.stdout);
  assert.equal(observed.primaryOnly, true);
  assert.equal(observed.traces.length, 13);
  assert.equal(observed.counts.configurations, 27);
  assert.equal(observed.counts.cleanHydrated, 22);
  assert.equal(observed.counts.fragmentedHydrationWithRetainedMarkers, 4);
  assert.equal(observed.counts.mountOnly, 1);
  if (process.env.VIZE_NATIVE_VAPOR_PRIMARY_LIFECYCLE_CAPTURE)
    writeFileSync(
      process.env.VIZE_NATIVE_VAPOR_PRIMARY_LIFECYCLE_CAPTURE,
      JSON.stringify(observed, null, 2),
    );
});
