const assert = require("node:assert/strict");
const { test } = require("node:test");
const ts = require("../node_modules/typescript");
const helper = require("../../npm/native/language-projection.js");
const { createMappedLanguageService } = require("../../npm/typescript-plugin/language-service.cjs");

function project(newline) {
  const file = "/src/日本語.vue";
  const prefix = `declare const generatedHelper: number;${newline}`;
  const before = `<script setup lang="ts">${newline}// 😀 日本語${newline}`;
  const script = `class Person { value = 1; }${newline}const person = new Person();${newline}person.value;${newline}`;
  let source = before + script + `</script>${newline}<template>{{ person.value }}</template>`;
  let generated = prefix + script + "generatedHelper;";
  let version = 0;
  const utf8 = (text) => Buffer.byteLength(text);
  const packet = () => ({
    text: generated,
    extension: ".tsx",
    diagnostics: [],
    mappings: [
      [
        utf8(prefix),
        utf8(script),
        utf8(before),
        utf8(script),
        0,
        helper.LanguageFeature.SemanticTokens,
      ],
    ],
  });
  let projection = helper.toUtf16LanguageProjection(source, JSON.stringify(packet()));
  const host = {
    getScriptFileNames: () => [file],
    getScriptVersion: () => String(version),
    getProjectVersion: () => String(version),
    getScriptSnapshot: () => ts.ScriptSnapshot.fromString(generated),
    getCompilationSettings: () => ({
      noLib: true,
      allowNonTsExtensions: true,
      target: ts.ScriptTarget.ESNext,
    }),
    getCurrentDirectory: () => "/src",
    getDefaultLibFileName: () => "/lib.d.ts",
    getScriptKind: () => ts.ScriptKind.TS,
  };
  const original = ts.createLanguageService(host);
  const calls = [];
  const raw = Object.create(original);
  for (const method of [
    "getEncodedSyntacticClassifications",
    "getEncodedSemanticClassifications",
  ]) {
    raw[method] = (...args) => {
      calls.push([method, ...args]);
      return original[method](...args);
    };
  }
  const support = {
    engine: helper,
    get: (name) => (name === file ? { source, projection } : undefined),
  };
  const service = createMappedLanguageService(ts, raw, support);
  return {
    file,
    service,
    original,
    calls,
    before,
    script,
    prefix,
    packet,
    get source() {
      return source;
    },
    get projection() {
      return projection;
    },
    edit() {
      source = newline + source;
      generated = newline + generated;
      version++;
      projection = {
        ...projection,
        text: generated,
        mappings: projection.mappings.map(([g, gl, s, sl, ...rest]) => [
          g + newline.length,
          gl,
          s + newline.length,
          sl,
          ...rest,
        ]),
      };
    },
    dispose: () => original.dispose(),
  };
}

for (const newline of ["\n", "\r\n"])
  for (const method of ["getEncodedSyntacticClassifications", "getEncodedSemanticClassifications"])
    void test(`real TypeScript ${method} maps the complete UTF-8 packet ${JSON.stringify(newline)}`, () => {
      const p = project(newline);
      try {
        const raw = p.original[method](
          p.file,
          { start: 0, length: p.projection.text.length },
          "2020",
        );
        assert.ok(raw.spans.length > 0, "genuine TypeScript classifications exist");
        const before = p.service[method](p.file, { start: 0, length: p.source.length }, "2020");
        assert.ok(before.spans.length > 0, "the plugin must not erase SFC classifications");
        const expected = [];
        for (let at = 0; at < raw.spans.length; at += 3) {
          const [start, length, classification] = raw.spans.slice(at, at + 3);
          if (start >= p.prefix.length && start + length <= p.prefix.length + p.script.length)
            expected.push(start + p.before.length - p.prefix.length, length, classification);
        }
        assert.deepEqual(
          before,
          { ...raw, spans: expected },
          "compare the entire classification packet",
        );
        assert.equal(p.calls.at(-1)[0], method, "use the existing TypeScript service");
        for (let at = 0; at < before.spans.length; at += 3) {
          const [start, length] = before.spans.slice(at, at + 2);
          assert.ok(
            start >= p.before.length && start + length <= p.before.length + p.script.length,
          );
          assert.ok(!p.source.slice(start, start + length).includes("generatedHelper"));
        }
        p.edit();
        const after = p.service[method](p.file, { start: 0, length: p.source.length }, "2020");
        assert.deepEqual(
          after,
          {
            ...before,
            spans: before.spans.map((value, index) =>
              index % 3 === 0 ? value + newline.length : value,
            ),
          },
          "read current projection and current TypeScript snapshots after an edit",
        );
        assert.equal(
          p.calls.length,
          2,
          "one genuine service call per eligible classification request",
        );
      } finally {
        p.dispose();
      }
    });

