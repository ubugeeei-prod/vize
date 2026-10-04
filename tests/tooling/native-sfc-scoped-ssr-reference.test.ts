import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  captureHashes,
  checkVersions,
  hash,
  loadModule,
  relink,
  render,
  stock,
} from "./support/native-scoped-ssr-reference.ts";
import {
  checkCssBytes,
  checkMap,
  codec,
  coordinate,
  outsideStyleWindow,
} from "./support/native-scoped-ssr-maps.ts";
import { hydrate } from "./support/native-scoped-ssr-browser.ts";

const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native-scoped-ssr-vue-3.5.35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const capturePath = process.env.VIZE_NATIVE_SCOPED_SSR_CAPTURE;
// A promised protected capture must exist. Fixtures never supply native credit.
const captured = capturePath ? JSON.parse(fs.readFileSync(capturePath, "utf8")) : null;
const skipNative = !capturePath && process.env.VIZE_NATIVE_SCOPED_SSR_REQUIRE_NATIVE !== "1";
const whole = ({
  id,
  source,
  filename,
  code,
  css,
  scopeId,
  map,
  cssMap,
  mapText,
  cssMapText,
}: any) => ({
  id,
  source,
  filename,
  code,
  css,
  scopeId,
  map,
  cssMap,
  mapText,
  cssMapText,
});

test("the CSS judge refuses coherent equal-byte origins outside the genuine style body", () => {
  const fixture = pack.fixtures[1];
  const oracle = stock(fixture);
  const insertion = oracle.css.code.indexOf(`[${fixture.scopeId}]`);
  assert(insertion > 0);
  const prefix = Buffer.byteLength(oracle.css.code.slice(0, insertion));
  const inserted = Buffer.byteLength(`[${fixture.scopeId}]`);
  const cssLinks = [
    {
      generated: { start: 0, end: prefix },
      authored: { start: oracle.cssSpan.start, end: oracle.cssSpan.start + prefix },
      segment: true,
      name: null,
    },
    {
      generated: { start: prefix + inserted, end: Buffer.byteLength(oracle.css.code) },
      authored: { start: oracle.cssSpan.start + prefix, end: oracle.cssSpan.end },
      segment: true,
      name: null,
    },
  ];
  const lines: number[][][] = oracle.css.code.split(/\r\n|[\r\n\u2028\u2029]/).map(() => []);
  for (const link of cssLinks) {
    const [line, column] = coordinate(oracle.css.code, link.generated.start);
    lines[line].push([column, 0, ...coordinate(fixture.source, link.authored.start)]);
  }
  const actual = {
    source: fixture.source,
    filename: fixture.filename,
    css: oracle.css.code,
    scopeId: fixture.scopeId,
    cssLinks,
    cssMap: {
      version: 3,
      file: fixture.filename,
      sources: [fixture.filename],
      sourcesContent: [fixture.source],
      names: [],
      mappings: codec.encode(lines),
    },
  };
  checkMap(actual.source, actual.filename, actual.css, actual.cssMap, actual.cssLinks);
  checkCssBytes(actual, oracle.originalCss, oracle.cssSpan);
  const forged = outsideStyleWindow(actual, oracle.originalCss, oracle.cssSpan);
  const foreignOracle = stock({ ...fixture, source: forged.source });
  assert.deepEqual(foreignOracle.cssSpan, oracle.cssSpan);
  checkMap(forged.source, forged.filename, forged.css, forged.cssMap, forged.cssLinks);
  assert.throws(
    () => checkCssBytes(forged, foreignOracle.originalCss, foreignOracle.cssSpan),
    /exact original style window/,
  );
});

test("dev import relinking changes only real top-level source spans", () => {
  const code = `import { renderToString } from "@vue/server-renderer";\n/* from "@vue/server-renderer" */ let label='from "vue"';`;
  assert.equal(
    relink(code, { "@vue/server-renderer": "data:actual-runtime" }),
    `import { renderToString } from "data:actual-runtime";\n/* from "@vue/server-renderer" */ let label='from "vue"';`,
  );
});

