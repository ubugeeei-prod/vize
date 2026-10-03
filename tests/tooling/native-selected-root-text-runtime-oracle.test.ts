import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

type Span = [number, number];
type Slot = { ordinal: number; kind: string; raw: string; span: Span };
type Window = { ordinal: number; raw: string; span: Span; content: string | null };
type MapValue = {
  version: number;
  file?: string;
  sources: string[];
  sourcesContent: string[];
  names: string[];
  mappings: string;
};
type Link = { generated: Span; authored: Span; name: string | null; segment: boolean };
type Native = { code: string; map: MapValue; links: Link[]; nodeCount: number };
type Fixture = {
  id: string;
  source: string;
  template: string;
  templateSpan: Span;
  rootSlots: Slot[];
  rootText: Window[];
  referenceCode: string;
  referenceMap: MapValue;
  referenceRuntime: unknown;
  native?: Native;
};
const pack = JSON.parse(
  readFileSync(
    new URL(
      "../../davinci/vize_l4/tests/fixtures/native-selected-root-text-vue-3.5.35.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as {
  schema: string;
  version: number;
  vueVersion: string;
  options: Record<string, unknown> & { filename: string };
  nativeCapture: unknown;
  cases: Fixture[];
  deferred: Array<{ id: string }>;
};
const provider = JSON.parse(
  readFileSync(
    new URL(
      "../../davinci/vize_l1/tests/fixtures/native-root-text-vue-3.5.35.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as { cases: Array<Fixture & { nativeEligible: boolean }> };
const fromUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const compiler = fromVue("@vue/compiler-dom") as {
  compile(source: string, options: Record<string, unknown>): { code: string; map: MapValue };
};
const runtime = fromVue("vue");
const dataUrl = (code: string) =>
  `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
const runtimeUrl = dataUrl(
  `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve("vue")).href)};\n` +
    [
      "openBlock",
      "createElementBlock",
      "createElementVNode",
      "createTextVNode",
      "createCommentVNode",
      "Fragment",
    ]
      .map((name) => `export const ${name} = runtime.${name};`)
      .join("\n"),
);
const symbols = new Map(
  ["Fragment", "Text", "Comment", "Static"].map((name) => [runtime[name], name]),
);
const canonical = (value: unknown): unknown =>
  JSON.parse(
    JSON.stringify(value, (_key, item) => {
      if (typeof item === "symbol") {
        assert(symbols.has(item), "known pinned Vue runtime symbol");
        return { runtimeSymbol: symbols.get(item) };
      }
      assert.notEqual(typeof item, "function", "complete static runtime value");
      return item;
    }),
  );

test("packet identity retains the bounded original provider closure and official version", () => {
  assert.equal(fromVue("@vue/compiler-dom/package.json").version, "3.5.35");
  assert.equal(fromVue("vue/package.json").version, "3.5.35");
  assert.equal(pack.schema, "vize.native-selected-root-text-reference");
  assert.equal(pack.version, 1);
  assert.equal(pack.vueVersion, "3.5.35");
  assert.deepEqual(pack.nativeCapture, {
    sourceHead: "afd0fbd88fc9d5878cae90eec3cc6c2bf74621de",
    executionCommit: "f57f6f7bc2f4d88ede347b0f53d9c459b2ff97d3",
    workflowRun: 37128761580,
    rustJob: 111220362717,
    artifactId: 11276012428,
    artifactName: "rust-test-shard-4-37128761580-1",
    artifactPath: "native-selected-root-text-dom.json",
    cases: 24,
  });
  assert.deepEqual(pack.options, {
    mode: "module",
    hoistStatic: false,
    prefixIdentifiers: true,
    comments: true,
    whitespace: "condense",
    filename: "SelectedRootText雪🌸.vue",
    sourceMap: true,
    bindingMetadata: {},
    cacheHandlers: false,
  });
  const eligible = provider.cases.filter(
    (fixture) =>
      fixture.nativeEligible && !fixture.rootSlots.some(({ kind }) => kind === "interpolation"),
  );
  assert.equal(pack.cases.length, 24);
  assert.equal(new Set(pack.cases.map(({ id }) => id)).size, 24);
  assert.deepEqual(
    pack.cases.map(({ id }) => id),
    eligible.map(({ id }) => id),
  );
  assert.deepEqual(
    pack.deferred.map(({ id }) => id),
    provider.cases.filter((fixture) => !eligible.includes(fixture)).map(({ id }) => id),
  );
  for (const [index, fixture] of pack.cases.entries()) {
    const original = eligible[index];
    for (const field of ["source", "template", "templateSpan", "rootSlots", "rootText"] as const) {
      assert.deepEqual(fixture[field], original[field], `${fixture.id}: original ${field}`);
    }
  }
});

function native(fixture: Fixture): Native {
  assert(fixture.native, `${fixture.id}: genuine hosted Rust capture is required`);
  return fixture.native;
}

function byteSlice(text: string, span: Span): string {
  const bytes = Buffer.from(text);
  assert(span.every(Number.isSafeInteger));
  assert(span[0] >= 0 && span[0] <= span[1] && span[1] <= bytes.length);
  const selected = bytes.subarray(...span);
  const decoded = selected.toString("utf8");
  assert(Buffer.from(decoded).equals(selected), "exact UTF8 boundaries");
  return decoded;
}

function position(text: string, byte: number): [number, number] {
  // ECMA-262 §12.3: CRLF is one break; ECMA-426 columns count UTF16 units.
  // https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-line-terminators
  const lines = byteSlice(text, [0, byte]).split(/\r\n|[\n\r\u2028\u2029]/u);
  return [lines.length - 1, lines[lines.length - 1].length];
}

test("independent map positions retain every JavaScript line terminator and Unicode column", () => {
  for (const ending of ["\n", "\r\n", "\r", "\u2028", "\u2029"]) {
    const text = `雪🌸${ending}漢🌸x`;
    assert.deepEqual(position(text, Buffer.byteLength(text.slice(0, -1))), [1, 3]);
  }
  const nextLineControl = "雪🌸\u0085x";
  assert.deepEqual(position(nextLineControl, Buffer.byteLength("雪🌸\u0085")), [0, 4]);
});

function vlq(segment: string): number[] {
  const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  const fields: number[] = [];
  let value = 0;
  let shift = 0;
  for (const character of segment) {
    const digit = alphabet.indexOf(character);
    assert(digit >= 0);
    value += (digit & 31) * 2 ** shift;
    if (digit & 32) shift += 5;
    else {
      fields.push((value & 1 ? -1 : 1) * Math.floor(value / 2));
      value = 0;
      shift = 0;
    }
  }
  assert.equal(shift, 0);
  return fields;
}

function decodedAnchors(map: MapValue): unknown[] {
  let source = 0;
  let line = 0;
  let column = 0;
  let name = 0;
  const anchors: unknown[] = [];
  map.mappings.split(";").forEach((encodedLine, generatedLine) => {
    let generatedColumn = 0;
    for (const encoded of encodedLine.split(",").filter(Boolean)) {
      const fields = vlq(encoded);
      assert(fields.length === 4 || fields.length === 5);
      generatedColumn += fields[0];
      source += fields[1];
      line += fields[2];
      column += fields[3];
      assert.equal(source, 0);
      assert(generatedColumn >= 0 && line >= 0 && column >= 0);
      if (fields.length === 5) {
        name += fields[4];
        assert(name >= 0 && name < map.names.length);
      }
      anchors.push({
        generated: [generatedLine, generatedColumn],
        authored: [line, column],
        name: fields.length === 5 ? map.names[name] : null,
      });
    }
  });
  return anchors;
}

function checkMap(fixture: Fixture, captured: Native): void {
  assert.deepEqual(Object.keys(captured.map).sort(), [
    "file",
    "mappings",
    "names",
    "sources",
    "sourcesContent",
    "version",
  ]);
  assert.equal(captured.map.version, 3);
  assert.equal(captured.map.file, pack.options.filename);
  assert.deepEqual(captured.map.sources, [pack.options.filename]);
  assert.deepEqual(captured.map.sourcesContent, [fixture.source]);
  assert.deepEqual(captured.map.names, []);
  for (const link of captured.links) {
    assert.equal(typeof link.segment, "boolean");
    byteSlice(captured.code, link.generated);
    byteSlice(fixture.source, link.authored);
    assert(link.authored[0] >= fixture.templateSpan[0]);
    assert(link.authored[1] <= fixture.templateSpan[1]);
    if (link.authored[0] !== link.authored[1]) {
      const slot = fixture.rootSlots.find(
        ({ span }) => span[0] === link.authored[0] && span[1] === link.authored[1],
      );
      assert(slot, "complete genuine original root slot owns each literal link");
      const value = JSON.parse(byteSlice(captured.code, link.generated));
      const expected =
        slot.kind === "text"
          ? fixture.rootText.find(({ ordinal }) => ordinal === slot.ordinal)?.content
          : slot.kind === "comment"
            ? slot.raw.slice(4, -3)
            : /^<([a-z]+)\/>$/.exec(slot.raw)?.[1];
      assert.notEqual(expected, undefined);
      assert.deepEqual(value, expected, "original authored slot and cooked literal agree");
    }
  }
  const anchors = captured.links
    .filter(({ segment }) => segment)
    .toSorted((left, right) => left.generated[0] - right.generated[0])
    .map((link) => ({
      generated: position(captured.code, link.generated[0]),
      authored: position(fixture.source, link.authored[0]),
      name: link.name,
    }));
  assert.deepEqual(
    decodedAnchors(captured.map),
    anchors,
    "every complete map anchor matches its byte link",
  );
  for (const window of fixture.rootText) {
    const links = captured.links.filter(
      ({ authored }) => authored[0] === window.span[0] && authored[1] === window.span[1],
    );
    if (window.content === null) assert.deepEqual(links, [], "omitted text has no generated link");
    else if (window.content !== " ")
      assert.equal(links.length, 1, "each cooked text literal keeps original span");
  }
}

async function load(code: string, tag: string): Promise<{ render(...args: unknown[]): unknown }> {
  // Only the import address changes; the complete actual render body executes.
  const loaded = await import(
    dataUrl(code.replace('from "vue"', `from ${JSON.stringify(runtimeUrl)}`)) + `#${tag}`
  );
  assert.deepEqual(Object.keys(loaded), ["render"]);
  assert.equal(typeof loaded.render, "function");
  return loaded;
}

