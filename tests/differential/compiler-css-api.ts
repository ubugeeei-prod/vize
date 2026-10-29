//! Closed direct-CSS API custody; these cases never enter the SSR CLI rate.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

import {
  compilerCssSpec,
  cssMapProducerPaths,
  knownCompilerCssSource,
} from "./compiler-css-api-registry.ts";
export { compilerCssApiSources, cssMapProducerPaths } from "./compiler-css-api-registry.ts";
const commonRoles = [
  ["input.css", "source"],
  ["options.json", "options"],
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
  assert(value.length <= 2, "only the registered direct-CSS families are admitted");
  const ids = new Set<unknown>();
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
    const spec = compilerCssSpec(row.id);
    assert(spec !== undefined, "unknown direct-CSS family");
    assert(!ids.has(row.id), "duplicate direct-CSS family");
    ids.add(row.id);
    const roles = [
      ...commonRoles,
      [
        spec.sourceBefore ? "source-before.json" : "public-before.json",
        spec.sourceBefore ? "source-before" : "public-before",
      ],
      ["source-after.json", "source-after"],
      ["authority.json", "authority"],
      ...(spec.sourceBefore
        ? [["source-before-observation.json", "source-before-observation"]]
        : []),
    ];
    const caseRoot = `tests/_fixtures/differential/compiler/${spec.root}/`;
    assert.equal(row.state, "active");
    assert.equal(row.adapter, spec.adapter);
    assert.deepEqual(row.targets, ["css"]);
    assert.equal(row.genericCli, "unsupported-css-api");
    assert.equal(row.nativeCompatibility, "unsupported");
    assert.deepEqual(row.witness, spec.witness);
    const input = object(row.inputs, ["root", "files"]);
    assert.equal(input.root, spec.root);
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
    const options = object(JSON.parse(bytes[1].toString("utf8")), Object.keys(spec.options));
    assert.equal(source, spec.source);
    assert.deepEqual(options, spec.options);
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
    assert.deepEqual(before, { code: spec.code, cssVars: spec.vars, errors: [], warnings: [] });
    if (spec.sourceBefore) {
      const observation = object(JSON.parse(bytes[5].toString("utf8")), ["identity", "call"]);
      const identity = observation.identity as Record<string, unknown>;
      assert.equal(identity.source, spec.beforeRevision);
      // This was the head in its original, retained first-slice ABBA campaign.
      assert.equal(identity.side, "head");
      assert.equal(identity.targetInitiallyEmpty, true);
      assert.equal(
        identity.binarySha256,
        "143afa198462e88b50e9ba9201207b406e015df7bce0f995b4e6795239a63707",
      );
      assert.equal(identity.token, "692c0d492de2d9c6736e365c5e2314e0");
      assert.deepEqual(observation.call, { source, options, result: before });
    }
    const { map, ...preserved } = after;
    assert.deepEqual(preserved, before);
    assert(typeof map === "string");
    assert.deepEqual(JSON.parse(map), {
      version: 3,
      file: spec.filename,
      names: [],
      sources: [spec.filename],
      sourcesContent: [source],
      mappings: spec.mappings,
    });
    const authority = object(JSON.parse(bytes[4].toString("utf8")), [
      "beforeRevision",
      "fixRevision",
      "sourceFiles",
      "scope",
    ]);
    assert.equal(authority.beforeRevision, spec.beforeRevision);
    assert.equal(authority.fixRevision, spec.fixRevision);
    assert.equal(
      authority.scope,
      "source-inspected direct API; generic CLI unsupported; no whole-product native credit",
    );
    assert(Array.isArray(authority.sourceFiles));
    assert.equal(authority.sourceFiles.length, spec.paths.length);
    const sourceFiles = authority.sourceFiles.map((raw: unknown, index: number) => {
      const file = object(raw, ["path", "beforeSha256", "afterSha256"]);
      assert.equal(file.path, spec.paths[index]);
      assert(typeof file.afterSha256 === "string");
      assert.match(file.afterSha256, /^[0-9a-f]{64}$/u);
      if (spec.missingBefore.includes(index)) assert.equal(file.beforeSha256, null);
      else {
        assert(typeof file.beforeSha256 === "string");
        assert.match(file.beforeSha256, /^[0-9a-f]{64}$/u);
      }
      return {
        path: spec.paths[index],
        beforeSha256: file.beforeSha256,
        afterSha256: file.afterSha256,
      };
    });
    return {
      id: spec.id,
      source,
      options,
      before,
      after,
      sourceFiles,
      beforeKind: spec.sourceBefore ? "source" : "public",
      beforeSha256: hash(bytes[2]),
      witness: spec.witness,
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

export function onlyCompilerCssChanges(
  paths: readonly string[],
  admitted: ReadonlySet<string> = new Set(cssMapProducerPaths),
) {
  const compiler = paths.filter(
    (file) =>
      knownCompilerCssSource(file) ||
      /^crates\/vize_(?:armature|relief|atelier_[^/]+)\/src\//u.test(file),
  );
  return compiler.length > 0 && compiler.every((file) => admitted.has(file));
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
      [row.beforeKind === "source" ? "sourceBeforeSha256" : "publicBeforeSha256"]: row.beforeSha256,
      sourceAfterSha256: row.sourceAfterSha256,
      witness: row.witness,
    })),
  };
}
