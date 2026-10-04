import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";

import {
  comparePageDiagnostics,
  type DiagnosticRenderContext,
  type DiagnosticRow,
} from "../fixtures/typechecker/page-route-types/diagnostic-rendering.ts";

// Independent original raw cf/37241714582 observation 073, SHA256
// 207547119fff623cc3f99dd9dd0278b7d48baa87461bae21a8d81f813f3931c7.
// Full source and both messages are copied from the raw observation, not helper exports.
const PAGE = "packages/playground-file-based/src/pages/users/[userId=int].vue";
const SOURCE =
  "<script lang=\"ts\" setup>\nimport { useRoute } from 'vue-router'\n\nconst route = useRoute(); const typedId: number = route.params.userId\n\nroute.params.userId\nroute.params.anyParam\nroute.params.page\n\ndefinePage({\n  // path: '/users/:userId(\\\\d+)',\n  // this doesn't work in custom param version and should warn\n  // path: '/users/:userId',\n  params: {\n    path: {\n      unknownId: 'int',\n    },\n    query: {\n      anyParam: {\n        default: '',\n      },\n      page: {\n        parser: 'int',\n        default: 1,\n        format: 'value',\n      },\n    },\n  },\n  meta: {\n    // te: 3, 2\n  },\n})\n</script>\n\n<template>\n  <h1>User by id - {{ $route.params.userId.toFixed() }}</h1>\n\n  <pre>{{ route.params }}</pre>\n</template>\n";
const NATIVE =
  'Object literal may only specify known properties, and \'unknownId\' does not exist in type \'{ userId?: "date" | "month-valibot" | "month-zod" | "npm-org" | "semver" | "set" | "test-bool-q" | "test-color" | "test-csv" | "test-num" | "test-set" | "test-set-shape" | "version-range" | keyof ParamParsers_Native | undefined; }\'.';
const REFERENCE =
  'Object literal may only specify known properties, and \'unknownId\' does not exist in type \'{ userId?: keyof ParamParsers_Native | "date" | "month-valibot" | "month-zod" | "npm-org" | "semver" | "set" | "test-bool-q" | "test-color" | "test-csv" | ... 4 more ... | undefined; }\'.';

const context: DiagnosticRenderContext = {
  renderCase: "define-page-unknown-id",
  sourcePath: PAGE,
  source: SOURCE,
  providerArchiveSha256: "5c0bf884438b9c58b1e926663e07572c80c5dcc8d0ddd57a32943a0161d8ee5f",
  generatedRoutesSha256: "1a19bf3a6f143d7b7ee5c931d0f8da9a4e0a06f378278fc2615f99db746d95c8",
};

function row(message: string): DiagnosticRow {
  return { file: PAGE, line: 16, column: 7, code: 2353, message };
}

function compare(
  actual = [row(NATIVE)],
  reference = [row(REFERENCE)],
  overrides: Partial<DiagnosticRenderContext> = {},
) {
  return comparePageDiagnostics(actual, reference, { ...context, ...overrides });
}

await test("the observed render pair validates without copying or changing native rows", () => {
  assert.equal(
    createHash("sha256").update(SOURCE, "utf8").digest("hex"),
    "fc7bc0530fc13ee5fcc67cba6f3e54b1a86280fdf80caf679e1fedbf58199844",
  );
  const actual = [row(NATIVE)];
  const reference = [row(REFERENCE)];
  const original = structuredClone({ actual, reference });
  assert.strictEqual(compare(actual, reference), actual);
  assert.deepEqual({ actual, reference }, original);
  assert.notEqual(NATIVE, REFERENCE, "no cross-engine whole-message equality is claimed");
});

await test("default comparisons retain complete ordinary vectors and their order", () => {
  const first = { file: "Ordinary.vue", line: 2, column: 4, code: 2322, message: "Type mismatch." };
  const second = {
    file: "Ordinary.vue",
    line: 9,
    column: 1,
    code: 2339,
    message: "Missing property.",
  };
  const actual = [first, second];
  const plain = { sourcePath: "Ordinary.vue", source: "" };
  assert.strictEqual(comparePageDiagnostics(actual, structuredClone(actual), plain), actual);
  assert.strictEqual(
    comparePageDiagnostics(actual, structuredClone(actual), { ...plain, renderCase: undefined }),
    actual,
  );
  assert.deepEqual(comparePageDiagnostics([], [], plain), []);
  assert.throws(() => comparePageDiagnostics(actual, [second, first], plain));
  assert.throws(() => comparePageDiagnostics(actual, [first], plain));
  assert.throws(() =>
    comparePageDiagnostics(actual, [{ ...first, message: "Type  mismatch." }, second], plain),
  );
  assert.throws(() => comparePageDiagnostics([row(NATIVE)], [row(REFERENCE)], plain));
});

