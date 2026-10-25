import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  newSharedEditorInitialApiCases,
  SHARED_EDITOR_INITIAL_API_PROTOCOL_ID as ID,
} from "../../tools/support/compat/github/shared-editor-initial-api-fixtures.ts";
import { analyzeLegacyFix } from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const manifest = JSON.parse(
  fs.readFileSync(
    path.join(root, "tests/_fixtures/differential/typechecker/manifest.json"),
    "utf8",
  ),
);
const row = manifest.protocolCases.find((row) => row.id === ID);
assert.ok(row);

// Isolated registry laws reconstruct the reviewed pre-change bytes, which the
// validator must hash exactly. These values are never executable producers.
function reader(overrides = new Map()) {
  const initialPath = "crates/vize_canon/src/lsp_client/initial_diagnostics.rs";
  const clientPath = "crates/vize_canon/src/lsp_client/editor_lsp/client.rs";
  const initial = fs
    .readFileSync(path.join(root, initialPath), "utf8")
    .replace(
      "        // Editor overlays use retained logical keys; the API can map the same\n        // document to a distinct materialized URI after acknowledgement.\n        self.refresh_acknowledged_editor_snapshot(uri)?;\n",
      "",
    );
  let client = fs
    .readFileSync(path.join(root, clientPath), "utf8")
    .replace("use corsa::runtime::block_on;\n", "");
  const begin = client.indexOf(
    "    /// An attached API snapshot must follow the acknowledged editor overlay.\n",
  );
  const end = client.indexOf("    /// Answer a hover through the editor LSP transport", begin);
  assert.ok(begin >= 0 && end > begin);
  client = client.slice(0, begin) + client.slice(end);
  const tests = client.indexOf("\n#[cfg(all(test, unix))]\nmod initial_snapshot_tests {");
  assert.ok(tests > 0);
  client = client.slice(0, tests);
  const before = new Map([
    [initialPath, Buffer.from(initial)],
    [clientPath, Buffer.from(client)],
    [row.lspCase.path, null],
  ]);
  const beforeTypechecker = structuredClone(manifest);
  beforeTypechecker.protocolCases = beforeTypechecker.protocolCases.filter(
    (item) => item.id !== ID,
  );
  before.set(
    "tests/_fixtures/differential/typechecker/manifest.json",
    Buffer.from(JSON.stringify(beforeTypechecker)),
  );
  const beforeLsp = JSON.parse(
    fs.readFileSync(path.join(root, "tests/_fixtures/differential/lsp/manifest.json"), "utf8"),
  );
  beforeLsp.cases = beforeLsp.cases.filter(
    (item) => item.id !== "lsp/regression/shared-api-snapshot-reopen-10730",
  );
  before.set(
    "tests/_fixtures/differential/lsp/manifest.json",
    Buffer.from(JSON.stringify(beforeLsp)),
  );
  return (side, name) => {
    if (overrides.has(`${side}:${name}`)) return overrides.get(`${side}:${name}`);
    if (side === "base" && before.has(name)) return before.get(name);
    try {
      return fs.readFileSync(path.join(root, name));
    } catch (error) {
      if (error.code === "ENOENT") return null;
      throw error;
    }
  };
}

void test("only both reviewed source pairs and the full ordinary LSP producer register this API fix", () => {
  const accepted = newSharedEditorInitialApiCases(undefined, [row], reader());
  assert.deepEqual(accepted.ids, [ID]);
  assert.deepEqual(
    [...accepted.sources],
    row.sources.map((source) => source.path),
  );
  assert.equal(accepted.sources.has("crates/vize_canon/src/other.rs"), false);
  const changed = new Map([[`head:${row.sources[0].path}`, Buffer.from("another ordinary edit")]]);
  const ordinary = newSharedEditorInitialApiCases(undefined, undefined, reader(changed));
  assert.deepEqual(ordinary.ids, []);
  assert.equal(ordinary.sources.size, 0);
});

void test("exact API source pairs cannot use missing, stale, duplicate or foreign rows", () => {
  for (const [before, after] of [
    [undefined, undefined],
    [[row], [row]],
    [undefined, [row, row]],
    [undefined, [{ ...row, id: "foreign" }]],
  ])
    assert.throws(() => newSharedEditorInitialApiCases(before, after, reader()));
  assert.throws(() =>
    newSharedEditorInitialApiCases(
      undefined,
      [{ ...row, adapter: "batch-typechecker-fixture-v1" }],
      reader(),
    ),
  );
});

void test("ordinary inputs cannot replace the pinned wire, image authority or complete producer", () => {
  for (const target of [
    row.sources[0].path,
    row.sources[1].path,
    row.lspCase.path,
    row.wireContract.path,
    ...Object.values(row.producer)
      .filter((ref) => ref.path)
      .map((ref) => ref.path),
  ]) {
    const overrides = new Map([[`head:${target}`, Buffer.from("changed or absent authority")]]);
    assert.throws(
      () => newSharedEditorInitialApiCases(undefined, [row], reader(overrides)),
      target,
    );
  }
  const overrides = new Map([
    [
      "head:tests/_fixtures/differential/lsp/manifest.json",
      Buffer.from(JSON.stringify({ cases: [] })),
    ],
  ]);
  assert.throws(() => newSharedEditorInitialApiCases(undefined, [row], reader(overrides)));
  const sdk = structuredClone(row);
  sdk.producer.sdk.sha256 = "0".repeat(64);
  assert.throws(() => newSharedEditorInitialApiCases(undefined, [sdk], reader()));
});

void test("shared report requires the reviewed API row even with ordinary filler and keeps mixed Canon requirements", () => {
  const paths = row.sources.map((source) => source.path);
  const analyze = (readFile, changed = paths) =>
    analyzeLegacyFix({ title: "fix(lsp): acknowledge shared snapshots", paths: changed, readFile });
  const accepted = analyze(reader());
  assert.ok(accepted.products.length > 0);
  assert.ok(accepted.products.every((product) => product.state === "new-inputs-registered"));
  assert.ok(accepted.products.some((product) => product.cases.includes(ID)));
  const absent = structuredClone(manifest);
  absent.protocolCases = absent.protocolCases.filter((item) => item.id !== ID);
  absent.cases.push({
    ...structuredClone(absent.cases[0]),
    id: "typechecker/ordinary-structural-control",
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
  const mixed = analyze(reader(), [...paths, "crates/vize_canon/src/batch/unrelated.rs"]);
  assert.ok(mixed.products.some((product) => product.state === "new-input-missing"));
  const foreign = structuredClone(manifest);
  foreign.protocolCases.push({ ...structuredClone(row), id: "lsp/unknown-api-contract" });
  const unknown = analyze(
    reader(
      new Map([
        [
          "head:tests/_fixtures/differential/typechecker/manifest.json",
          Buffer.from(JSON.stringify(foreign)),
        ],
      ]),
    ),
  );
  assert.ok(unknown.products.some((product) => product.state === "invalid-manifest-or-input"));
});