test("three whole no-authored-class SFCs retain complete pinned CSS/SSR/client outputs and maps", async () => {
  checkVersions();
  assert.equal(pack.schema, "vize.native-sfc.scoped-ssr-reference");
  assert.equal(pack.version, 1);
  assert.equal(pack.runtime, "vue@3.5.35");
  assert.equal(pack.fixtures.length, 3);
  assert.equal(new Set(pack.fixtures.map((row: any) => row.id)).size, 3);
  for (const fixture of pack.fixtures) {
    assert(!fixture.template.includes("class="));
    const actual = stock(fixture);
    assert.equal(actual.css.code, fixture.css);
    assert.deepEqual(actual.css.map, fixture.stockCssMap);
    assert.equal(actual.ssr.code, fixture.stockSsrModule);
    assert.deepEqual(actual.ssr.map, fixture.stockSsrMap);
    assert.equal(actual.client.code, fixture.stockClientModule);
    assert.deepEqual(actual.client.map, fixture.stockClientMap);
    for (const output of [actual.css, actual.ssr, actual.client])
      assert.deepEqual(output.map.sourcesContent, [fixture.source]);
    const positive = await loadModule(actual.ssr.code);
    const negative = await loadModule(actual.metadataOnly.code);
    assert.equal(
      await render({ ssrRender: positive.ssrRender, __scopeId: fixture.scopeId }, fixture.props),
      fixture.html,
    );
    assert.equal(
      await render({ ssrRender: negative.ssrRender, __scopeId: fixture.scopeId }, fixture.props),
      fixture.metadataOnlyHtml,
    );
    assert.notEqual(fixture.metadataOnlyHtml, fixture.html);
    assert(!fixture.metadataOnlyHtml.includes(fixture.scopeId));
  }
});

test(
  "fresh whole native scoped SSR modules and every original map anchor execute against pinned Vue",
  { skip: skipNative },
  async () => {
    assert(captured, "protected styled SSR requires fresh Rust source-built captures");
    assert.deepEqual(captured.map(whole), pack.fixtures.map(whole));
    const rendered = [];
    for (const [index, fixture] of pack.fixtures.entries()) {
      const actual = captured[index];
      assert.deepEqual(JSON.parse(actual.mapText), actual.map);
      assert.deepEqual(JSON.parse(actual.cssMapText), actual.cssMap);
      checkMap(actual.source, actual.filename, actual.code, actual.map, actual.links);
      checkMap(actual.source, actual.filename, actual.css, actual.cssMap, actual.cssLinks);
      const oracle = stock(fixture);
      checkCssBytes(actual, oracle.originalCss, oracle.cssSpan);
      const foreign = outsideStyleWindow(actual, oracle.originalCss, oracle.cssSpan);
      checkMap(foreign.source, foreign.filename, foreign.css, foreign.cssMap, foreign.cssLinks);
      assert.throws(
        () => checkCssBytes(foreign, oracle.originalCss, oracle.cssSpan),
        /exact original style window/,
      );
      const loaded = await loadModule(actual.code);
      assert.equal(loaded.default.__scopeId, actual.scopeId);
      assert.equal(typeof loaded.default.ssrRender, "function");
      const html = await render(loaded.default, fixture.props);
      assert.equal(html, fixture.html);
      rendered.push({ ...actual, html, hashes: captureHashes(actual), htmlSha256: hash(html) });
      for (const found of actual.code.matchAll(new RegExp(actual.scopeId, "g"))) {
        const start = Buffer.byteLength(actual.code.slice(0, found.index));
        const end = start + Buffer.byteLength(actual.scopeId);
        assert(
          actual.links.every(
            (link: any) => link.generated.end <= start || link.generated.start >= end,
          ),
          "generated scope attributes/metadata must remain unlinked",
        );
      }
    }
    const broken = structuredClone(captured[1]);
    const decoded = codec.decode(broken.cssMap.mappings);
    const anchors = decoded.find((line: number[][]) => line.length > 0);
    assert(anchors);
    anchors.splice(0, 1);
    broken.cssMap.mappings = codec.encode(decoded);
    assert.throws(
      () => checkMap(broken.source, broken.filename, broken.css, broken.cssMap, broken.cssLinks),
      /complete UTF-16 start anchor/,
    );
    const invalidBytes = structuredClone(captured[1]);
    invalidBytes.cssLinks[0].authored.start =
      Buffer.byteLength(invalidBytes.source.slice(0, invalidBytes.source.indexOf("🌸"))) + 2;
    assert.throws(
      () =>
        checkMap(
          invalidBytes.source,
          invalidBytes.filename,
          invalidBytes.css,
          invalidBytes.cssMap,
          invalidBytes.cssLinks,
        ),
      /UTF-8 boundaries/,
    );
    const runtime = await hydrate(pack.fixtures, rendered);
    if (process.env.VIZE_NATIVE_SCOPED_SSR_RUNTIME_CAPTURE)
      fs.writeFileSync(
        process.env.VIZE_NATIVE_SCOPED_SSR_RUNTIME_CAPTURE,
        JSON.stringify(
          {
            ...runtime,
            rendered: rendered.map((row) => ({
              id: row.id,
              scopeId: row.scopeId,
              html: row.html,
              hashes: row.hashes,
              htmlSha256: row.htmlSha256,
            })),
          },
          null,
          2,
        ) + "\n",
      );
  },
);
