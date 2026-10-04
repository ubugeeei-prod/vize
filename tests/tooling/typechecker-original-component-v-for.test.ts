import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { sha256 } from "../differential/harness.mjs";
import {
  loadTypecheckerManifest,
  validateTypecheckerCapture,
  type Capture,
} from "../differential/typechecker.ts";

const directory = fileURLToPath(new URL("../_fixtures/differential/typechecker/", import.meta.url));
const base = path.join(directory, "v-for-component-source-original");
const read = (file: string) => fs.readFileSync(path.join(base, file));
const reference = (file: string) => fs.readFileSync(path.join(directory, file));
const proof = JSON.parse(read("input-provenance.json").toString());
const fix = "04aedfb8e8b41dfae89fe65256e2703fc3718cf3";
const loaded = loadTypecheckerManifest(path.join(directory, "manifest.json"));
const planned = loaded.cases.filter(
  (fixture) => fixture.pack === "v-for-component-source-original",
);

test("component callback history retains the whole original function, child and exact options", () => {
  assert.equal(proof.regressionCommit, fix);
  assert.equal(proof.parent, "b77e0a21485b9c9fb23506ee9b30fc857e4345a1");
  assert.equal(proof.historicalIssue, 3756);
  assert.equal(proof.historicalPullRequest, 3818);
  assert.equal(proof.originalTestSource.referenceRoot, "typechecker");
  assert.equal(proof.originalTestSource.carrier, "v-for-source-original/original-test.rs.txt");
  assert.equal(proof.originalTestSource.blob, "64432e2a79f44519b47f4466cf8a5bd0818a6bed");
  assert.equal(proof.originalTestSource.revision, fix);
  const original = reference(proof.originalTestSource.carrier);
  assert.equal(
    sha256(original),
    "3d33956b4de2a0af20d1274dd2093f959600df6fefea9f81650158650d3e3e1d",
  );
  assert.equal(sha256(original), proof.originalTestSource.sha256);
  assert.deepEqual(proof.selectedFunction, {
    utf8Start: 1206,
    utf8End: 2265,
    sha256: "44f8ba167367e6ff34d14e89aabe6f1eca616037602850c5ca5b87beaabe5634",
  });
  assert.equal(sha256(original.subarray(1206, 2265)), proof.selectedFunction.sha256);
  assert(original.subarray(1206, 2265).toString().startsWith(`fn ${proof.originalTest}()`));
  assert.equal(proof.projectHelperSource.blob, "a7c74b9593646b124c753b35fa395f5bc215b9b6");
  assert.equal(proof.projectHelperSource.revision, fix);
  assert.equal(
    proof.projectHelperSource.sha256,
    "45997a074c644c38d72b013d6d2cad889a3a21d2a189669ff0bb2c2088391489",
  );
  assert.deepEqual(proof.projectHelperSource.excerpt, {
    carrier: "v-for-source-original/original-helper.rs.txt",
    utf8Start: 50636,
    utf8End: 61048,
    sha256: "b3beb7d333a17aa20c66d8be50cb26b1414d5b4fc3810f3250c56e3b1d98baab",
    referenceRoot: "typechecker",
  });
  const helper = reference(proof.projectHelperSource.excerpt.carrier);
  assert.equal(helper.length, 61048 - 50636);
  assert.equal(sha256(helper), proof.projectHelperSource.excerpt.sha256);
  const expectedInputs = [
    ["src/Child.vue", 1508, "ecc30a8300c920c41da4d185e2859b184c4433f77584e1779b4cb67c8af62c9f"],
    ["src/App.vue", 1660, "d3d4597988e701d3aed3f7c1edbcec9c69e9ec3b565044aacb32ae052dfd2b13"],
    ["tsconfig.json", null, "1c527226a15d6c5b6204ece1efc333308251f5e6c21680b355aaaf811d333047"],
  ];
  assert.deepEqual(
    proof.inputs.map(
      (input: { file: string; originalLiteralUtf8Start: number | null; sha256: string }) => [
        input.file,
        input.originalLiteralUtf8Start,
        input.sha256,
      ],
    ),
    expectedInputs,
  );
  assert.equal(proof.configLiteralUtf8Start, 52426);
  for (const input of proof.inputs) {
    assert.equal(input.source, `original-project/${input.file}.txt`);
    const bytes = read(input.source);
    assert.equal(sha256(bytes), input.sha256);
    const config = input.file === "tsconfig.json";
    const owner = config ? helper : original;
    const offset = config ? 52426 - 50636 : input.originalLiteralUtf8Start;
    assert.deepEqual(bytes, owner.subarray(offset, offset + bytes.length));
  }
  assert.deepEqual(JSON.parse(read("original-project/tsconfig.json.txt").toString()), {
    compilerOptions: {
      strict: true,
      target: "ES2022",
      module: "ESNext",
      moduleResolution: "bundler",
      noEmit: true,
    },
    include: ["src/**/*"],
  });
  assert.equal(proof.originalCheckerOptions, "default BatchTypeChecker; no extra checker flags");
  assert.equal(proof.reporter.login, "ubugeeei");
  assert.equal(proof.reporter.id, 71201308);
  assert.equal(proof.reporter.publicProfileEmail, null);
  assert.equal(proof.reporter.publicCommitEmail, "ubuge1122@gmail.com");
  assert.equal(proof.reporter.emailSource, `https://github.com/ubugeeei-prod/vize/commit/${fix}`);
});

