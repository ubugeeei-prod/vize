import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { loadFormatterApiManifest } from "../differential/formatter-api.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const directory = path.join(root, "tests/_fixtures/differential/formatter-history");

void test("complete literal references retain explicit Vue selectors and source-bound options", (t) => {
  for (const [name, count] of [
    ["literal", 84],
    ["literal-extra", 34],
    ["vue-version", 14],
  ]) {
    const loaded = loadFormatterApiManifest(path.join(directory, `${name}-manifest.json`), root);
    assert.equal(loaded.cases.length, count);
    assert(loaded.cases.every((item) => item.contract === "full-output-bytes-and-fixed-point"));
    assert(loaded.cases.every((item) => item.passCount === 3));
  }
  const manifestPath = path.join(directory, "vue-version-manifest.json");
  const loaded = loadFormatterApiManifest(manifestPath, root);
  const explicit = loaded.cases.filter((item) => item.vueVersion !== undefined);
  assert.equal(explicit.length, 13);
  for (const fixture of explicit) {
    assert.deepEqual(fixture.argv.slice(-2), ["--vue-version", fixture.vueVersion]);
    assert.deepEqual(fixture.optionsArgv.slice(-2), ["--vue-version", fixture.vueVersion]);
    assert.equal(fixture.effectiveOptions.vueVersion, fixture.vueVersion);
  }
  const implicit = loaded.cases.find((item) => item.id.endsWith("hyphenated-default-v3"));
  assert.deepEqual(implicit.argv, ["--template"]);
  assert(!Object.hasOwn(implicit.effectiveOptions, "vueVersion"));
  assert(
    implicit.expected.equals(
      loaded.cases.find((item) => item.id.endsWith("hyphenated-explicit-v3")).expected,
    ),
  );
  const crlf = loaded.cases.find((item) => item.id.endsWith("filter-chain-crlf-v2"));
  assert.equal(crlf.effectiveOptions.endOfLine, "crlf");
  assert(crlf.expected.includes(Buffer.from("\r\n")));

  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "formatter-version-contract-"));
  t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  for (const mutate of [
    (manifest) => {
      manifest.cases[0].vueVersion = "4";
    },
    (manifest) => {
      manifest.cases[0].api = "format_json";
    },
    (manifest) => {
      manifest.cases[0].witness.sourceSha256 = "0".repeat(64);
    },
  ]) {
    const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
    mutate(manifest);
    const file = path.join(temporary, "manifest.json");
    fs.writeFileSync(file, JSON.stringify(manifest));
    assert.throws(() => loadFormatterApiManifest(file, root));
  }
});
