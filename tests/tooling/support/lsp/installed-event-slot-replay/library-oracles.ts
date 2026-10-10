import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { isDeepStrictEqual } from "node:util";
import { gunzipSync } from "node:zlib";
import type { Packet } from "../installed-alias-replay/protocol.ts";

type ObjectValue = Record<string, any>;
export type LibraryCase = {
  context: string;
  kind: "safe" | "cold" | "ambiguous" | "partial";
  inputs: ObjectValue;
  expected: ObjectValue | ObjectValue[];
  publicationSequences: ObjectValue[];
  parentOpen: boolean;
  newline: "\n" | "\r\n";
};
export type LibraryRecord = { expected: ObjectValue; actual: ObjectValue; failures: ObjectValue[] };
export interface LibrarySession {
  request(method: string, params?: unknown): Promise<Packet>;
  open(uri: string, text: string, version?: number): Promise<Packet>;
  changeWithPublications(
    uri: string,
    text: string,
    version: number,
    count: number,
  ): Promise<Packet[]>;
  readonly notifications: readonly Packet[];
}
export const libraryManifestSha256 =
  "a739b97adfda8284e0704a4175d71142cc8775da62f4ce3df4b805012a1bb385";
export const libraryGuardSource =
  "<script setup lang=\"ts\">\nconst nativeGuard: number = 'wrong';\n</script>\n<template>{{ nativeGuard }}</template>\n";
export const libraryGuardRepair = libraryGuardSource.replace("'wrong'", "1");
export const libraryTsconfig =
  '{\n  "compilerOptions": {\n    "lib": ["ESNext", "DOM"], "strict": true,\n    "moduleResolution": "Bundler", "module": "ESNext", "target": "ESNext"\n  },\n  "include": ["src/**/*.vue"]\n}';
export const libraryVizeConfig = Object.freeze({
  typeChecker: Object.freeze({}),
  lsp: Object.freeze({ lint: true, typecheck: true, hover: true, crossFile: true }),
});
const names = ["src/Toggle.vue", "src/App.vue"] as const;
const digest = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
const uri = (root: string, file: string) => pathToFileURL(path.join(root, file)).href;
const publication = (root: string, file: string, version: number, diagnostics: unknown) => ({
  jsonrpc: "2.0",
  method: "textDocument/publishDiagnostics",
  params: { uri: uri(root, file), version, diagnostics },
});
function freeze<T>(value: T): T {
  if (value && typeof value === "object") {
    Object.values(value).forEach(freeze);
    Object.freeze(value);
  }
  return value;
}

/** Select only preauthored fields: the historical actual and witness are never read as an oracle. */
export function selectLibraryOracle(packet: ObjectValue): LibraryCase {
  assert.equal(packet.tsconfig, libraryTsconfig);
  const config = JSON.parse(packet.vizeConfig);
  assert.equal(typeof config.typeChecker?.corsaPath, "string");
  delete config.typeChecker.corsaPath;
  assert.deepEqual(
    config,
    libraryVizeConfig,
    "only the explicit workspace Corsa path may be removed",
  );
  assert.equal(packet.sourceSha, null);
  const context: string = packet.context;
  const kind = context.startsWith("original strict event safe update ")
    ? "safe"
    : context.startsWith("cold event/value collision, ")
      ? "cold"
      : context.startsWith("ambiguous mutable event whole refusal, ")
        ? "ambiguous"
        : "partial";
  assert.match(
    context,
    /^(original strict event safe update (Original|UnsavedEmoji|Restored), parent_open=(false|true)|cold event\/value collision, unopened parent|ambiguous mutable event whole refusal, parent_open=(false|true)|incomplete event application (MissingCall|CallOnly)), newline="\\(?:n|r\\n)"$/u,
  );
  return freeze({
    context,
    kind,
    inputs: packet.inputs,
    expected: packet.expected,
    publicationSequences: packet.publicationSequences.map((row: ObjectValue) => ({
      file: row.file,
      version: row.version,
      opening: row.opening,
      expected: row.expected,
    })),
    parentOpen: context.includes("parent_open=true") || kind === "partial",
    newline: context.endsWith('newline="\\r\\n"') ? "\r\n" : "\n",
  });
}
/** Authenticate immutable ZIP-member bytes and original source files before any provider launch. */
export function loadFrozenLibraryCases(root: string): readonly LibraryCase[] {
  const bytes = fs.readFileSync(path.join(root, "manifest.json.txt"));
  assert.equal(digest(bytes), libraryManifestSha256);
  const manifest = JSON.parse(bytes.toString("utf8"));
  assert.equal(manifest.schema, "vize-installed-library22-originals-v1");
  const checked = (file: string, sha: string) => {
    const value = fs.readFileSync(path.join(root, file));
    assert.equal(digest(value), sha);
    return value;
  };
  const receipts = JSON.parse(
    checked(manifest.artifactReceipt.file, manifest.artifactReceipt.sha256).toString("utf8"),
  );
  for (const archive of manifest.archives) {
    assert.ok(
      receipts.some((receipt: ObjectValue) =>
        isDeepStrictEqual(
          receipt,
          Object.fromEntries(
            Object.entries(archive).filter(([key]) => !key.startsWith("metadata")),
          ),
        ),
      ),
    );
    const metadata = JSON.parse(
      checked(archive.metadataFile, archive.metadataSha256).toString("utf8"),
    );
    assert.equal(metadata.id, archive.id);
    assert.equal(metadata.name, archive.name);
    assert.equal(metadata.digest, `sha256:${archive.zipSHA256}`);
    assert.equal(metadata.workflow_run.id, archive.run);
    assert.equal(metadata.workflow_run.head_sha, manifest.sourceHead);
  }
  const sourceRoot = path.resolve(root, "../../../../../..");
  for (const [file, sha] of Object.entries(manifest.sourceFiles))
    assert.equal(digest(fs.readFileSync(path.join(sourceRoot, file))), sha);
  const cases = manifest.cases.map((entry: ObjectValue) => {
    const raw = gunzipSync(checked(entry.file, entry.gzipSha256));
    assert.equal(digest(raw), entry.rawSha256);
    const oracle = selectLibraryOracle(JSON.parse(raw.toString("utf8")));
    assert.equal(oracle.context, entry.context);
    return oracle;
  });
  assert.equal(cases.length, 22);
  assert.equal(new Set(cases.map((item: LibraryCase) => item.context)).size, 22);
  assert.deepEqual(
    ["safe", "cold", "ambiguous", "partial"].map(
      (kind) => cases.filter((item: LibraryCase) => item.kind === kind).length,
    ),
    [12, 2, 4, 4],
  );
  return freeze(cases);
}

