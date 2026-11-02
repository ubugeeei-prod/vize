import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { asArray, asRecord, asString } from "../../../support/json-value.ts";
import { sha256, type Profile, type SourceFile } from "./contract.ts";

export const expressionDuplicatesFileName = "expression-duplicates-files.json";
export const expressionDuplicatesArgs = [
  "--runExternalCode",
  "-p",
  "tsconfig.json",
  "--pretty",
  "false",
];
export const expressionDuplicatesReferenceArgs = [
  "--noEmit",
  "-p",
  "tsconfig.json",
  "--pretty",
  "false",
];
export interface ExpressionState {
  id: string;
  files: SourceFile[];
  status: number;
  stdout: string;
}
export function expressionDuplicatesData() {
  const raw = readFileSync(new URL(expressionDuplicatesFileName, import.meta.url));
  assert.equal(
    sha256(raw),
    "bf8324d1f68e025b2d52ca8961e351a7a3433df1b89e308bffdbbe47e0f66e79",
    "source-owned expression data differs",
  );
  const data = asRecord(JSON.parse(new TextDecoder("utf8", { fatal: true }).decode(raw)));
  assert.equal(data.schema, "vize.expressionDuplicates11374.data.v1");
  const original: SourceFile[] = asArray(data.original).map((value) => {
    const row = asRecord(value),
      source = asString(row.source);
    assert.equal(sha256(source), row.sha256);
    return { path: asString(row.path), source };
  });
  assert.deepEqual(
    original.map((f) => f.path),
    [
      "package.json",
      "tsconfig.json",
      "src/Child.vue",
      "src/MultilineProp.vue",
      "src/MultilineVIf.vue",
      "src/NestedVIf.vue",
    ],
  );
  const states: ExpressionState[] = asArray(data.states).map((value) => {
    const row = asRecord(value),
      id = asString(row.id),
      files = original.map((f) => ({ ...f }));
    assert.equal(row.repairAll, ["repair-all", "independent-script-error"].includes(id));
    assert.equal(row.control, id === "independent-script-error");
    if (row.repairAll)
      for (const f of files.slice(3)) {
        assert.equal(f.source.split("items[0].label").length, 2);
        f.source = f.source.replace("items[0].label", "items[0]!.label");
      }
    if (row.control)
      files.push({ path: "src/Control.ts", source: "const wrong: string = 1; void wrong;\n" });
    assert.equal(sha256(JSON.stringify(files)), row.inputSha256);
    const stdout = asString(row.stdout);
    return { id, files, stdout, status: stdout === "" ? 0 : 2 };
  });
  assert.deepEqual(
    states.map((s) => s.id),
    ["original", "repair-all", "independent-script-error", "restored-original"],
  );
  assert.deepEqual(states[0]!.files, states[3]!.files);
  return { states, sourceAuthority: data.sourceAuthority };
}
export function expressionDuplicatesFiles(state: ExpressionState, profile: Profile) {
  assert.equal(profile.subset, "ExpressionDuplicates11374");
  return state.files.map((file) => {
    if (file.path !== "package.json") return { ...file };
    const needle = '"vize": "0.640.0"';
    assert.equal(file.source.split(needle).length, 2);
    return {
      path: file.path,
      source: file.source.replace(needle, '"vize": "' + profile.version + '"'),
    };
  });
}
