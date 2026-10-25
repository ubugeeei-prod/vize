import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  newOwnedEditorShutdownCases,
  OWNED_EDITOR_SHUTDOWN_PROTOCOL_ID as ID,
} from "../../tools/support/compat/github/owned-editor-shutdown-fixtures.ts";
import { analyzeLegacyFix } from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const load = (name) => JSON.parse(fs.readFileSync(path.join(root, name), "utf8"));
const manifest = load("tests/_fixtures/differential/typechecker/manifest.json");
const row = manifest.protocolCases.find((entry) => entry.id === ID);
assert.ok(row);
const before = load("tests/_fixtures/runtime/lsp-owned-shutdown-10760/reviewed-before.json");
assert.equal(before.revision, "ce95f3c38fd72ba471b94b5f6078c631035a3381");
assert.equal(before.sources.length, 14);
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");

// Source-only reviewed blobs are immutable negative controls, never producers.
function reader(overrides = new Map()) {
  const sources = new Map(
    before.sources.map((entry) => {
      const bytes = entry.rawBase64 === null ? null : Buffer.from(entry.rawBase64, "base64");
      assert.equal(bytes === null ? null : digest(bytes), entry.sha256);
      assert.deepEqual(
        row.sources.find((source) => source.path === entry.path)?.before,
        entry.sha256,
      );
      return [entry.path, bytes];
    }),
  );
  sources.set(row.lspCase.path, null);
  const oldTypechecker = structuredClone(manifest);
  oldTypechecker.protocolCases = oldTypechecker.protocolCases.filter((entry) => entry.id !== ID);
  sources.set(
    "tests/_fixtures/differential/typechecker/manifest.json",
    Buffer.from(JSON.stringify(oldTypechecker)),
  );
  const oldLsp = load("tests/_fixtures/differential/lsp/manifest.json");
  oldLsp.cases = oldLsp.cases.filter(
    (entry) => entry.id !== "lsp/regression/owned-outer-shutdown-10760",
  );
  sources.set(
    "tests/_fixtures/differential/lsp/manifest.json",
    Buffer.from(JSON.stringify(oldLsp)),
  );
  return (side, name) => {
    if (overrides.has(`${side}:${name}`)) return overrides.get(`${side}:${name}`);
    if (side === "base" && sources.has(name)) return sources.get(name);
    try {
      return fs.readFileSync(path.join(root, name));
    } catch (error) {
      if (error.code === "ENOENT") return null;
      throw error;
    }
  };
}

void test("complete real shutdown ownership inputs register exactly two finite Canon paths", () => {
  const actual = newOwnedEditorShutdownCases(undefined, [row], reader());
  assert.deepEqual(actual.ids, [ID]);
  assert.deepEqual(
    [...actual.sources],
    [
      "crates/vize_canon/src/corsa_bridge/bridge.rs",
      "crates/vize_canon/src/corsa_bridge/bridge/shutdown_tests.rs",
    ],
  );
  assert.equal(actual.sources.has("crates/vize_canon/src/corsa_bridge/other.rs"), false);
  assert.equal(actual.sources.has("crates/vize_maestro/src/server.rs"), false);
});

void test("exact source transition requires a new unique unchanged protocol row", () => {
  for (const [old, next] of [
    [undefined, undefined],
    [[row], [row]],
    [undefined, [row, row]],
    [undefined, [{ ...row, id: "lsp/foreign-shutdown" }]],
    [undefined, [{ ...row, qualification: "structural-only" }]],
  ])
    assert.throws(() => newOwnedEditorShutdownCases(old, next, reader()));
  const sdk = structuredClone(row);
  sdk.producer.sdk.sha256 = "0".repeat(64);
  assert.throws(() => newOwnedEditorShutdownCases(undefined, [sdk], reader()));
});

