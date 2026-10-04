import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { sha256 } from "../differential/harness.mjs";
import { loadTypecheckerManifest } from "../differential/typechecker.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const directory = path.join(root, "tests/_fixtures/differential/typechecker");
const base = path.join(directory, "v-for-source-original");
const read = (file: string) => fs.readFileSync(path.join(base, file));
const proof = JSON.parse(read("input-provenance.json").toString());
const fix = "04aedfb8e8b41dfae89fe65256e2703fc3718cf3";

test("HTML callback history retains the full original function and its exact project configuration", () => {
  const original = read("original-test.rs.txt");
  const helper = read("original-helper.rs.txt");
  assert.equal(proof.regressionCommit, fix);
  assert.equal(proof.parent, "b77e0a21485b9c9fb23506ee9b30fc857e4345a1");
  assert.equal(proof.originalTestSource.blob, "64432e2a79f44519b47f4466cf8a5bd0818a6bed");
  assert.equal(proof.originalTestSource.revision, fix);
  assert.equal(
    sha256(original),
    "3d33956b4de2a0af20d1274dd2093f959600df6fefea9f81650158650d3e3e1d",
  );
  assert.equal(sha256(original), proof.originalTestSource.sha256);
  assert.deepEqual(proof.selectedFunction, {
    utf8Start: 196,
    utf8End: 1197,
    sha256: "08947ad0613edc2fecf95303674564f86810f1d325f3dec666b6be3b578f523b",
  });
  const selected = original.subarray(196, 1197);
  assert.equal(sha256(selected), proof.selectedFunction.sha256);
  assert(selected.toString().startsWith(`fn ${proof.originalTest}()`));
  assert(selected.toString().includes("vue-tsc 3.3.4 with TypeScript 6.0.3"));
  assert.equal(proof.projectHelperSource.blob, "a7c74b9593646b124c753b35fa395f5bc215b9b6");
  assert.equal(proof.projectHelperSource.revision, fix);
  assert.equal(
    proof.projectHelperSource.sha256,
    "45997a074c644c38d72b013d6d2cad889a3a21d2a189669ff0bb2c2088391489",
  );
  assert.deepEqual(proof.projectHelperSource.excerpt, {
    carrier: "original-helper.rs.txt",
    utf8Start: 50636,
    utf8End: 61048,
    sha256: "b3beb7d333a17aa20c66d8be50cb26b1414d5b4fc3810f3250c56e3b1d98baab",
  });
  assert.equal(helper.length, 61048 - 50636);
  assert.equal(sha256(helper), proof.projectHelperSource.excerpt.sha256);
  const diagnosticHelper = helper
    .toString()
    .split("fn snapshot_project_diagnostics(")[1]
    .split("fn link_workspace_node_modules(")[0];
  assert(diagnosticHelper.includes(".diagnostics\n        .into_iter()"));
  assert(diagnosticHelper.includes("snapshot.sort();"));
  assert(!diagnosticHelper.includes(".filter("));
  assert.equal(proof.originalCheckerOptions, "default BatchTypeChecker; no extra checker flags");
  const expectedInputs = [
    [
      "src/App.vue",
      "original-project/src/App.vue.txt",
      483,
      "7fc0b2dfdbe12dda3f1cf7d43e50895789c0bbf6c2b6bee5b18b77951a466830",
    ],
    [
      "tsconfig.json",
      "original-project/tsconfig.json.txt",
      null,
      "1c527226a15d6c5b6204ece1efc333308251f5e6c21680b355aaaf811d333047",
    ],
  ];
  assert.deepEqual(
    proof.inputs.map(
      (input: {
        file: string;
        source: string;
        originalLiteralUtf8Start: number | null;
        sha256: string;
      }) => [input.file, input.source, input.originalLiteralUtf8Start, input.sha256],
    ),
    expectedInputs,
  );
  assert.equal(proof.configLiteralUtf8Start, 52426);
  for (const input of proof.inputs) {
    const bytes = read(input.source);
    assert.equal(sha256(bytes), input.sha256);
    const config = input.file === "tsconfig.json";
    const owner = config ? helper : original;
    const offset = config ? 52426 - 50636 : input.originalLiteralUtf8Start;
    assert.deepEqual(bytes, owner.subarray(offset, offset + bytes.length));
  }
  const config = JSON.parse(read("original-project/tsconfig.json.txt").toString());
  assert.deepEqual(config, {
    compilerOptions: {
      strict: true,
      target: "ES2022",
      module: "ESNext",
      moduleResolution: "bundler",
      noEmit: true,
    },
    include: ["src/**/*"],
  });
  assert(!Object.hasOwn(config, "vueCompilerOptions"));
  assert(!Object.hasOwn(config, "extends"));
});

