import assert from "node:assert/strict";
import fs from "node:fs";
import { after, test } from "node:test";
import {
  bodyFacts,
  compiler,
  decode,
  executeLocalComponent,
  fromVue,
  hash,
  mapAnchors,
  position,
} from "./support/native-handler-local-sfc-dom-runtime.ts";

const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native_handler_local_sfc_click_vue_3_5_35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const capturePath = process.env.VIZE_NATIVE_HANDLER_LOCAL_SFC_DOM_CAPTURE;
const captured = capturePath ? JSON.parse(fs.readFileSync(capturePath, "utf8")) : null;
const required = process.env.VIZE_NATIVE_HANDLER_LOCAL_SFC_DOM_REQUIRE_CAPTURE === "1";
const runtimePath = process.env.VIZE_NATIVE_HANDLER_LOCAL_SFC_DOM_RUNTIME_CAPTURE;
const executions: unknown[] = [];

function checkMap(fixture: any, code: string, map: any) {
  assert(map, `${fixture.id}: map must be frozen from genuine hosted Rust output`);
  assert.equal(map.version, 3);
  assert.equal(map.file, pack.filename);
  assert.deepEqual(map.sources, [pack.filename]);
  assert.deepEqual(map.sourcesContent, [fixture.source]);
  const points = mapAnchors(map, code, fixture.source);
  const names = [
    ...new Set(
      fixture.bodyFacts.flatMap((facts: any) =>
        facts.references.map((reference: any) => reference.name),
      ),
    ),
  ];
  assert.deepEqual(map.names, names);
  let generatedCursor = 0;
  const windows = [...fixture.source.matchAll(/@click="([^"]*)"/g)];
  assert.deepEqual(
    windows.map((window: any) => window[1]),
    fixture.bodies,
  );
  for (let index = 0; index < windows.length; index++) {
    const raw = windows[index][1],
      facts = bodyFacts(raw),
      decoded = facts.decoded;
    const authoredStart = windows[index].index! + '@click="'.length;
    const marker = "onClick: $event => {";
    const start = code.indexOf(marker, generatedCursor);
    assert(start >= 0);
    const generatedStart = start + marker.length;
    generatedCursor = generatedStart + decoded.length;
    assert.equal(code.slice(generatedStart, generatedCursor), decoded);
    assert.equal(code[generatedCursor], "}");
    const entities: { raw: number; decoded: number }[] = [];
    for (const match of raw.matchAll(/&quot;/g)) {
      entities.push({ raw: match.index, decoded: decode(raw.slice(0, match.index)).length });
    }
    const authoredOffset = (offset: number) =>
      offset + entities.filter((entity) => entity.decoded < offset).length * ("&quot;".length - 1);
    const cuts = [
      ...new Set([
        0,
        decoded.length,
        ...facts.references.flatMap((reference: any) => reference.span),
        ...entities.flatMap((entity) => [entity.decoded, entity.decoded + 1]),
      ]),
    ].sort((a, b) => a - b);
    const expected = [];
    let recoveredRaw = "",
      recoveredDecoded = "";
    for (let piece = 0; piece < cuts.length - 1; piece++) {
      const begin = cuts[piece],
        end = cuts[piece + 1];
      const original = raw.slice(authoredOffset(begin), authoredOffset(end));
      const text = decoded.slice(begin, end);
      assert.equal(decode(original), text);
      recoveredRaw += original;
      recoveredDecoded += text;
      const reference = facts.references.find(
        (reference: any) => reference.span[0] === begin && reference.span[1] === end,
      );
      expected.push({
        generated: position(code, generatedStart + begin),
        authored: position(fixture.source, authoredStart + authoredOffset(begin)),
        name: reference?.name ?? null,
      });
    }
    assert.equal(recoveredRaw, raw);
    assert.equal(recoveredDecoded, decoded);
    const [line, column] = position(code, generatedStart);
    assert.deepEqual(
      points.filter(
        (point: any) =>
          point.generated[0] === line &&
          point.generated[1] >= column &&
          point.generated[1] < column + decoded.length,
      ),
      expected,
      `${fixture.id}: every original local/event/entity body anchor`,
    );
  }
}

function freshReference(fixture: any) {
  assert.equal(
    fixture.source,
    `<!--Local雪🌸:${fixture.id}-->\r\n<template>${fixture.template}</template>`,
  );
  assert.equal(
    fixture.template,
    fixture.bodies
      .map((body: string, index: number) => `<button id="b${index}" @click="${body}"/>`)
      .join(""),
  );
  const parsed = compiler.parse(fixture.source, { filename: pack.filename });
  assert.deepEqual(parsed.errors, []);
  const descriptor = parsed.descriptor;
  assert.equal(descriptor.source, fixture.source);
  assert.equal(descriptor.script, null);
  assert.equal(descriptor.scriptSetup, null);
  assert.deepEqual(descriptor.styles, []);
  assert.deepEqual(descriptor.customBlocks, []);
  assert.equal(descriptor.template.content, fixture.template);
  assert.equal(
    fixture.source.slice(descriptor.template.loc.start.offset, descriptor.template.loc.end.offset),
    fixture.template,
  );
  const measured = compiler.compileTemplate({
    source: descriptor.template.content,
    filename: pack.filename,
    id: fixture.id,
    sourceMap: true,
    compilerOptions: pack.options,
  });
  assert.deepEqual(measured.errors, []);
  assert.equal(measured.code, fixture.referenceRender);
  assert.deepEqual(measured.map, fixture.referenceMap);
  assert.equal(
    fixture.referenceModule,
    measured.code +
      "\nconst reference_component = { render };\nexport default reference_component;\n",
  );
  assert.deepEqual(fixture.bodyFacts, fixture.bodies.map(bodyFacts));
  assert.equal(fixture.events.length, 2);
  assert.equal(fixture.expected.length, 2);
  for (const facts of fixture.bodyFacts) {
    assert(facts.wrappedUnits <= 31);
    assert(facts.utf16 <= 31);
  }
  return measured;
}