test("the complete scoped historical patch reuses byte-exact authenticated carriers", () => {
  const expected = [
    [
      "v-for-source-original/original-change-1.patch.txt",
      0,
      11926,
      "d036fb589a0986653cc6c0181124d498f8271b89e9e0c882d4794731b1f9610e",
    ],
    [
      "v-for-source-original/original-change-2.patch.txt",
      11926,
      25727,
      "28e63da1f0643fdb2a0f76e08b5095896d414e7061fca5c5208fa8405f6cef51",
    ],
  ];
  assert.deepEqual(
    proof.originalChange.chunks.map(
      (chunk: { carrier: string; utf8Start: number; utf8End: number; sha256: string }) => [
        chunk.carrier,
        chunk.utf8Start,
        chunk.utf8End,
        chunk.sha256,
      ],
    ),
    expected,
  );
  const chunks = proof.originalChange.chunks.map(
    (chunk: {
      referenceRoot: string;
      carrier: string;
      utf8Start: number;
      utf8End: number;
      sha256: string;
    }) => {
      assert.equal(chunk.referenceRoot, "typechecker");
      const bytes = reference(chunk.carrier);
      assert.equal(bytes.length, chunk.utf8End - chunk.utf8Start);
      assert.equal(sha256(bytes), chunk.sha256);
      return bytes;
    },
  );
  assert.equal(
    sha256(Buffer.concat(chunks)),
    "892e5418ad74c6d99c4006ace43c2ef1f457bd01904977a8f8e25d9ebed0922c",
  );
  assert.equal(sha256(Buffer.concat(chunks)), proof.originalChange.sha256);
});