test("complete ordered patch fragments preserve the original Canon and Croquis changes", () => {
  const chunks = [
    {
      carrier: "original-change-1.patch.txt",
      utf8Start: 0,
      utf8End: 11926,
      sha256: "d036fb589a0986653cc6c0181124d498f8271b89e9e0c882d4794731b1f9610e",
    },
    {
      carrier: "original-change-2.patch.txt",
      utf8Start: 11926,
      utf8End: 25727,
      sha256: "28e63da1f0643fdb2a0f76e08b5095896d414e7061fca5c5208fa8405f6cef51",
    },
  ];
  assert.deepEqual(proof.originalChange.chunks, chunks);
  const bytes = chunks.map((chunk) => {
    const input = read(chunk.carrier);
    assert.equal(input.length, chunk.utf8End - chunk.utf8Start);
    assert.equal(sha256(input), chunk.sha256);
    assert(input.toString().split("\n").length - 1 <= 350);
    return input;
  });
  const patch = Buffer.concat(bytes);
  assert.equal(sha256(patch), "892e5418ad74c6d99c4006ace43c2ef1f457bd01904977a8f8e25d9ebed0922c");
  assert.equal(sha256(patch), proof.originalChange.sha256);
  assert(patch.toString().includes("diff --git a/crates/vize_canon/"));
  assert(patch.toString().includes("diff --git a/crates/vize_croquis/"));
});

test("the whole authored original diagnostic list is registered without native or missing-field credit", () => {
  const loaded = loadTypecheckerManifest(path.join(directory, "manifest.json"));
  const planned = loaded.cases.filter((fixture) => fixture.pack === "v-for-source-original");
  const pack = JSON.parse(read("cases.json").toString());
  assert.equal(planned.length, 1);
  assert.equal(pack.historicalIssue, 3818);
  assert.equal(planned[0].case, "complete-original-html-callback");
  assert.equal(planned[0].sourceRevision, fix);
  assert.equal(pack.sourceRevision, fix);
  assert.equal(planned[0].adapters.native, null);
  assert.equal(pack.diagnosticContract.native, "unsupported");
  assert.equal(pack.diagnosticContract.comparison, "all production diagnostics in returned order");
  assert.deepEqual(pack.diagnosticContract.missingFields, [
    "end",
    "relatedInformation",
    "raw backend diagnostics",
  ]);
  const selected = read("original-test.rs.txt").subarray(196, 1197).toString();
  const tuple =
    /Some\(vec!\[\(\s*vize_carton::String::from\("([^"]+)"\),\s*Some\((\d+)\),\s*vize_carton::String::from\("(\d+):(\d+):error ([^"]+)"\),\s*\)\]\)/.exec(
      selected,
    );
  assert(tuple);
  assert.deepEqual(planned[0].diagnostics, [
    {
      file: tuple[1],
      line: Number(tuple[3]),
      column: Number(tuple[4]),
      severity: 1,
      code: Number(tuple[2]),
      message: tuple[5],
    },
  ]);
  assert.deepEqual(planned[0].diagnostics, [
    {
      file: "src/App.vue",
      line: 6,
      column: 36,
      severity: 1,
      code: 7006,
      message: "Parameter 'value' implicitly has an 'any' type.",
    },
  ]);
  assert.deepEqual(planned[0].diagnostics, pack.cases[0].diagnostics);
  assert.equal(planned[0].inputs.length, 2);
  const source = read("original-project/src/App.vue.txt");
  assert.equal(proof.authoredParameter.utf8Start, 104);
  assert.equal(proof.authoredParameter.authoredIdentifier, "value");
  assert.equal(source.subarray(104, 109).toString(), "value");
  const lines = source.subarray(0, 104).toString().split("\n");
  assert.equal(lines.length, 6);
  assert.equal(lines.at(-1)!.length + 1, 36);
  assert.equal(proof.execution, "pending fresh required T1 source-built Actions");
  assert.equal(proof.native, "unsupported");
  assert.deepEqual(proof.missingFields, pack.diagnosticContract.missingFields);
  assert.deepEqual(proof.missingHistoricalState, [
    "actual historical loader/SDK snapshot",
    "returned-order identity for multi-row outputs",
  ]);
});
