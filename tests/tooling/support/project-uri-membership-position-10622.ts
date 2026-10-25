import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import { root } from "./lsp/paths.ts";

type Roots = { workspace: string; nativeProject: string };
type NativeDocument = { uri: string; text: string };
type NativeQuery = { textDocument: { uri: string }; position: { line: number; character: number } };
type Paths = { authored: ReadonlySet<string>; native: ReadonlySet<string> };
type Replacement = { start: number; end: number; token: string };
const fixture = path.join(root, "tests/_fixtures/differential/lsp/project-uri-membership-10622");
const read = (name: string) => JSON.parse(fs.readFileSync(path.join(fixture, name), "utf8"));
const record = (value: unknown): Record<string, unknown> => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
};
const originalPaths: Paths = {
  authored: new Set(
    (read("original400.inputs.json").files as Array<{ path: string }>).map(
      ({ path: source }) => source,
    ),
  ),
  native: new Set(
    (read("public-0.567.0.native-sources.json") as Array<{ path: string }>).map(
      ({ path: source }) => source,
    ),
  ),
};
let parser: ((source: string, options: unknown) => unknown) | undefined;

function parseImports(source: string) {
  if (!parser) {
    const vue = process.env.VIZE_PROJECT_URI_VUE_PACKAGE;
    assert.ok(vue, "the frozen isolated Vue3.5.41 parser is required for the correctness oracle");
    const manifest = JSON.parse(fs.readFileSync(path.join(vue, "package.json"), "utf8"));
    assert.equal(manifest.version, "3.5.41");
    const sfc = record(createRequire(path.join(vue, "package.json"))("@vue/compiler-sfc"));
    assert.equal(sfc.version, "3.5.41");
    assert.equal(typeof sfc.babelParse, "function");
    parser = sfc.babelParse as (source: string, options: unknown) => unknown;
  }
  const program = record(
    record(parser(source, { sourceType: "module", plugins: ["typescript"] })).program,
  );
  assert.ok(Array.isArray(program.body));
  return program.body.map(record).filter((node) => node.type === "ImportDeclaration");
}

/** Test-only import AST authority; root-looking comments, strings and opaque data cannot relocate. */
export function nativeImportRelocation(source: string, roots: Roots) {
  assert.ok(path.isAbsolute(roots.workspace) && path.isAbsolute(roots.nativeProject));
  assert.notEqual(roots.workspace, roots.nativeProject);
  assert.ok(!roots.workspace.startsWith(`${roots.nativeProject}${path.sep}`));
  assert.ok(!roots.nativeProject.startsWith(`${roots.workspace}${path.sep}`));
  const admitted: Replacement[] = [];
  for (const node of parseImports(source)) {
    const literal = record(node.source);
    assert.equal(literal.type, "StringLiteral");
    assert.equal(typeof literal.value, "string");
    assert.ok(Number.isSafeInteger(literal.start) && Number.isSafeInteger(literal.end));
    const start = Number(literal.start);
    const end = Number(literal.end);
    const raw = source.slice(start, end);
    for (const [physical, token] of [
      [roots.nativeProject, "$NATIVE_PROJECT"],
      [roots.workspace, "$WORKSPACE"],
    ]) {
      if (!(literal.value as string).startsWith(`${physical}/`)) continue;
      assert.ok(["'", '"'].includes(raw[0]) && raw.at(-1) === raw[0]);
      assert.equal(
        raw.slice(1, physical.length + 1),
        physical,
        "root bytes must be the actual import literal prefix",
      );
      admitted.push({ start: start + 1, end: start + 1 + physical.length, token });
    }
  }
  for (const physical of [roots.workspace, roots.nativeProject]) {
    let at = source.indexOf(physical);
    while (at !== -1) {
      assert.ok(
        admitted.some((span) => span.start === at && span.end === at + physical.length),
        "unadmitted root occurrence outside an actual import source literal",
      );
      at = source.indexOf(physical, at + physical.length);
    }
  }
  admitted.sort((a, b) => b.start - a.start);
  const prefix = (offset: number) => {
    let text = source.slice(0, offset);
    for (const span of admitted) {
      assert.ok(
        offset <= span.start || offset >= span.end,
        "native query cannot split a transported import root",
      );
      if (span.end <= offset) text = text.slice(0, span.start) + span.token + text.slice(span.end);
    }
    return text;
  };
  return { text: prefix(source.length), prefix, admitted };
}

