import assert from "node:assert/strict";
import { readFileSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { asArray, asRecord, asString, parseJson } from "../../json-value.ts";
import { fileImage, sha256, inside } from "../computed_event_custody.ts";
import { fixturePins, fixtureSource } from "./fixture-pins.ts";
import { publicVersion } from "./authority.ts";

export type Row = {
  file: string;
  line: number;
  column: number;
  code: number;
  message: string;
  continuation: string[];
};
export type Kind = "number" | "any" | "error" | "unknown" | "never" | "string";
export type Stage = {
  population: "original10" | "imported7" | "registered15" | "collision16";
  id: string;
  files: Map<string, string>;
  expected: Row[];
  kind?: Kind;
  providerNames: string[];
  slotRoles: string[];
  collisionInvalid?: boolean;
  frameworkStdout?: string;
  repairCopy: boolean;
};
const historical = "tests/_fixtures/differential/typechecker/";
const original = historical + "unknown-component-slot-payload-11027";
const declared = historical + "declared-any-slot-provider-11027";
const owners = "tests/_fixtures/typechecker/slot-provider-owners-11027";
const payload =
  historical +
  "empty-vue-star-exports/malformed-membership/slot-provider-payload-update-11027.json";
const originalKinds: Record<string, Kind> = {
  valid: "number",
  unknown: "any",
  "unknown-union": "any",
  repaired: "number",
  "unresolved-tag": "error",
  "slot-any": "any",
  "slot-unknown": "unknown",
  "slot-never": "never",
  "slot-string": "string",
  "unicode-crlf-unknown": "any",
};
const importedKinds: Record<string, Kind> = {
  known: "number",
  "declared-any": "any",
  "error-any": "error",
  unknown: "any",
  "unresolved-tag": "error",
  repaired: "number",
  "unicode-crlf-any": "any",
};
const registeredKinds: Record<string, Kind> = {
  known: "number",
  "declared-any": "any",
  "error-any": "error",
  "missing-key": "error",
  "fallback-any": "any",
  "pascal-any-raw-typed": "any",
  "pascal-typed-raw-any": "number",
  "camel-any-only": "any",
  "camel-typed-only": "number",
  "camel-any-raw-typed": "any",
  "camel-typed-raw-any": "number",
  "pascal-any-camel-typed": "any",
  "pascal-typed-camel-any": "number",
  repaired: "number",
  "unicode-any": "any",
};
export function installedManifest(version: string) {
  assert.equal(version, publicVersion, "future SDK layouts require an explicit receiver successor");
  return {
    private: true,
    type: "module",
    dependencies: {
      vize: version,
      "@vizejs/native": version,
      vue: "3.5.42",
      typescript: "6.0.3",
      "typescript-native": "npm:typescript@7.1.0-beta",
      "vue-tsc": "3.3.13",
    },
    overrides: {
      "@vue/compiler-dom": "3.5.42",
      "@vue/compiler-core": "3.5.42",
      "@vue/compiler-sfc": "3.5.42",
      "@vue/shared": "3.5.42",
    },
  };
}
function expected(values: unknown): Row[] {
  return asArray(values).map((value) => {
    const row = asRecord(value);
    assert.deepEqual(
      Object.keys(row).sort(),
      (Object.hasOwn(row, "severity")
        ? ["file", "line", "column", "severity", "code", "message"]
        : ["file", "line", "column", "code", "message"]
      ).sort(),
    );
    if (Object.hasOwn(row, "severity")) assert.equal(row.severity, 1);
    for (const name of ["line", "column", "code"])
      assert.ok(Number.isSafeInteger(row[name]) && Number(row[name]) > 0);
    return {
      file: asString(row.file),
      line: Number(row.line),
      column: Number(row.column),
      code: Number(row.code),
      message: asString(row.message),
      continuation: [],
    };
  });
}
export function loadFixtures(directory: string) {
  const root = realpathSync(directory),
    bytes = new Map<string, string>();
  const custody = Object.entries(fixturePins).map(([file, expected]) => {
    const image = fileImage(join(root, file));
    assert.ok(inside(root, image.path));
    assert.equal(image.sha256, expected, file);
    const raw = readFileSync(image.path),
      text = new TextDecoder("utf-8", { fatal: true }).decode(raw);
    assert.equal(Buffer.from(text).equals(raw), true, "fixture UTF8 bytes must roundtrip exactly");
    bytes.set(file, text);
    return image;
  });
  const read = (file: string) => {
    assert.ok(bytes.has(file), "unregistered fixture: " + file);
    return bytes.get(file)!;
  };
  const stages: Stage[] = [];
  function cases(
    folder: string,
    population: Stage["population"],
    kinds: Record<string, Kind>,
    providerNames: string[],
  ) {
    const manifest = asRecord(
      parseJson(read(folder + (population === "registered15" ? "/controls.json" : "/cases.json"))),
    );
    const rows = asArray(manifest.cases).map((value) => asRecord(value));
    assert.deepEqual(
      rows.map((row) => row.id),
      Object.keys(kinds),
      "whole fixture order required",
    );
    for (const row of rows) {
      const id = asString(row.id),
        files = new Map<string, string>();
      let authored = "";
      for (const value of asArray(row.inputs)) {
        const input = asRecord(value),
          file = asString(input.file),
          source = asString(input.source);
        assert.ok(["component.ts", "globals.d.ts", "src/Tiny.vue", "tsconfig.json"].includes(file));
        assert.ok(!files.has(file));
        const text = read(folder + "/" + source);
        if (
          population === "imported7" &&
          ["component-any.ts.txt", "component-error.ts.txt"].includes(source)
        )
          assert.equal(
            text,
            read(owners + "/imported/" + source),
            "new imported owners preserve original bytes",
          );
        files.set(file, text);
        if (file === "src/Tiny.vue") authored = source;
      }
      assert.ok(files.has("src/Tiny.vue") && files.has("tsconfig.json"));
      const slotRoles =
        population === "registered15" && authored === "source-fallback-any.txt"
          ? ["template"]
          : ["template", "props"];
      stages.push({
        population,
        id,
        files,
        expected: expected(row.diagnostics),
        kind: kinds[id],
        providerNames,
        slotRoles,
        repairCopy: id === "repaired",
      });
    }
  }
  cases(original, "original10", originalKinds, []);
  cases(declared, "imported7", importedKinds, []);
  cases(owners + "/registered", "registered15", registeredKinds, [
    "RegistryChild",
    "registry_child",
  ]);
  const folder = owners + "/collisions",
    collisions = asArray(asRecord(parseJson(read(folder + "/controls.json"))).cases).map((value) =>
      asRecord(value),
    );
  assert.equal(collisions.length, 8);
  for (const control of collisions) {
    const id = asString(control.id),
      source = read(folder + "/" + id + "/source.vue.txt");
    const globals = read(folder + "/" + id + "/globals.d.ts.txt");
    assert.equal(sha256(source), control.sourceSha256);
    assert.equal(sha256(globals), control.globalsSha256);
    assert.equal(globals.split(": any;").length - 1, 1, "one exact declaration repair only");
    for (const repair of [false, true]) {
      const files = new Map([
        ["src/Tiny.vue", source],
        [
          "globals.d.ts",
          repair ? globals.replace(": any;", ": typeof import('./component').default;") : globals,
        ],
      ]);
      for (const file of ["component.ts", "types.ts", "tsconfig.json"])
        files.set(file, read(folder + "/" + file + ".txt"));
      stages.push({
        population: "collision16",
        id: id + (repair ? "/repair" : "/invalid"),
        files,
        expected: repair ? [] : expected(control.diagnostics),
        providerNames: ["A_b", "$Component_A_x2d_b", "$Component_A_x5f_b"],
        slotRoles: ["template", "props", "template", "props"],
        collisionInvalid: !repair,
        frameworkStdout: repair
          ? ""
          : read(folder + "/" + id + "/independent-framework.stdout.txt"),
        repairCopy: repair,
      });
    }
  }
  const helper = asRecord(asRecord(parseJson(read(payload))).logicalPreamble);
  const helperText = asString(helper.afterWholeUtf8);
  assert.equal(Buffer.byteLength(helperText), helper.afterBytes);
  assert.equal(sha256(helperText), helper.afterSha256);
  assert.deepEqual(
    ["original10", "imported7", "registered15", "collision16"].map(
      (population) => stages.filter((stage) => stage.population === population).length,
    ),
    [10, 7, 15, 16],
  );
  return { root, fixtureSource, custody, stages, helperText };
}
export function assertInputs(project: string, files: Map<string, string>) {
  return [...files].map(([file, text]) => {
    const raw = readFileSync(join(project, file));
    assert.equal(raw.equals(Buffer.from(text)), true, file);
    return { file, bytes: raw.length, sha256: sha256(raw) };
  });
}
