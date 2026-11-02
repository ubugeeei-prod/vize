import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { asArray, asRecord, asString } from "../../../support/json-value.ts";
import { sha256, type Profile, type SourceFile } from "./contract.ts";

export const genericArrowFileName = "generic-arrow-data.json";
export interface GenericArrowState {
  id: string;
  files: SourceFile[];
  expectedTsc: { exit: "ZERO" | "NONZERO_DIAGNOSTIC"; stdout: string; stderr: string };
}
export const genericArrowFix = "ba01953b4429fabdd652d3497d89a8cf1737697b";
export const genericArrowArgv = ["--runExternalCode", "-p", "tsconfig.json", "--pretty", "false"];
export function genericArrowData() {
  const raw = readFileSync(new URL(genericArrowFileName, import.meta.url));
  assert.equal(
    sha256(raw),
    "4c1c3ea8fe2d54690183df463d74fc67a54ee381f5d7d24b2f8ad91fbf8403b8",
    "source intent bytes differ",
  );
  const value = asRecord(JSON.parse(raw.toString("utf8")));
  assert.equal(value.schema, "vize.genericArrowTsc.sourceIntent.v1");
  assert.equal(value.id, "GArrowTsc");
  assert.equal(asRecord(value.sourceAuthority).fixMerge, genericArrowFix);
  const states: GenericArrowState[] = asArray(value.cases).map((item) => {
    const row = asRecord(item),
      expected = asRecord(row.expectedTsc);
    assert(expected.exit === "ZERO" || expected.exit === "NONZERO_DIAGNOSTIC");
    return {
      id: asString(row.id),
      files: asArray(row.files).map((item) => {
        const file = asRecord(item);
        return { path: asString(file.path), source: asString(file.source) };
      }),
      expectedTsc: {
        exit: expected.exit,
        stdout: asString(expected.stdout),
        stderr: asString(expected.stderr),
      },
    };
  });
  assert.deepEqual(
    states.map((row) => row.id),
    [
      "original",
      "explicit-comma",
      "bad-return",
      "repair-return",
      "restore-bad-return",
      "bad-count",
      "repair-count",
      "comments-crlf",
      "nested",
      "same-line-template-ref",
    ],
  );
  for (const state of states)
    assert.deepEqual(
      state.files.map((file) => file.path),
      ["package.json", "tsconfig.json", "src/App.vue"],
    );
  assert.equal(
    sha256(states[0]!.files[2]!.source),
    "14d54e14493903d7f65a7b2d98865c70dd427801c15da860233a1a70f6c3cd30",
  );
  assert.deepEqual(states[0]!.files, states[3]!.files);
  assert.deepEqual(states[0]!.files, states[6]!.files);
  assert.deepEqual(states[2]!.files, states[4]!.files);
  return { sourceAuthority: value.sourceAuthority, states };
}
export function genericArrowFiles(state: GenericArrowState, profile: Profile) {
  assert.equal(profile.subset, "GArrowTsc");
  return state.files.map((file) => {
    if (file.path !== "package.json") return { ...file };
    const needle = '"vize": "0.649.0"';
    assert.equal(file.source.split(needle).length, 2);
    return {
      path: file.path,
      source: file.source.replace(needle, '"vize": "' + profile.version + '"'),
    };
  });
}
