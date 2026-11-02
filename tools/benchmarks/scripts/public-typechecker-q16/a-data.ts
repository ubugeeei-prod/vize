import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { asArray, asRecord, asString } from "../../../support/json-value.ts";
import type { Profile, SourceFile } from "./contract.ts";
export const a65FileName = "declaration-11365.json";
const value = asRecord(JSON.parse(readFileSync(new URL(a65FileName, import.meta.url), "utf8")));
assert.equal(value.schema, "vize.q16.public.declaration11365.v1");
assert.equal(value.fixMerge, "a321c37109d86c03afeacc833b1760c85ad4a32c");
assert.equal(
  asRecord(value.fixture).sha256,
  "36707b0c34eda5929ba17a98ff107b6ac7cbc3904432ae6bb914bcac0c133e73",
);
assert.equal(value.fullDeclarationByteGoldenOwned, false);
assert.equal(value.sourceDeclarationsUsedAsPublicInput, false);
export const a65Data = {
  fixMerge: asString(value.fixMerge),
  sourceAuthority: {
    fixture: value.fixture,
    independentSourceTest: value.independentSourceTest,
    renderers: value.renderers,
  },
  originalArgs: asArray(value.originalArgs).map((v) => asString(v)),
  jsonArgs: asArray(value.authoredJsonArgs).map((v) => asString(v)),
  outputFiles: asArray(value.outputFiles).map((v) => asString(v)),
  states: asArray(value.states).map((v) => {
    const row = asRecord(v);
    assert.equal(typeof row.json, "boolean");
    assert(row.status === 0 || row.status === 1);
    assert(row.expectedDiagnostic === null || typeof row.expectedDiagnostic === "string");
    return {
      id: asString(row.id),
      json: row.json as boolean,
      status: Number(row.status),
      expectedDiagnostic: row.expectedDiagnostic as string | null,
      files: asArray(row.files).map((v) => {
        const file = asRecord(v);
        return { path: asString(file.path), source: asString(file.source) };
      }),
    };
  }),
};
assert.deepEqual(
  a65Data.states.map((row) => row.status),
  [0, 0, 0, 0, 1, 1, 0],
);
assert.deepEqual(a65Data.outputFiles, ["Foo.vue.d.ts", "Probe.vue.d.ts", "__vize_helpers.d.ts"]);
assert.deepEqual(a65Data.originalArgs, [
  "npx",
  "vize",
  "check",
  "--declaration",
  "--declaration-dir",
  "out",
  "src/Foo.vue",
  "src/Probe.vue",
]);
export type A65State = (typeof a65Data.states)[number];
export function a65Inputs(state: A65State, profile: Profile): SourceFile[] {
  return state.files.map((file) => {
    assert(/^(package\.json|tsconfig\.json|src\/(Foo|Probe)\.vue)$/u.test(file.path));
    if (file.path !== "package.json") return { ...file };
    assert.equal(file.source.split('"vize": "0.638.0"').length, 2);
    return {
      path: file.path,
      source: file.source.replace('"vize": "0.638.0"', '"vize": "' + profile.version + '"'),
    };
  });
}