test("the complete original callback vector retains authored mapping and whole child context", () => {
  const pack = JSON.parse(read("cases.json").toString());
  assert.equal(planned.length, 1);
  assert.equal(planned[0].case, "complete-original-component-callback");
  assert.equal(pack.historicalIssue, 3818);
  assert.equal(pack.sourceRevision, fix);
  assert.equal(planned[0].sourceRevision, fix);
  assert.equal(planned[0].adapters.native, null);
  assert.equal(pack.checkerOptions, undefined);
  assert.equal(pack.projectOptions, undefined);
  const selected = reference(proof.originalTestSource.carrier).subarray(1206, 2265).toString();
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
      line: 7,
      column: 38,
      severity: 1,
      code: 7006,
      message: "Parameter 'value' implicitly has an 'any' type.",
    },
  ]);
  assert.deepEqual(planned[0].diagnostics, pack.cases[0].diagnostics);
  assert.equal(planned[0].inputs.length, 3);
  assert.deepEqual(proof.authoredParameter, {
    utf8Start: 138,
    authoredIdentifier: "value",
    positionContract: "one-based authored UTF16 start; end not historically observed",
  });
  const app = read("original-project/src/App.vue.txt");
  assert.equal(app.subarray(138, 143).toString(), "value");
  const lines = app.subarray(0, 138).toString().split("\n");
  assert.equal(lines.length, 7);
  assert.equal(lines.at(-1)!.length + 1, 38);
  const helper = reference(proof.projectHelperSource.excerpt.carrier)
    .toString()
    .split("fn snapshot_project_diagnostics(")[1]
    .split("fn link_workspace_node_modules(")[0];
  assert(helper.includes(".diagnostics\n        .into_iter()"));
  assert(helper.includes("snapshot.sort();"));
  assert(!helper.includes(".filter("));
  assert.equal(pack.diagnosticContract.comparison, "all production diagnostics in returned order");
  assert.equal(pack.diagnosticContract.requiredTier, "T1");
  assert.equal(proof.native, "unsupported");
  assert.equal(proof.execution, "pending fresh required T1 source-built Actions");
  assert.deepEqual(proof.missingFields, ["end", "relatedInformation", "raw backend diagnostics"]);
});

test("synthetic component transport controls refuse mapped-field changes, duplicates and lost context", () => {
  // These objects are rejection controls, never an actual runtime observation.
  const fixture = planned[0];
  const receipt = Buffer.from("synthetic component rejection receipt");
  const binarySha256 = sha256(Buffer.from("synthetic executable"));
  const control: Capture = {
    schema: "vize.typechecker-fixture-observation",
    version: 1,
    pack: "v-for-component-source-original",
    test: fixture.test,
    fixturePackSha256: fixture.packReference.sha256,
    archiveReceiptSha256: sha256(receipt),
    binaryPath: "/synthetic/executable",
    binarySha256,
    missingFields: ["end", "relatedInformation", "raw backend diagnostics"],
    matchedContract: "batch-start-diagnostics-v1",
    unbaselinedFields: ["diagnostics[].blockType", "exitCode", "success"],
    cases: [
      {
        id: fixture.case,
        projectRoot: "/synthetic/project",
        inputs: fixture.inputs.map((input) => ({ file: input.file, sha256: input.sha256 })),
        diagnostics: structuredClone(fixture.diagnostics),
        publicResult: {
          exitCode: 1,
          success: false,
          diagnostics: fixture.diagnostics.map((diagnostic) => ({
            ...diagnostic,
            file: `/synthetic/project/${diagnostic.file}`,
            line: diagnostic.line - 1,
            column: diagnostic.column - 1,
            blockType: "template",
          })),
        },
      },
    ],
  };
  validateTypecheckerCapture(loaded, control, { receipt, binarySha256 });
  const changes = [
    (value: Capture) => {
      value.cases[0].diagnostics[0].line = 6;
    },
    (value: Capture) => {
      value.cases[0].diagnostics[0].column = 36;
    },
    (value: Capture) => {
      value.cases[0].diagnostics[0].file = "src/Child.vue";
    },
    (value: Capture) => {
      value.cases[0].diagnostics[0].message = "different callback";
    },
    (value: Capture) => {
      value.cases[0].diagnostics[0].code = 2339;
    },
    (value: Capture) => {
      value.cases[0].diagnostics[0].severity = 2;
    },
    (value: Capture) => {
      value.cases[0].diagnostics.push(value.cases[0].diagnostics[0]);
    },
    (value: Capture) => {
      value.cases[0].inputs.shift();
    },
    (value: Capture) => {
      value.cases[0].inputs[0].sha256 = "0".repeat(64);
    },
    (value: Capture) => {
      value.cases[0].publicResult.diagnostics[0].column = 35;
    },
  ];
  for (const change of changes) {
    const changed = structuredClone(control);
    change(changed);
    assert.throws(() => validateTypecheckerCapture(loaded, changed, { receipt, binarySha256 }));
  }
});
