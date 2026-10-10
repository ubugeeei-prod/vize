import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { gunzipSync } from "node:zlib";
import { selectLibraryOracle, libraryVizeConfig } from "./library.ts";

const root = new URL(
  "../../../../_fixtures/differential/lsp/installed-event-slot-replay/library/",
  import.meta.url,
);
const manifest = JSON.parse(fs.readFileSync(new URL("manifest.json.txt", root), "utf8"));
const packet = () =>
  JSON.parse(gunzipSync(fs.readFileSync(new URL(manifest.cases[0].file, root))).toString("utf8"));
test("only the original workspace Corsa path is removed from the whole library configuration", () => {
  const original = packet();
  assert.doesNotThrow(() => selectLibraryOracle(original));
  assert.deepEqual(libraryVizeConfig, {
    typeChecker: {},
    lsp: { lint: true, typecheck: true, hover: true, crossFile: true },
  });
  for (const mutation of [
    (config: any) => {
      config.typeChecker.optionsApi = true;
    },
    (config: any) => {
      config.experimentals = { patternedTemplate: true };
    },
    (config: any) => {
      config.lsp.typecheck = false;
    },
    (config: any) => {
      delete config.typeChecker;
    },
  ]) {
    const changed = packet(),
      config = JSON.parse(changed.vizeConfig);
    mutation(config);
    changed.vizeConfig = JSON.stringify(config);
    assert.throws(() => selectLibraryOracle(changed));
  }
});
