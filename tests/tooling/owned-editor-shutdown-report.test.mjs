import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { OWNED_EDITOR_SHUTDOWN_PROTOCOL_ID as ID } from "../../tools/support/compat/github/owned-editor-shutdown-fixtures.ts";
import { analyzeLegacyFix } from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const load = (name) => JSON.parse(fs.readFileSync(path.join(root, name), "utf8"));

void test("owned shutdown protocol credit stays with the admitted Canon source family", () => {
  const manifest = load("tests/_fixtures/differential/typechecker/manifest.json");
  const row = manifest.protocolCases.find((entry) => entry.id === ID);
  assert.ok(row);
  const before = load("tests/_fixtures/runtime/lsp-owned-shutdown-10760/reviewed-before.json");
  const base = new Map(
    before.sources.map((entry) => [
      entry.path,
      entry.rawBase64 === null ? null : Buffer.from(entry.rawBase64, "base64"),
    ]),
  );
  base.set(row.lspCase.path, null);
  const oldTypechecker = structuredClone(manifest);
  oldTypechecker.protocolCases = oldTypechecker.protocolCases.filter((entry) => entry.id !== ID);
  base.set(
    "tests/_fixtures/differential/typechecker/manifest.json",
    Buffer.from(JSON.stringify(oldTypechecker)),
  );
  const oldLsp = load("tests/_fixtures/differential/lsp/manifest.json");
  oldLsp.cases = oldLsp.cases.filter(
    (entry) => entry.id !== "lsp/regression/owned-outer-shutdown-10760",
  );
  base.set("tests/_fixtures/differential/lsp/manifest.json", Buffer.from(JSON.stringify(oldLsp)));
  const readFile = (side, name) => {
    if (side === "base" && base.has(name)) return base.get(name);
    try {
      return fs.readFileSync(path.join(root, name));
    } catch (error) {
      if (error.code === "ENOENT") return null;
      throw error;
    }
  };
  const analyze = (paths) =>
    analyzeLegacyFix({ title: "fix(lsp): await owned compiler shutdown", paths, readFile });
  const unrelated = analyze(["crates/vize_canon/src/batch/unrelated.rs"]);
  assert.equal(unrelated.products[0].state, "new-input-missing");
  assert.equal(unrelated.products[0].cases.includes(ID), false);
  const lsp = analyze(["crates/vize_maestro/src/server.rs"]);
  assert.equal(lsp.products[0].state, "new-inputs-registered");
  assert.deepEqual(lsp.products[0].cases, ["lsp/regression/owned-outer-shutdown-10760"]);
});
