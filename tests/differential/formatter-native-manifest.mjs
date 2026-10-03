import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./manifest.mjs";
import { loadFormatterApiManifest } from "./formatter-api.mjs";
import { nativeFunction, originalViteSortingControls } from "./formatter-native-source.mjs";

export const NATIVE_FORMATTER_MANIFEST =
  "tests/_fixtures/differential/formatter-history/import-sorting-native-manifest.json";
const references = {
  "full-usercard": "usercard-enabled",
  false: "usercard-disabled",
  omitted: "usercard-omitted",
  "script-0": "usercard-enabled",
  "script-1": "custom-groups",
  "script-2": "side-effects-comments",
  "script-3": "descending-internal",
  "script-4": "newline-partitions",
  "invalid-boundary": "invalid-boundary",
};
export const nativeScriptWrapper = (script) => `<script setup lang="ts">\n${script}</script>\n`;
export function nativeScriptBody(code) {
  const start = code.indexOf("\n") + 1;
  const end = code.indexOf("</script>");
  assert(code.startsWith("<script ") && start > 0 && end > start);
  return code.slice(start, end);
}
export function nativeManifestAsset(root, declared) {
  assert(declared && typeof declared.path === "string");
  assert(!path.isAbsolute(declared.path) && !declared.path.split("/").includes(".."));
  const resolved = fs.realpathSync(path.join(root, declared.path));
  const relative = path.relative(fs.realpathSync(root), resolved);
  assert(relative && !relative.startsWith("..") && !path.isAbsolute(relative));
  const raw = fs.readFileSync(resolved);
  assert.equal(sha256(raw), declared.sha256, "public native fixture/source changed");
  return raw;
}
export function loadPublicNativeManifest(root, manifestPath = NATIVE_FORMATTER_MANIFEST) {
  const raw = fs.readFileSync(path.join(root, manifestPath));
  const manifest = JSON.parse(raw);
  assert.equal(manifest.schema, "vize.public-native-formatter-history");
  assert.equal(manifest.version, 1);
  assert.equal(manifest.issue, 6882);
  assert.equal(manifest.featureIssue, 7258);
  assert.equal(manifest.nativeHandled, 0);
  assert.deepEqual(manifest.runtimePolicy, { errorStackTraceLimit: 0 });
  nativeManifestAsset(root, manifest.apiManifest);
  const api = loadFormatterApiManifest(path.join(root, manifest.apiManifest.path), root);
  assert.equal(manifest.napiWitness.path, "crates/vize_vitrine/src/napi/format.rs");
  const napi = nativeManifestAsset(root, {
    path: manifest.napiWitness.path,
    sha256: manifest.napiWitness.sourceSha256,
  });
  assert.deepEqual(
    manifest.napiWitness.functions.map(({ name }) => name),
    ["native_sfc_sorting_uses_the_authored_full_reference", "format_sfc_napi"],
  );
  for (const witness of manifest.napiWitness.functions)
    assert.equal(sha256(nativeFunction(napi, witness.name)), witness.sha256);
  assert.equal(manifest.viteWitness.path, "npm/builder/vite/src/vite-plus/import-sorting.test.ts");
  const vite = nativeManifestAsset(root, {
    path: manifest.viteWitness.path,
    sha256: manifest.viteWitness.sourceSha256,
  });
  assert.equal(
    manifest.viteWitness.testTitle,
    "native SFC sorting matches complete script output from the pinned Oxfmt",
  );
  assert(vite.toString().includes(JSON.stringify(manifest.viteWitness.testTitle)));
  const usercard = api.cases.find(({ id }) => id === "import-sorting/usercard-enabled");
  const controls = originalViteSortingControls(vite, usercard.input.toString());
  const seen = new Set();
  const cases = manifest.cases.map((fixture) => {
    const id = fixture.id.replace(/^import-sorting-native\//, "");
    assert.equal(fixture.id, `import-sorting-native/${id}`);
    assert(Object.hasOwn(references, id) && !seen.has(id));
    seen.add(id);
    assert.equal(fixture.apiReference, `import-sorting/${references[id]}`);
    const reference = api.cases.find(({ id }) => id === fixture.apiReference);
    const input = nativeManifestAsset(root, fixture.input);
    const optionsRaw = nativeManifestAsset(root, fixture.options);
    const options = JSON.parse(optionsRaw);
    assert.equal(optionsRaw.toString(), `${JSON.stringify(options)}\n`);
    let expectedInput = reference.input.toString();
    let expectedOptions = reference.importSorting.provided
      ? { sortImports: reference.importSorting.setting }
      : {};
    const scriptIndex = id.startsWith("script-") ? Number(id.slice(7)) : null;
    if (scriptIndex !== null) {
      assert.equal(fixture.controlIndex, scriptIndex);
      expectedInput = nativeScriptWrapper(controls[scriptIndex][0]);
      expectedOptions = { sortImports: controls[scriptIndex][1] };
      assert.deepEqual(expectedOptions.sortImports, reference.importSorting.setting);
    } else assert.equal(fixture.controlIndex, undefined);
    assert.equal(input.toString(), expectedInput, "unbound original public source");
    assert.deepEqual(options, expectedOptions, "unbound original public options");
    if (id === "invalid-boundary") {
      assert.equal(fixture.outcome, "error");
      const errorRaw = nativeManifestAsset(root, fixture.expectedError);
      const expectedError = JSON.parse(errorRaw);
      const message = "Failed to format script: sortImports boundary must sit between two groups";
      assert.deepEqual(expectedError, {
        name: "Error",
        constructor: "Error",
        ownProperties: { code: "InvalidArg", message, stack: `Error: ${message}` },
      });
      assert.equal(fixture.expected, undefined);
      return { ...fixture, input, options, optionsRaw, errorRaw, expectedError };
    }
    assert.equal(fixture.outcome, "success");
    assert.equal(fixture.expectedError, undefined);
    const expected = nativeManifestAsset(root, fixture.expected);
    assert.equal(
      expected.toString(),
      scriptIndex === null
        ? reference.expected.toString()
        : nativeScriptWrapper(nativeScriptBody(reference.expected.toString())),
      "unbound complete public reference",
    );
    return { ...fixture, input, options, optionsRaw, expected };
  });
  assert.deepEqual([...seen].sort(), Object.keys(references).sort());
  return { manifest, manifestSha256: sha256(raw), cases };
}
