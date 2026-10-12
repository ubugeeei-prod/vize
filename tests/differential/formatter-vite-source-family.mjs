import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const configPath = "npm/builder/vite/src/vite-plus/config.test.ts";
const archivePath =
  "tests/_fixtures/differential/config/vue-namespace/source-custody/original-vite-plus-config.test.ts.fixture";
const originalFamily = {
  "npm/builder/vite/src/vite-plus/import-sorting.test.ts": {
    bytes: 3809,
    sha256: "02b2910051683dca5108a47a622db63dc086f5a0a9b599b20ab33e043473832c",
  },
  [configPath]: {
    bytes: 9327,
    sha256: "82570d354aa7cfb9300bca05be49e7bd451694beea019b635cac1ccb7c8056a4",
  },
  "crates/vize_glyph/tests/fixtures/sort-imports/UserCard.vue": {
    bytes: 363,
    sha256: "ae86278aa0a0b247d9e027b703699f02cf370ea37da9dd96368adf3ff334b106",
  },
};
const currentConfig = {
  bytes: 9330,
  sha256: "0e6efef9ed98c4dfa593e692aadef9e3620c1f924d2c630deda6bfb8c070ddfd",
};
const oldAssertion = "assert.equal(withVue, defineConfig);";
const newAssertion = "assert.notEqual(withVue, defineConfig);";
const hash = (raw) => createHash("sha256").update(raw).digest("hex");
const matches = (raw, expected) => raw.length === expected.bytes && hash(raw) === expected.sha256;

function regularBytes(root, file, label) {
  const target = path.join(root, file);
  const mode = fs.lstatSync(target);
  assert(mode.isFile() && !(mode.mode & 0o111), `${label}: ${file} must be regular 100644`);
  return fs.readFileSync(target);
}

/** Admit only the complete original or declared API-successor input family. */
export function originalViteSourceFamily(root) {
  const actual = Object.fromEntries(
    Object.entries(originalFamily).map(([file, original]) => {
      const raw = regularBytes(root, file, "original Vite vector changed");
      assert(
        matches(raw, original) || (file === configPath && matches(raw, currentConfig)),
        `original Vite vector changed: ${file}`,
      );
      return [file, raw];
    }),
  );
  const archived = regularBytes(root, archivePath, "historical Vite vector changed");
  assert(matches(archived, originalFamily[configPath]), "historical Vite vector changed: config");
  const current = actual[configPath].toString();
  const original = archived.toString();
  if (matches(actual[configPath], currentConfig)) {
    assert.equal(current.split(newAssertion).length, 2, "ambiguous Vite API successor assertion");
    assert.equal(original.split(oldAssertion).length, 2, "ambiguous historical Vite assertion");
    assert.equal(
      current.replace(newAssertion, oldAssertion),
      original,
      "Vite API successor differs outside its one declared assertion",
    );
  } else {
    assert(
      actual[configPath].equals(archived),
      "historical config family differs from its archive",
    );
  }
  for (const file of [...Object.keys(actual), archivePath]) {
    const raw = file === archivePath ? archived : actual[file];
    assert(
      regularBytes(root, file, "Vite source changed during authentication").equals(raw),
      `Vite source changed during authentication: ${file}`,
    );
  }
  // The unchanged input-only VM receives the complete historical config body.
  return Object.fromEntries(
    Object.entries(actual).map(([file, raw]) => [
      file,
      (file === configPath ? archived : raw).toString(),
    ]),
  );
}