function knownPath(
  raw: unknown,
  physical: string,
  token: string,
  paths: ReadonlySet<string>,
  uri: boolean,
) {
  assert.equal(typeof raw, "string");
  const file = uri ? fileURLToPath(raw as string) : (raw as string);
  assert.ok(path.isAbsolute(file));
  const relative = path.relative(physical, file).split(path.sep).join("/");
  assert.ok(paths.has(relative), `unowned transport identity ${String(raw)}`);
  return `${uri ? "file://" : ""}${token}/${relative}`;
}

/** Only the existing transport fields have relocated identities. Every other field is untouched. */
export function normalizePacket(value: unknown, roots: Roots, paths: Paths = originalPaths) {
  const copy = structuredClone(value);
  const row = record(copy);
  const document = record(record(row.params).textDocument);
  document.uri = knownPath(document.uri, roots.workspace, "$WORKSPACE", paths.authored, true);
  const result = record(row.response).result;
  if (row.method === "textDocument/references") {
    assert.ok(Array.isArray(result));
    for (const item of result) {
      const location = record(item);
      location.uri = knownPath(location.uri, roots.workspace, "$WORKSPACE", paths.authored, true);
    }
  }
  if (row.method === "textDocument/completion") {
    assert.ok(Array.isArray(result));
    for (const item of result) {
      const data = record(item).data;
      if (!data || typeof data !== "object" || !("vizeCompletion" in data)) continue;
      const custody = record(record(data).vizeCompletion);
      custody.uri = knownPath(custody.uri, roots.workspace, "$WORKSPACE", paths.authored, true);
      custody.requestUri = knownPath(
        custody.requestUri,
        roots.nativeProject,
        "$NATIVE_PROJECT",
        paths.native,
        true,
      );
      const original = record(record(custody.item).data);
      original.fileName = knownPath(
        original.fileName,
        roots.nativeProject,
        "$NATIVE_PROJECT",
        paths.native,
        false,
      );
    }
  }
  return copy;
}

export function lspUtf16Offset(text: string, position: NativeQuery["position"]) {
  const lines = text.split("\n");
  const { line, character } = position;
  assert.ok(Number.isSafeInteger(line) && line >= 0 && line < lines.length);
  assert.ok(Number.isSafeInteger(character) && character >= 0 && character <= lines[line].length);
  const offset = lines.slice(0, line).reduce((sum, value) => sum + value.length + 1, 0) + character;
  const before = text.charCodeAt(offset - 1);
  const after = text.charCodeAt(offset);
  assert.ok(
    !(before >= 0xd800 && before <= 0xdbff && after >= 0xdc00 && after <= 0xdfff),
    "native LSP position must not split a UTF16 pair",
  );
  return offset;
}