void test("classification ranges use existing feature ownership, atomic maps, clipping and ordered triples", () => {
  const file = "/src/Clipped.art";
  const source = "😀 日本語 abcd -- name -- masked";
  const generated = "helper;nameLong::abcd::masked";
  const atomic = generated.indexOf("nameLong"),
    linear = generated.indexOf("abcd");
  const authoredAtomic = source.indexOf("name"),
    authoredLinear = source.indexOf("abcd");
  const mappings = [
    [atomic, 8, authoredAtomic, 4, 1, helper.LanguageFeature.SemanticTokens],
    [linear, 4, authoredLinear, 4, 0, helper.LanguageFeature.SemanticTokens],
    [generated.indexOf("masked"), 6, source.indexOf("masked"), 6, 0, helper.LanguageFeature.Hover],
  ];
  const packet = {
    spans: [
      0,
      6,
      100,
      atomic,
      8,
      200,
      atomic,
      8,
      200,
      linear - 1,
      6,
      777,
      linear,
      4,
      201,
      generated.indexOf("masked"),
      6,
      999,
    ],
    endOfLineState: 123,
  };
  const calls = [];
  let snapshots = 0;
  const raw = {
    getEncodedSemanticClassifications: (...args) => {
      calls.push(args);
      return packet;
    },
  };
  const service = createMappedLanguageService(ts, raw, {
    engine: helper,
    get: () => {
      snapshots++;
      return { source, projection: { text: generated, mappings } };
    },
  });
  assert.deepEqual(
    service.getEncodedSemanticClassifications(file, { start: 0, length: source.length }, "2020"),
    { spans: [authoredLinear, 4, 201, authoredAtomic, 4, 200], endOfLineState: 123 },
  );
  assert.deepEqual(calls[0], [file, { start: atomic, length: linear + 4 - atomic }, "2020"]);
  assert.deepEqual(
    service.getEncodedSemanticClassifications(
      file,
      { start: authoredLinear + 1, length: 2 },
      "2020",
    ),
    { spans: [authoredLinear + 1, 2, 201], endOfLineState: 123 },
  );
  assert.deepEqual(calls[1][1], { start: linear + 1, length: 2 });
  assert.deepEqual(
    service.getEncodedSemanticClassifications(
      file,
      { start: authoredAtomic + 1, length: 2 },
      "2020",
    ),
    { spans: [authoredAtomic + 1, 2, 200], endOfLineState: 123 },
  );
  assert.deepEqual(calls[2][1], { start: atomic, length: 8 });
  for (const range of [
    { start: source.indexOf("masked"), length: 6 },
    { start: 0, length: 2 },
    { start: authoredLinear, length: 0 },
  ])
    assert.deepEqual(service.getEncodedSemanticClassifications(file, range, "2020"), {
      spans: [],
      endOfLineState: ts.EndOfLineState.None,
    });
  assert.equal(calls.length, 3, "unowned and empty authored ranges never query generated helpers");
  assert.equal(snapshots, 6, "reuse one current document snapshot per request, not per token");
});

void test("ordinary TypeScript retains the original packet, range, format and service call", () => {
  const packet = { spans: [8, 2, 512], endOfLineState: 123 };
  const range = { start: 8, length: 2 };
  const calls = [];
  const raw = Object.fromEntries(
    ["getEncodedSyntacticClassifications", "getEncodedSemanticClassifications"].map((method) => [
      method,
      (...args) => {
        calls.push(args);
        return packet;
      },
    ]),
  );
  const service = createMappedLanguageService(ts, raw, {
    engine: helper,
    get: () => assert.fail("ordinary TypeScript must not resolve a projection"),
  });
  for (const method of Object.keys(raw))
    assert.equal(service[method]("/src/plain.ts", range, "2020"), packet);
  assert.equal(calls[0][1], range);
  assert.deepEqual(calls, [
    ["/src/plain.ts", range, "2020"],
    ["/src/plain.ts", range, "2020"],
  ]);
});
