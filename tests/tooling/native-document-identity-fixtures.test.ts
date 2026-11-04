import assert from "node:assert/strict";
import fs from "node:fs";
import { gunzipSync } from "node:zlib";
import { test } from "node:test";
import {
  newNativeDocumentIdentityCases,
  assertCompiledIdentityConsumer,
  nativeDocumentIdentityActor,
  NATIVE_DOCUMENT_IDENTITY_ID,
} from "../../tools/support/compat/github/native-document-identity-fixtures.ts";
import { analyzeLegacyFix } from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";
import { loadLspManifest } from "../differential/lsp-manifest.ts";

const manifestPath = "tests/_fixtures/differential/typechecker/manifest.json";
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
const rows = manifest.nativeDocumentIdentityCases;
const row = rows[0];
const originals: Record<string, string> = JSON.parse(
  gunzipSync(fs.readFileSync(row.sourceArchive.path)).toString("utf8"),
);
const beforeManifest = structuredClone(manifest);
delete beforeManifest.nativeDocumentIdentityCases;
type Read = (side: "base" | "head", path: string) => Buffer | null;
const actor = nativeDocumentIdentityActor(
  (_, path) => (fs.existsSync(path) ? fs.readFileSync(path) : null),
  rows,
);
const read: Read = (side, path) => {
  if (side === "base" && path === manifestPath) return Buffer.from(JSON.stringify(beforeManifest));
  if (side === "base" && Object.hasOwn(originals, path)) {
    const original = originals[path];
    assert.ok(typeof original === "string");
    return Buffer.from(original, "base64");
  }
  if (side === "head" && actor.has(path)) return actor.get(path)!;
  return fs.existsSync(path) ? fs.readFileSync(path) : null;
};
const validate = (reader: Read = read, current: unknown = rows, previous: unknown = []) =>
  newNativeDocumentIdentityCases(previous, current, reader);
const changedSource = (side: "base" | "head", path: string) =>
  new RegExp(`changed ${side} identity source: ${path.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}`);
const corrupt =
  (world: "base" | "head", target: string): Read =>
  (side, path) =>
    side === world && path === target
      ? Buffer.from("unregistered changed bytes")
      : read(side, path);

test("the exact identity transition consumes its input and retains every ABI law", () => {
  const result = validate();
  assert.deepEqual(result.ids, [NATIVE_DOCUMENT_IDENTITY_ID]);
  assert.deepEqual(
    [...result.sources],
    row.sources.map((source: { path: string }) => source.path),
  );
  assert.equal(result.sources.size, 17);
  assert.equal(row.nativeQualification, false);
  assert.equal(row.legacyProtocol.inputs.length, 4);
  assert.equal(row.legacyProtocol.tests.length, 5);
});

test("identity registration cannot cover another source, input, consumer or repeated transition", () => {
  for (const source of row.sources) {
    assert.throws(() => validate(corrupt("base", source.path)), changedSource("base", source.path));
    assert.throws(() => validate(corrupt("head", source.path)), changedSource("head", source.path));
  }
  for (const target of [
    row.sourceArchive.path,
    ...row.inputs.map((input: { path: string }) => input.path),
  ]) {
    assert.throws(() => validate(corrupt("head", target)), changedSource("head", target));
  }
  assert.deepEqual([...validate(read, rows, rows).sources], []);
  assert.throws(
    () => validate(read, [row, row]),
    /identity transition requires one registered row/,
  );
  for (const field of ["sources", "tests", "inputs"] as const) {
    const altered = structuredClone(row);
    altered[field] = altered[field].slice(1);
    assert.throws(
      () => validate(read, [altered]),
      new RegExp(`identity transition requires .* ${field}`),
    );
  }
  const altered = structuredClone(row);
  altered.sources[0].path = "crates/vize_canon/src/unknown.rs";
  assert.throws(() => validate(read, [altered]), /unregistered identity transition/);
});

