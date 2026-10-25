//! Closed direct-CSS API custody; these cases never enter the SSR CLI rate.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export const cssMapProducerPaths = [
  "crates/vize_atelier_sfc/src/css.rs",
  "crates/vize_atelier_sfc/src/css/parser.rs",
  "crates/vize_atelier_sfc/src/css/parser/engine_boundary.rs",
  "crates/vize_atelier_sfc/src/css/parser/source_map.rs",
  "vendor/lightningcss/src/printer.rs",
] as const;
const caseRoot = "tests/_fixtures/differential/compiler/css-source-map-10693/";
const id = "compiler/css/source-map-10693";
const roles = [
  ["input.css", "source"],
  ["options.json", "options"],
  ["public-before.json", "public-before"],
  ["source-after.json", "source-after"],
  ["authority.json", "authority"],
] as const;
type Reader = (path: string) => Buffer | null;
type SideReader = (side: "base" | "head", path: string) => Buffer | null;
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

function object(value: unknown, keys: readonly string[]): Record<string, unknown> {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  assert.deepEqual(Object.keys(value).sort(), [...keys].sort(), "closed CSS schema fields differ");
  return value as Record<string, unknown>;
}

export function loadCompilerCssApiCases(value: unknown, read: Reader) {
  if (value === undefined) return [];
  assert(Array.isArray(value));
  assert.equal(value.length, 1, "only the registered direct-CSS family is admitted");
  return value.map((raw: unknown) => {
    const row = object(raw, [
      "id",
      "state",
      "adapter",
      "targets",
      "inputs",
      "genericCli",
      "nativeCompatibility",
      "witness",
    ]);
    assert.equal(row.id, id);
    assert.equal(row.state, "active");
    assert.equal(row.adapter, "compile-css-original-input-map-10693");
    assert.deepEqual(row.targets, ["css"]);
    assert.equal(row.genericCli, "unsupported-css-api");
    assert.equal(row.nativeCompatibility, "unsupported");
    assert.deepEqual(row.witness, {
      runner: "tests/native/css-compile-source-maps.mjs",
      requestCount: 84,
    });
    const input = object(row.inputs, ["root", "files"]);
    assert.equal(input.root, "css-source-map-10693");
    assert(Array.isArray(input.files));
    assert.equal(input.files.length, roles.length);
    const bytes = input.files.map((raw: unknown, index: number) => {
      const pin = object(raw, ["path", "role", "sha256"]);
      assert.equal(pin.path, roles[index][0]);
      assert.equal(pin.role, roles[index][1]);
      assert(typeof pin.sha256 === "string");
      assert.match(pin.sha256, /^[0-9a-f]{64}$/u);
      const content = read(caseRoot + pin.path);
      assert(content !== null, `missing CSS API input ${String(pin.path)}`);
      assert.equal(hash(content), pin.sha256);
      return content;
    });
    const source = bytes[0].toString("utf8");
    const options = object(JSON.parse(bytes[1].toString("utf8")), [
      "filename",
      "minify",
      "sourceMap",
    ]);
    assert.equal(source, "\r\n.雪😀 { color: red }\r\n.b{opacity:0.5}");
    assert.equal(options.filename, "/fixture/组件 space'\"&😀.css");
    assert.equal(options.minify, true);
    assert.equal(options.sourceMap, true);
    const before = object(JSON.parse(bytes[2].toString("utf8")), [
      "code",
      "cssVars",
      "errors",
      "warnings",
    ]);
    const after = object(JSON.parse(bytes[3].toString("utf8")), [
      "code",
      "map",
      "cssVars",
      "errors",
      "warnings",
    ]);
    assert.deepEqual(before, {
      code: ".雪😀{color:red}.b{opacity:.5}",
      cssVars: [],
      errors: [],
      warnings: [],
    });
    const { map, ...preserved } = after;
    assert.deepEqual(preserved, before);
    assert(typeof map === "string");
    assert.deepEqual(JSON.parse(map), {
      version: 3,
      file: options.filename,
      names: [],
      sources: [options.filename],
      sourcesContent: [source],
      mappings: "AACA,eACA",
    });
    const authority = object(JSON.parse(bytes[4].toString("utf8")), [
      "beforeRevision",
      "fixRevision",
      "sourceFiles",
      "scope",
    ]);
    assert.equal(authority.beforeRevision, "b0033883518f364cfbd3dd0ab9464715e9f51758");
    assert.equal(authority.fixRevision, "510b0bb1033a22265e3408693a4aa00b78351cef");
    assert.equal(
      authority.scope,
      "source-inspected direct API; generic CLI unsupported; no whole-product native credit",
    );
    assert(Array.isArray(authority.sourceFiles));
    assert.equal(authority.sourceFiles.length, cssMapProducerPaths.length);
    const sourceFiles = authority.sourceFiles.map((raw: unknown, index: number) => {
      const file = object(raw, ["path", "beforeSha256", "afterSha256"]);
      assert.equal(file.path, cssMapProducerPaths[index]);
      assert(typeof file.afterSha256 === "string");
      assert.match(file.afterSha256, /^[0-9a-f]{64}$/u);
      if (index === 3) assert.equal(file.beforeSha256, null);
      else {
        assert(typeof file.beforeSha256 === "string");
        assert.match(file.beforeSha256, /^[0-9a-f]{64}$/u);
      }
      return {
        path: cssMapProducerPaths[index],
        beforeSha256: file.beforeSha256,
        afterSha256: file.afterSha256,
      };
    });
    return {
      id,
      source,
      options,
      before,
      after,
      sourceFiles,
      publicBeforeSha256: hash(bytes[2]),
      sourceAfterSha256: hash(bytes[3]),
    };
  });
}

export function newCompilerCssApiCases(previous: unknown, current: unknown, read: SideReader) {
  const rows = loadCompilerCssApiCases(current, (file) => read("head", file));
  assert(previous === undefined || Array.isArray(previous));
  const old = new Set((previous ?? []).map((row: { id?: unknown }) => row.id));
  return rows
    .filter((row) => !old.has(row.id))
    .map((row) => {
      for (const file of row.sourceFiles) {
        const before = read("base", file.path),
          after = read("head", file.path);
        assert.equal(
          before === null ? null : hash(before),
          file.beforeSha256,
          `wrong original CSS source ${file.path}`,
        );
        assert(after !== null, `missing CSS producer ${file.path}`);
        assert.equal(hash(after), file.afterSha256, `wrong repaired CSS source ${file.path}`);
      }
      return row.id;
    });
}

export function onlyCompilerCssChanges(paths: readonly string[]) {
  const compiler = paths.filter((file) =>
    /^crates\/vize_(?:armature|relief|atelier_[^/]+)\/src\//u.test(file),
  );
  return (
    compiler.length > 0 &&
    compiler.every((file) => cssMapProducerPaths.some((owned) => owned === file))
  );
}

export function compilerCssApiReport(rows: ReturnType<typeof loadCompilerCssApiCases>) {
  return {
    plannedCases: rows.length,
    genericCliUnsupported: rows.length,
    nativeHandled: 0,
    nativeEquivalent: 0,
    pairedComparisons: 0,
    rows: rows.map((row) => ({
      id: row.id,
      state: "source-inspected",
      genericCli: "unsupported-css-api",
      wholeProductNative: "unsupported",
      inputSha256: hash(Buffer.from(row.source)),
      publicBeforeSha256: row.publicBeforeSha256,
      sourceAfterSha256: row.sourceAfterSha256,
      witness: { runner: "tests/native/css-compile-source-maps.mjs", requestCount: 84 },
    })),
  };
}