export function assertNativeCompletionExchange(
  row: unknown,
  document: NativeDocument,
  request: unknown,
  response: unknown,
) {
  const query = record(request);
  const reply = record(response);
  assert.equal(query.jsonrpc, "2.0");
  assert.equal(query.method, "textDocument/completion");
  assert.equal(reply.jsonrpc, "2.0");
  assert.equal(reply.id, query.id, "the native response belongs to the actual completion frame");
  assert.equal(reply.error, undefined);
  const params = record(query.params);
  assert.equal(record(params.textDocument).uri, document.uri);
  const position = record(params.position);
  assert.equal(typeof position.line, "number");
  assert.equal(typeof position.character, "number");
  const offset = lspUtf16Offset(document.text, {
    line: Number(position.line),
    character: Number(position.character),
  });
  const bytes = Buffer.byteLength(document.text.slice(0, offset));
  assert.notEqual(
    bytes,
    offset,
    "the actual astral-before-cursor control distinguishes UTF8 bytes from UTF16 units",
  );
  const nativeItems = record(reply.result).items;
  const outerItems = record(record(row).response).result;
  assert.ok(Array.isArray(nativeItems) && Array.isArray(outerItems));
  assert.deepEqual(
    outerItems.map((item) => record(record(record(item).data).vizeCompletion).item),
    nativeItems,
    "every original native completion item is retained completely",
  );
  for (const item of nativeItems) assert.equal(record(record(item).data).position, bytes);
  return { utf8Bytes: bytes, utf16Units: offset };
}

export function sdkPositionUnitEvidence(runtime?: string) {
  const directory = path.join(fixture, "position-unit");
  const evidence = JSON.parse(fs.readFileSync(path.join(directory, "sdk-source.json"), "utf8"));
  assert.equal(evidence.version, "7.0.2");
  assert.equal(evidence.gitHead, "2bd066d87f5bafd315be9f40889d0a60b9e58e0b");
  assert.equal(evidence.completionDataPositionUnit, "UTF8 bytes");
  for (const [index, file] of ["sdk-completions.go.gz", "sdk-converters.go.gz"].entries()) {
    const source = gunzipSync(fs.readFileSync(path.join(directory, file)));
    assert.equal(createHash("sha256").update(source).digest("hex"), evidence.source[index].sha256);
    assert.equal(
      createHash("sha1").update(`blob ${source.length}\0`).update(source).digest("hex"),
      evidence.source[index].blob,
    );
  }
  const installed = runtime
    ? JSON.parse(
        fs.readFileSync(
          path.join(path.dirname(fs.realpathSync(runtime)), "../package.json"),
          "utf8",
        ),
      )
    : JSON.parse(fs.readFileSync(path.join(directory, "public-sdk-package.json"), "utf8"));
  assert.equal(installed.version, evidence.version);
  assert.equal(installed.gitHead, evidence.gitHead);
  return { primary: evidence, actualPackage: installed };
}

export function normalizeMemberPacket(
  value: unknown,
  roots: Roots,
  main: NativeDocument,
  query: NativeQuery,
  expected = {
    roots: read("authority.json").roots as Roots,
    text: gunzipSync(
      fs.readFileSync(path.join(fixture, "public-0.567.0.native-main.ts.gz")),
    ).toString(),
  },
  paths: Paths = originalPaths,
) {
  assert.equal(
    query.textDocument.uri,
    main.uri,
    "the actual native frame owns this acknowledged source",
  );
  knownPath(main.uri, roots.nativeProject, "$NATIVE_PROJECT", paths.native, true);
  const actual = nativeImportRelocation(main.text, roots);
  assert.equal(
    actual.text,
    nativeImportRelocation(expected.text, expected.roots).text,
    "complete native code is equal after only AST-admitted import relocation",
  );
  const offset = lspUtf16Offset(main.text, query.position);
  const nativeBytes = Buffer.byteLength(main.text.slice(0, offset));
  const copy = structuredClone(value);
  const result = record(record(copy).response).result;
  assert.ok(Array.isArray(result));
  for (const item of result) {
    const custody = record(record(record(item).data).vizeCompletion);
    assert.equal(custody.requestUri, main.uri);
    const data = record(record(custody.item).data);
    assert.equal(data.fileName, fileURLToPath(main.uri));
    assert.equal(
      data.position,
      nativeBytes,
      "SDK7.0.2 completion data.position is a UTF8 byte offset, not UTF16",
    );
    data.position = Buffer.byteLength(actual.prefix(offset));
  }
  return normalizePacket(copy, roots, paths);
}
