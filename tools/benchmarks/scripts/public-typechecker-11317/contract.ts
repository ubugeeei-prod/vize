import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { asArray, asRecord, asString, type JsonRecord } from "../../../support/json-value.ts";

export const profiles = {
  before: {
    version: "0.632.0",
    source: "1ee7b144458ac1bd0c07156ce24189e59ca6080a",
    candidate: "62ea08f12d21436b4844056625e64667e31b6ef9",
    run: "41305625974",
    capsule: "ecdf8e42455699888a706224b0a427113c5d6166",
  },
  after: {
    version: "0.634.0",
    source: "43ce7d085fdfe29e1fa53df5268aa65da63cdaac",
    candidate: "6d51ce5d2763d39e21c5a5d9795ccad72f15a5a2",
    run: "41318458779",
    capsule: "ebc8e7c422d0ba3ae7883eeefde48d33d6b38701",
  },
} as const;
export type Role = keyof typeof profiles;
export const fixturePath =
  "tests/_fixtures/differential/typechecker/package-import-redirects-11317/input.json";
export const inputHash = "13e0288f0d69e7be48366148023c8ec2324fd650e8396966ce46fabb1449d0b4";
export const ownerSource = profiles.after.source;
export const sha256 = (data: string | Buffer) => createHash("sha256").update(data).digest("hex");
export const cases = [
  "Original",
  "NoVersion",
  "Relative",
  "NoVue",
  "VueImporter",
  "Repair",
  "RestoreOriginal",
  "VueImporterRepair",
  "VueImporterRestore",
] as const;
export type Case = (typeof cases)[number];
export type SourceFile = { path: string; source: string };
export const sourcePins = [
  ["package.json", "08f4cfe109af9537ffdef766d938740269e964844097670dff0d4854ece09332"],
  ["tsconfig.json", "ee5fca5e89d5c8f4d98aeb6175b41eb702baec8e52c2d4e1b4725d8b35959c0d"],
  ["src/main.ts", "69b4fbb7f9c187371ef0eb580641636e9dcb47881d67de24de9527440a6183eb"],
  ["src/plain.ts", "268fe558ea66b8f47689eb3688cf0dda6c96366705b018e6256da5957f8e64d3"],
  ["src/Other.vue", "cdba2734bf2fe1c7db516518142c8e6085fa68cbf5519360641963e3d21e7fb9"],
] as const;
export function decodeInput(raw: Buffer): SourceFile[] {
  assert.equal(sha256(raw), inputHash);
  const value = asRecord(JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(raw)));
  assert.deepEqual(Object.keys(value).sort(), ["files", "id", "schema"]);
  assert.equal(value.schema, "vize.package-import-redirects");
  assert.equal(value.id, "typechecker/regression/package-import-redirects-11317");
  const files = asArray(value.files).map((value) => {
    const row = asRecord(value);
    assert.deepEqual(Object.keys(row).sort(), ["path", "source"]);
    return { path: asString(row.path), source: asString(row.source) };
  });
  assert.deepEqual(
    files.map((file) => [file.path, sha256(file.source)]),
    sourcePins,
  );
  return files;
}
export function input(): SourceFile[] {
  return decodeInput(
    readFileSync(fileURLToPath(new URL("../../../../" + fixturePath, import.meta.url))),
  );
}
export const vueImporter = (id: Case) =>
  id === "VueImporter" || id === "VueImporterRepair" || id === "VueImporterRestore";
export const repaired = (id: Case) => id === "Repair" || id === "VueImporterRepair";
export function sources(original: SourceFile[], role: Role, id: Case): SourceFile[] {
  const result = original.map((file) => {
    let source = file.source;
    if (file.path === "package.json") {
      source = source.replace('"vize": "0.632.0"', '"vize": "' + profiles[role].version + '"');
      if (id === "NoVersion") source = source.replace('  "version": "0.0.0",\n', "");
    }
    if (file.path === "src/main.ts") {
      if (id === "Relative") source = source.replace("#src/plain", "./plain");
      if (vueImporter(id)) source = "export {};\n";
    }
    if (file.path === "src/plain.ts") {
      if (id === "NoVue")
        source = source
          .replace('import Other from "./Other.vue";\n', "\n")
          .replace("Other, probe", "probe");
      if (repaired(id)) source = source.replace('"not a number"', "1");
    }
    return { path: file.path, source };
  });
  if (vueImporter(id))
    result.push({
      path: "src/Importer.vue",
      source:
        '<script setup lang="ts">\nimport { value } from "#src/plain";\nconst imported = value;\n</script>\n<template><p>{{ imported }}</p></template>\n',
    });
  return result;
}
export const diagnostic = "error:5:7 [TS2322] Type 'string' is not assignable to type 'number'.";
/** Independent authored law. BEFORE is a witness, never acceptance of the bug. */
export function expected(
  original: SourceFile[],
  id: Case,
  errors = repaired(id) ? 0 : 1,
): JsonRecord {
  const config = asRecord(JSON.parse(original[1]!.source));
  const paths = ["src/Other.vue", "src/main.ts", "src/plain.ts"];
  if (vueImporter(id)) paths.push("src/Importer.vue");
  paths.sort();
  return {
    files: paths.map((file) => ({
      file,
      diagnostics: file === "src/plain.ts" && errors ? [diagnostic] : [],
    })),
    programs: [
      {
        root: ".",
        tsconfig: "tsconfig.json",
        compilerOptions: config.compilerOptions,
        files: paths,
      },
    ],
    errorCount: errors,
    warningCount: 0,
    fileCount: paths.length,
  };
}
