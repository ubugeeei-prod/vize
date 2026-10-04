import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { sha256 } from "../differential/harness.mjs";
import { loadTypecheckerManifest } from "../differential/typechecker.ts";

const directory = fileURLToPath(new URL("../_fixtures/differential/typechecker/", import.meta.url));
const base = path.join(directory, "declared-prop-names-original");
const read = (file: string) => fs.readFileSync(path.join(base, file));
const proof = JSON.parse(read("input-provenance.json").toString());
const fix = "0f15c8b49d0630c1f2492c7ad8447ef4c9711534";

test("declared prop history retains all four original SFCs, configuration and public reporter", () => {
  const original = read("original-test.rs.txt");
  assert.equal(proof.regressionCommit, fix);
  assert.equal(proof.parent, "a78ed49185df9ec28df5f80b24699348c653d69d");
  assert.equal(proof.historicalIssue, 3863);
  assert.equal(proof.historicalPullRequest, 3869);
  assert.deepEqual(proof.reporter, {
    login: "AndreyYolkin",
    id: 11289484,
    name: "Andrei Elkin",
    publicEmail: "andrey@yolkin.co",
    identitySource: "https://api.github.com/users/AndreyYolkin",
  });
  assert.equal(proof.originalTestSource.blob, "cb438bd2c3cda580bfa6d16126b40c2ec2534ec8");
  assert.equal(proof.originalTestSource.revision, fix);
  assert.equal(
    sha256(original),
    "22228119bf5a52cedd48be2506eae9821d76968a18b633e5843fae321d08760d",
  );
  assert.equal(sha256(original), proof.originalTestSource.sha256);
  assert.deepEqual(proof.selectedFunction, {
    utf8Start: 594,
    utf8End: 2194,
    sha256: "8967c9a4faf87896a1008346f4e1f4a8bd535c79056e1a089da3a7695f7a2ebd",
  });
  assert.equal(sha256(original.subarray(594, 2194)), proof.selectedFunction.sha256);
  assert(original.toString().includes("vue-tsc 3.3.9 with TypeScript 6.x"));
  assert.equal(proof.projectHelperSource.blob, "a7c74b9593646b124c753b35fa395f5bc215b9b6");
  assert.equal(proof.projectHelperSource.revision, fix);
  assert.equal(
    proof.projectHelperSource.sha256,
    "45997a074c644c38d72b013d6d2cad889a3a21d2a189669ff0bb2c2088391489",
  );
  assert.deepEqual(proof.projectHelperSource.excerpt, {
    referenceRoot: "typechecker",
    carrier: "v-for-source-original/original-helper.rs.txt",
    utf8Start: 50636,
    utf8End: 61048,
    sha256: "b3beb7d333a17aa20c66d8be50cb26b1414d5b4fc3810f3250c56e3b1d98baab",
  });
  const helper = fs.readFileSync(path.join(directory, proof.projectHelperSource.excerpt.carrier));
  assert.equal(helper.length, 61048 - 50636);
  assert.equal(sha256(helper), proof.projectHelperSource.excerpt.sha256);
  const expectedInputs = [
    ["src/Child.vue", 882, "94b461657b2f03827686f0be2f5a45d387405e35b806d0574b9fd4c705630df5"],
    [
      "src/SnakeChild.vue",
      1106,
      "886757b3de64347c19bd3fa69a31b0da1628ebcc52da3e19b0bbd91de6cd19e2",
    ],
    [
      "src/KebabChild.vue",
      1305,
      "50cdfe2b4af57df9a807921a07d0fdef88b4a5917ed779b8a3a2a6e83c9d7acb",
    ],
    ["src/Parent.vue", 1502, "80c166665faa9ca5f6fb09fa6d81a7270bf27afa3c51a5370fec88512a81f31a"],
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
});

test("the complete scoped historical patch remains byte exact", () => {
  const patch = read("original-change.patch.txt");
  const pin = "6ba819f73104d99054e39cd53d11429b0b685d69196097b92ae32d362dacd2ef";
  assert.deepEqual(proof.originalChange.scope, ["crates/vize_canon", "crates/vize_croquis"]);
  assert.equal(patch.length, 9512);
  assert.equal(patch.toString().split("\n").length - 1, 252);
  assert.equal(sha256(patch), pin);
  assert.equal(proof.originalChange.sha256, pin);
  assert.deepEqual(proof.originalChange.chunks, [
    { carrier: "original-change.patch.txt", utf8Start: 0, utf8End: 9512, sha256: pin },
  ]);
  assert.equal((patch.toString().match(/^diff --git /gm) ?? []).length, 5);
});

test("the original whole empty vector is registered with all inputs and no native credit", () => {
  const loaded = loadTypecheckerManifest(path.join(directory, "manifest.json"));
  const planned = loaded.cases.filter((fixture) => fixture.pack === "declared-prop-names-original");
  const pack = JSON.parse(read("cases.json").toString());
  assert.equal(planned.length, 1);
  assert.equal(pack.historicalIssue, 3863);
  assert.equal(planned[0].case, "complete-original-declared-prop-names");
  assert.equal(planned[0].sourceRevision, fix);
  assert.equal(pack.sourceRevision, fix);
  assert.equal(pack.checkerOptions, undefined);
  assert.equal(pack.projectOptions, undefined);
  assert.equal(planned[0].adapters.native, null);
  assert.equal(pack.diagnosticContract.requiredTier, "T1");
  assert.equal(pack.diagnosticContract.native, "unsupported");
  assert.equal(pack.diagnosticContract.comparison, "all production diagnostics in returned order");
  assert.deepEqual(pack.diagnosticContract.missingFields, [
    "end",
    "relatedInformation",
    "raw backend diagnostics",
  ]);
  const selected = read("original-test.rs.txt").subarray(594, 2194).toString();
  assert(selected.startsWith(`fn ${proof.originalTest}()`));
  assert.match(selected, /assert_eq!\(\s*snapshot,\s*vec!\[\],/);
  assert.deepEqual(planned[0].diagnostics, []);
  assert.deepEqual(planned[0].diagnostics, pack.cases[0].diagnostics);
  assert.equal(planned[0].inputs.length, 5);
  const helper = fs
    .readFileSync(path.join(directory, proof.projectHelperSource.excerpt.carrier))
    .toString()
    .split("fn snapshot_project_diagnostics(")[1]
    .split("fn link_workspace_node_modules(")[0];
  assert(helper.includes(".diagnostics\n        .into_iter()"));
  assert(helper.includes("snapshot.sort();"));
  assert(!helper.includes(".filter("));
  assert.equal(proof.execution, "pending fresh required T1 source-built Actions");
  assert.equal(proof.native, "unsupported");
  assert.deepEqual(proof.missingFields, pack.diagnosticContract.missingFields);
  assert.deepEqual(proof.missingHistoricalState, ["actual historical loader/SDK snapshot"]);
});