for (const fixture of pack.cases) {
  test(`${fixture.id}: reference module and complete original official map`, async () => {
    const compiled = compiler.compile(fixture.template, pack.options);
    assert.equal(compiled.code, fixture.referenceCode);
    assert.deepEqual(compiled.map, fixture.referenceMap);
    const official = await load(compiled.code, `verified-official-${fixture.id}`);
    const forbidden = new Proxy(
      {},
      {
        get(_target, key) {
          throw Error(`Unexpected read ${String(key)}`);
        },
      },
    );
    for (const context of [{}, forbidden]) {
      assert.deepEqual(
        canonical(official.render(context, [], forbidden, forbidden, forbidden, forbidden)),
        fixture.referenceRuntime,
      );
    }
  });
  test(`${fixture.id}: genuine native code and independently decoded whole source map`, () => {
    const captured = native(fixture);
    assert.equal(captured.code, fixture.referenceCode);
    const omitted = fixture.rootText.filter(({ content }) => content === null).length;
    assert.equal(captured.nodeCount, fixture.rootSlots.length - omitted);
    checkMap(fixture, captured);
  });
  test(`${fixture.id}: genuine native and reference complete raw runtime`, async () => {
    const captured = native(fixture);
    const official = await load(fixture.referenceCode, `official-${fixture.id}`);
    const actual = await load(captured.code, `native-${fixture.id}`);
    const forbidden = new Proxy(
      {},
      {
        get(_target, key) {
          throw Error(`Unexpected read ${String(key)}`);
        },
      },
    );
    for (const context of [{}, forbidden]) {
      const arguments_ = [context, [], forbidden, forbidden, forbidden, forbidden];
      const expected = official.render(...arguments_);
      const result = actual.render(...arguments_);
      assert.deepEqual(
        result,
        expected,
        "complete live return, including internal and undefined fields",
      );
      assert.deepEqual(canonical(expected), fixture.referenceRuntime);
      assert.deepEqual(canonical(result), fixture.referenceRuntime);
    }
  });
}