await test("each complete frozen engine message rejects corruption and alternative rendering", async (t) => {
  const changes = [
    ["native message", NATIVE.replace("'unknownId'", "'anotherId'"), REFERENCE],
    ["reference message", NATIVE, REFERENCE.replace("'unknownId'", "'anotherId'")],
    ["native truncation", REFERENCE, REFERENCE],
    ["reference expansion", NATIVE, NATIVE],
    ["reference elision count", NATIVE, REFERENCE.replace("... 4 more ...", "... 3 more ...")],
    ["native parser name", NATIVE.replace('"test-num"', '"other-parser"'), REFERENCE],
    ["reference parser name", NATIVE, REFERENCE.replace('"date"', '"other-parser"')],
    ["native whitespace", NATIVE.replace("Object literal", "Object  literal"), REFERENCE],
    ["reference whitespace", NATIVE, REFERENCE.replace("Object literal", "Object  literal")],
    [
      "native union order",
      NATIVE.replace('"date" | "month-valibot"', '"month-valibot" | "date"'),
      REFERENCE,
    ],
  ];
  for (const [name, native, official] of changes) {
    await t.test(name, () => {
      assert.throws(() => compare([row(native)], [row(official)]));
    });
  }
});

await test("full vectors reject extra, missing and misplaced diagnostics", async (t) => {
  const additional = { ...row(NATIVE), line: 17, code: 2322, message: "Unexpected extra error." };
  const changes: Array<[string, DiagnosticRow[], DiagnosticRow[]]> = [
    ["missing native", [], [row(REFERENCE)]],
    ["missing reference", [row(NATIVE)], []],
    ["both missing", [], []],
    ["native extra", [row(NATIVE), additional], [row(REFERENCE)]],
    ["reference extra", [row(NATIVE)], [row(REFERENCE), additional]],
    ["native reordered extra", [additional, row(NATIVE)], [row(REFERENCE)]],
    ["reference reordered extra", [row(NATIVE)], [additional, row(REFERENCE)]],
    ["both extra", [row(NATIVE), additional], [row(REFERENCE), additional]],
  ];
  for (const [name, actual, reference] of changes) {
    await t.test(name, () => assert.throws(() => compare(actual, reference)));
  }
});

await test("full frozen locations and codes reject drift on either engine", async (t) => {
  const changes: Array<[string, Partial<DiagnosticRow>]> = [
    ["code", { code: 2339 }],
    ["line", { line: 15 }],
    ["column", { column: 8 }],
    ["file", { file: "packages/playground-file-based/src/pages/other.vue" }],
  ];
  for (const [name, change] of changes) {
    await t.test(`native ${name}`, () =>
      assert.throws(() => compare([{ ...row(NATIVE), ...change }])),
    );
    await t.test(`reference ${name}`, () =>
      assert.throws(() => compare([row(NATIVE)], [{ ...row(REFERENCE), ...change }])),
    );
  }
  await t.test("unknown native row fields", () =>
    assert.throws(() => compare([{ ...row(NATIVE), severity: 2 } as DiagnosticRow])),
  );
  await t.test("unknown reference row fields", () =>
    assert.throws(() =>
      compare([row(NATIVE)], [{ ...row(REFERENCE), severity: 2 } as DiagnosticRow]),
    ),
  );
});

await test("only the original source, path, provider and complete route map authorize the render", async (t) => {
  const changes: Array<[string, Partial<DiagnosticRenderContext>]> = [
    ["different source", { source: SOURCE.replace("unknownId", "anotherId") }],
    ["source byte change", { source: SOURCE + "\n" }],
    ["source unicode change", { source: SOURCE + "😀" }],
    ["other page", { sourcePath: PAGE.replace("[userId=int]", "[otherId=int]") }],
    ["URI-encoded path", { sourcePath: PAGE.replace("[userId=int]", "%5BuserId=int%5D") }],
    ["wrong provider", { providerArchiveSha256: "0".repeat(64) }],
    ["missing provider", { providerArchiveSha256: undefined }],
    ["wrong routes", { generatedRoutesSha256: "0".repeat(64) }],
    ["missing routes", { generatedRoutesSha256: undefined }],
  ];
  for (const [name, change] of changes) {
    await t.test(name, () => assert.throws(() => compare(undefined, undefined, change)));
  }
  for (const renderCase of ["unknown", "", null, false]) {
    await t.test(`unknown render case ${JSON.stringify(renderCase)}`, () => {
      assert.throws(() =>
        Reflect.apply(comparePageDiagnostics, undefined, [
          [row(NATIVE)],
          [row(REFERENCE)],
          { ...context, renderCase },
        ]),
      );
    });
  }
});