void test("all Canon and Maestro before/head bytes including absent private tests are mandatory", () => {
  for (const source of row.sources) {
    for (const side of ["base", "head"]) {
      const overrides = new Map([[`${side}:${source.path}`, Buffer.from("unreviewed source")]]);
      assert.throws(
        () => newOwnedEditorShutdownCases(undefined, [row], reader(overrides)),
        `${side}:${source.path}`,
      );
      const ordinary = newOwnedEditorShutdownCases(undefined, undefined, reader(overrides));
      assert.deepEqual(ordinary.ids, []);
      assert.equal(ordinary.sources.size, 0);
    }
  }
  const historical = newOwnedEditorShutdownCases(
    [row],
    [row],
    reader(new Map([[`head:${row.sources[0].path}`, Buffer.from("a later unrelated fix")]])),
  );
  assert.deepEqual(historical.ids, []);
  assert.equal(historical.sources.size, 0);
});

void test("ordinary sources never replace ownership, selectors, whole wire or actual producer authority", () => {
  const data = load(row.lspCase.path);
  const pack = path.dirname(row.lspCase.path);
  const wire = load(row.wireContract.path);
  for (const target of [
    row.lspCase.path,
    row.wireContract.path,
    row.ownership.path,
    ...data.files.map((file) => `${pack}/${file.source}`),
    `${pack}/${data.expected.source}`,
    `${pack}/${wire.initializeRequest.path}`,
    wire.initializerSource.path,
    ...row.producer.files.map((ref) => ref.path),
  ]) {
    const overrides = new Map([[`head:${target}`, Buffer.from("changed authority")]]);
    assert.throws(() => newOwnedEditorShutdownCases(undefined, [row], reader(overrides)), target);
  }
  for (const cases of [[], [null], [{ id: "lsp/ordinary-filler" }]]) {
    const overrides = new Map([
      [
        "head:tests/_fixtures/differential/lsp/manifest.json",
        Buffer.from(JSON.stringify({ cases })),
      ],
    ]);
    assert.throws(() => newOwnedEditorShutdownCases(undefined, [row], reader(overrides)));
  }
  const oldCase = new Map([[`base:${row.lspCase.path}`, Buffer.from("existing unrelated case")]]);
  assert.throws(() => newOwnedEditorShutdownCases(undefined, [row], reader(oldCase)));
});

void test("shared report rejects missing closed row with ordinary filler and mixed unknown Canon edits", () => {
  const paths = row.sources.map((source) => source.path);
  const analyze = (readFile, changed = paths) =>
    analyzeLegacyFix({
      title: "fix(lsp): await owned compiler shutdown",
      paths: changed,
      readFile,
    });
  const accepted = analyze(reader());
  assert.ok(accepted.products.length > 0);
  assert.ok(accepted.products.every((product) => product.state === "new-inputs-registered"));
  assert.ok(accepted.products.some((product) => product.cases.includes(ID)));
  const absent = structuredClone(manifest);
  absent.protocolCases = absent.protocolCases.filter((entry) => entry.id !== ID);
  absent.cases.push({
    ...structuredClone(absent.cases[0]),
    id: "typechecker/ordinary-filler-10760",
  });
  const missing = analyze(
    reader(
      new Map([
        [
          "head:tests/_fixtures/differential/typechecker/manifest.json",
          Buffer.from(JSON.stringify(absent)),
        ],
      ]),
    ),
  );
  assert.ok(missing.products.some((product) => product.state !== "new-inputs-registered"));
  const mixed = analyze(reader(), [...paths, "crates/vize_canon/src/batch/unknown.rs"]);
  assert.ok(mixed.products.some((product) => product.state === "new-input-missing"));
  const unknown = structuredClone(manifest);
  unknown.protocolCases.push({ ...structuredClone(row), id: "lsp/unknown-outer-shutdown" });
  const rejected = analyze(
    reader(
      new Map([
        [
          "head:tests/_fixtures/differential/typechecker/manifest.json",
          Buffer.from(JSON.stringify(unknown)),
        ],
      ]),
    ),
  );
  assert.ok(rejected.products.some((product) => product.state === "invalid-manifest-or-input"));
});