test("the separate local packet pins complete original SFCs and real block/declaration/reference roots", () => {
  assert.equal(pack.schema, "vize.native-sfc.handler-local-click-reference");
  assert.equal(pack.version, 1);
  assert.deepEqual(pack.compiler, { name: "@vue/compiler-sfc", version: "3.5.35" });
  assert.deepEqual(pack.runtime, { name: "vue", version: "3.5.35" });
  assert.equal(compiler.version, "3.5.35");
  assert.equal(fromVue("vue/package.json").version, "3.5.35");
  assert.deepEqual(pack.options, {
    mode: "module",
    hoistStatic: false,
    prefixIdentifiers: true,
    comments: true,
    bindingMetadata: {},
    cacheHandlers: false,
  });
  assert.deepEqual(
    pack.fixtures.map((row: any) => row.id),
    [
      "const-block",
      "let-block",
      "var-block",
      "forward-tdz",
      "block-shadow",
      "hoisted-var",
      "repeated-var",
      "ancestor-block",
      "unicode-entity",
      "repeated-handlers",
    ],
  );
  assert.deepEqual(
    pack.divergences.map((row: any) => row.id),
    ["root-local", "root-to-block", "block-var-to-root", "sibling-var", "sibling-let"],
  );
  assert.equal(new Set([...pack.fixtures, ...pack.divergences].map((row: any) => row.id)).size, 15);
  for (const fixture of pack.fixtures) {
    const measured = freshReference(fixture);
    for (const facts of fixture.bodyFacts)
      assert(measured.code.includes("onClick: $event => {" + facts.decoded + "}"));
    const [imports, render] = measured.code.split("\n\nexport function render");
    assert.equal(
      fixture.expectedCode,
      imports +
        "\nconst _sfc_main = {}\n;\nfunction render" +
        render +
        "\n_sfc_main.render = render\nexport default _sfc_main\n",
    );
  }
  for (const fixture of pack.divergences) {
    assert.match(freshReference(fixture).code, /_ctx\.x/);
    assert.equal(
      fixture.expectedCode,
      undefined,
      "divergence rows never authorize native assembly",
    );
  }
});

for (const fixture of [...pack.fixtures, ...pack.divergences]) {
  test(`${fixture.id}: independent pinned complete component preserves declared effects, TDZ and cleanup`, async () => {
    const reference = await executeLocalComponent(fixture.referenceModule, fixture);
    assert.equal(reference.replacedCallbacks, fixture.bodies.length);
  });
}

test("hosted local acceptance requires genuine complete source/code/map capture and frozen equality", () => {
  if (required) assert(captured, "actual hosted Rust local capture is mandatory");
  for (const fixture of pack.fixtures) checkMap(fixture, fixture.expectedCode, fixture.nativeMap);
  if (captured) {
    assert.equal(captured.schema, "vize.native-sfc.handler-local-click-capture");
    assert.equal(captured.adapter, "vize_atelier_sfc::compile_native_selected_sfc_dom");
    assert.deepEqual(
      captured.fixtures,
      pack.fixtures.map((fixture: any) => ({
        id: fixture.id,
        source: fixture.source,
        code: fixture.expectedCode,
        nativeMap: fixture.nativeMap,
      })),
    );
  }
});

for (const fixture of pack.fixtures) {
  test(`${fixture.id}: actual frozen native complete component matches the untouched pinned runtime`, async () => {
    if (required) assert(captured, "source-built whole local components are mandatory");
    const actual = captured?.fixtures.find((row: any) => row.id === fixture.id);
    if (captured)
      assert.deepEqual(actual, {
        id: fixture.id,
        source: fixture.source,
        code: fixture.expectedCode,
        nativeMap: fixture.nativeMap,
      });
    const code = actual?.code ?? fixture.expectedCode,
      nativeMap = actual?.nativeMap ?? fixture.nativeMap;
    checkMap(fixture, code, nativeMap);
    const reference = await executeLocalComponent(fixture.referenceModule, fixture);
    const native = await executeLocalComponent(code, fixture);
    assert.deepEqual(native, reference);
    executions.push({
      id: fixture.id,
      sourceHash: hash(fixture.source),
      codeHash: hash(code),
      nativeMapHash: hash(JSON.stringify(nativeMap)),
      referenceCodeHash: hash(fixture.referenceModule),
      origin: captured ? "current-source-rust-capture" : "frozen-rust-fixture",
      native,
      reference,
    });
  });
}
after(() => {
  if (!runtimePath) return;
  assert(required && captured);
  assert.equal(executions.length, pack.fixtures.length);
  fs.writeFileSync(
    runtimePath,
    JSON.stringify(
      {
        schema: "vize.native-sfc.handler-local-click-runtime-capture",
        adapter: "vize_atelier_sfc::compile_native_selected_sfc_dom",
        runtime: pack.runtime,
        host: "vue-createRenderer-custom-host",
        sourceCaptureHash: hash(fs.readFileSync(capturePath!, "utf8")),
        fixtures: executions,
      },
      null,
      2,
    ) + "\n",
  );
});