/** Only exact historical owned URI keys/values move; unknown identities always refuse. */
export function remapLibraryExpected(value: unknown, projectRoot: string): any {
  assert.ok(path.isAbsolute(projectRoot));
  const roots = new Set<string>();
  const visit = (item: unknown): any => {
    if (typeof item === "string" && /^[A-Za-z][A-Za-z0-9+.-]*:/u.test(item)) {
      const match =
        /^(file:\/\/\/home\/runner\/_work\/vize\/vize\/target\/vize-tests\/tests\/lsp-rename-authority-[A-Za-z0-9]+)\/(src\/(?:Toggle|App)\.vue)$/u.exec(
          item,
        );
      assert.ok(match, "unknown historical URI");
      roots.add(match[1]);
      assert.equal(roots.size, 1);
      return uri(projectRoot, match[2]);
    }
    if (Array.isArray(item)) return item.map(visit);
    if (item && typeof item === "object")
      return Object.fromEntries(
        Object.entries(item).map(([key, entry]) => [
          /^[A-Za-z][A-Za-z0-9+.-]*:/u.test(key) ? visit(key) : key,
          visit(entry),
        ]),
      );
    return item;
  };
  return freeze(visit(value));
}
export function prepareLibraryExpectation(oracle: LibraryCase, projectRoot: string): ObjectValue {
  const [transaction, sequences] = remapLibraryExpected(
    [oracle.expected, oracle.publicationSequences],
    projectRoot,
  );
  const invalid = publication(projectRoot, "src/NativeGuard.vue", 1, [
    {
      range: { start: { line: 1, character: 6 }, end: { line: 1, character: 17 } },
      severity: 1,
      code: 2322,
      source: "vize/types",
      message: "Type 'string' is not assignable to type 'number'.",
    },
  ]);
  const repaired = [
    publication(projectRoot, "src/NativeGuard.vue", 2, []),
    publication(projectRoot, "src/NativeGuard.vue", 2, []),
  ];
  const initial = names
    .slice(0, oracle.parentOpen ? 2 : 1)
    .map((file) => publication(projectRoot, file, 1, []));
  const requests =
    oracle.kind === "ambiguous"
      ? (transaction as ObjectValue[]).map((row) => ({
          method: "textDocument/rename",
          result: row.rename,
        }))
      : [
          { method: "textDocument/rename", result: transaction.rename },
          ...(oracle.kind === "partial"
            ? []
            : [{ method: "textDocument/references", result: transaction.references }]),
        ];
  return freeze({
    transaction,
    nativeGuard: { invalid, repaired, disk: libraryGuardSource },
    initialDisk: names.map((file, index) => ({
      file,
      text: index === 0 ? (oracle.inputs.toggleDisk ?? oracle.inputs.toggle) : oracle.inputs.app,
    })),
    initialPublications: initial,
    responses: requests.map((request, index) => ({
      method: request.method,
      packet: { jsonrpc: "2.0", id: index + 2, result: request.result },
    })),
    publicationSequences: sequences.map((row: ObjectValue) => ({
      file: row.file,
      version: row.version,
      opening: row.opening,
      packets: row.expected,
    })),
    libraryBytesUnchanged: true,
    publicationGroups: [
      [invalid],
      repaired,
      ...initial.map((packet) => [packet]),
      ...sequences.map((row: ObjectValue) => row.expected),
    ],
  });
}