test("four historical inputs, all five current consumers and actual A/B producers remain required", () => {
  for (const input of row.legacyProtocol.inputs) {
    assert.throws(() => validate(corrupt("base", input.path)), changedSource("base", input.path));
    assert.throws(() => validate(corrupt("head", input.path)), changedSource("head", input.path));
  }
  for (const consumer of row.legacyProtocol.tests) {
    assert.throws(
      () => validate(corrupt("head", consumer.source)),
      changedSource("head", consumer.source),
    );
  }
  assert.throws(
    () => validate(corrupt("head", row.legacyProtocol.workflow.path)),
    changedSource("head", row.legacyProtocol.workflow.path),
  );
  for (const [path, expected] of [
    [
      "tests/tooling/support/native-beta-dto/prepare-role.ts",
      /unregistered current common-observer preparation runtime/,
    ],
    ["tests/tooling/support/native-beta-dto/run-source.ts", /typed producer executable changed/],
    [
      "tests/tooling/support/native-beta-dto/audit-wire.ts",
      /unregistered current watcher observer runtime/,
    ],
  ] as const) {
    assert.throws(
      () =>
        validate((side, file) =>
          side === "head" && file === path
            ? Buffer.concat([
                read(side, file)!,
                Buffer.from('\nvoid "identity producer tamper";\n'),
              ])
            : read(side, file),
        ),
      expected,
    );
  }
  for (const side of ["base", "head"] as const) {
    const altered = structuredClone(side === "base" ? beforeManifest : manifest);
    altered.protocolCases
      .find((entry: { id: string }) => entry.id === row.legacyProtocol.id)
      .tests.pop();
    assert.throws(
      () =>
        validate((world, path) =>
          world === side && path === manifestPath
            ? Buffer.from(JSON.stringify(altered))
            : read(world, path),
        ),
      new RegExp(`${side} historical ABI registration changed`),
    );
  }
});

test("the shared fix gate requires this source contract for component identity changes", () => {
  const paths = row.sources.map((source: { path: string }) => source.path);
  const report = (registered: boolean) =>
    analyzeLegacyFix({
      title: "fix(lsp): use one native document identity",
      paths,
      readFile: (side, path) => {
        if (side === "head" && path === manifestPath && !registered)
          return Buffer.from(JSON.stringify(beforeManifest));
        return read(side, path);
      },
    }).products.find((product) => product.product === "typechecker");
  assert.equal(report(false)?.state, "new-input-missing");
  assert.equal(report(true)?.state, "new-inputs-registered");
});

test("the LSP input materializes a complete authored import definition without native qualification", () => {
  const fixture = loadLspManifest("tests/_fixtures/differential/lsp/manifest.json").cases.find(
    (row) => row.id === "lsp/regression/native-document-identity-11586",
  );
  assert.ok(fixture);
  assert.equal(fixture.requests.length, 1);
  const [request] = fixture.requests;
  assert.ok(request);
  assert.deepEqual(request.result, {
    uri: "${workspace}/src/format.ts",
    range: { start: { line: 0, character: 16 }, end: { line: 0, character: 22 } },
  });
  assert.equal(fixture.data.initializationOptions?.typecheck, false);
});

test("registered identity custody cannot disappear with its manifest field", () => {
  assert.throws(
    () => newNativeDocumentIdentityCases(rows, undefined, read),
    /registered identity history must remain/,
  );
  for (const previous of [undefined, []])
    assert.deepEqual(newNativeDocumentIdentityCases(previous, undefined, read).ids, []);
});

test("each named identity consumer owns its test attribute", () => {
  assertCompiledIdentityConsumer("#[test]\nfn intended() {}", "module::intended", null);
  assertCompiledIdentityConsumer(
    '#[test]\n#[ignore = "actual native runtime only"]\nfn intended() {}',
    "module::intended",
    null,
  );
  assert.throws(
    () =>
      assertCompiledIdentityConsumer(
        "#[test]\nfn other() {}\nfn intended() {}",
        "module::intended",
        null,
      ),
    /test attribute must belong to intended/,
  );
});

test("the immutable actor refuses missing, altered and unregistered source images", () => {
  const archive =
    "tests/_fixtures/differential/typechecker/native-document-identity-11586/after-source.json.gz";
  assert.equal(actor.size, 17);
  for (const replacement of [null, Buffer.from("unregistered actor")]) {
    assert.throws(
      () =>
        nativeDocumentIdentityActor(
          (side, file) => (file === archive ? replacement : read(side, file)),
          rows,
        ),
      /(?:missing|changed) head identity source: .*after-source/,
    );
  }
  const changed = structuredClone(rows);
  changed[0].sources[0].after = "0".repeat(64);
  assert.throws(
    () => nativeDocumentIdentityActor(read, changed),
    /unregistered historical identity actor/,
  );
});

test("historical actor replay does not qualify a changed initial source registration", () => {
  assert.throws(
    () =>
      newNativeDocumentIdentityCases([], rows, (side, file) => {
        if (side === "head" && actor.has(file))
          return Buffer.from("unregistered source correction");
        return read(side, file);
      }),
    /changed head identity source:/,
  );
});
