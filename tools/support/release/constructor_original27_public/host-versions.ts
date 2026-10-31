import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { asRecord, asString, parseJson } from "../../json-value.ts";
import { Recorder, sha256 } from "../../config/windows_consumer/common.ts";
import { fileImage, inside } from "../computed_event_custody.ts";

export const vueFrameworkPackages = [
  "@vue/compiler-core",
  "@vue/compiler-dom",
  "@vue/compiler-sfc",
  "@vue/compiler-ssr",
  "@vue/reactivity",
  "@vue/runtime-core",
  "@vue/runtime-dom",
  "@vue/server-renderer",
  "@vue/shared",
] as const;
export function assertVuePackageVersions(rows: { name: string; version: string }[]) {
  const names = rows.map((row) => row.name);
  assert.equal(new Set(names).size, rows.length, "Vue namespace package names must be unique");
  assert.deepEqual(
    [...names].sort(),
    [...vueFrameworkPackages, "@vue/language-core"].sort(),
    "exact frozen framework and language-tooling package names required",
  );
  for (const row of rows)
    assert.equal(row.version, row.name === "@vue/language-core" ? "3.3.13" : "3.5.42", row.name);
}
export function assertVueProviderLineage(overrides: unknown, official: unknown) {
  assert.deepEqual(
    overrides,
    {
      "@vue/compiler-dom": "3.5.42",
      "@vue/compiler-core": "3.5.42",
      "@vue/compiler-sfc": "3.5.42",
      "@vue/shared": "3.5.42",
    },
    "the four original dependency overrides must remain exact",
  );
  const vueTsc = asRecord(official);
  assert.equal(vueTsc.name, "vue-tsc");
  assert.equal(vueTsc.version, "3.3.13");
  assert.equal(
    asRecord(vueTsc.dependencies)["@vue/language-core"],
    "3.3.13",
    "language-core must follow the exact frozen VueTSC dependency lineage",
  );
}
export function captureVueNamespace(modules: string, record: Recorder) {
  const names = readdirSync(join(modules, "@vue")).sort();
  const rows = names.map((name) => {
    const path = join(modules, "@vue", name, "package.json"),
      raw = readFileSync(path);
    return {
      directory: name,
      path,
      bytes: raw.length,
      sha256: sha256(raw),
      rawBase64: raw.toString("base64"),
    };
  });
  // Keep every actual namespace row and its raw manifest before semantic checks.
  record.json("vue-namespace-inventory.json", rows);
  const parsed = rows.map((row) => {
    const manifest = asRecord(parseJson(Buffer.from(row.rawBase64, "base64").toString("utf8")));
    assert.equal(
      manifest.name,
      "@vue/" + row.directory,
      "Vue manifest name must match its actual directory",
    );
    return { name: asString(manifest.name), version: asString(manifest.version) };
  });
  assertVuePackageVersions(parsed);
  return rows.map((row) => {
    const image = fileImage(row.path);
    assert.ok(inside(modules, image.path), "Vue package escaped the public installation");
    assert.equal(image.bytes, row.bytes);
    assert.equal(image.sha256, row.sha256);
    return image;
  });
}
