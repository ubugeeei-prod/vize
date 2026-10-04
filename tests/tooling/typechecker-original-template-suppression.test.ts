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
const base = path.join(directory, "template-suppression-original");
const read = (file: string) => fs.readFileSync(path.join(base, file));
const proof = JSON.parse(read("input-provenance.json").toString());
const fix = "e199979dfb32dfd8eb9ad32a2928fb010768f239";
const loaded = loadTypecheckerManifest(path.join(directory, "manifest.json"));
const planned = loaded.cases.filter((fixture) => fixture.pack === "template-suppression-original");

test("suppression history retains the whole original function and exact strictTemplates project", () => {
  assert.equal(proof.regressionCommit, fix);
  assert.equal(proof.parent, "dcb5bae88876fdaf5015d6499b0eb288447b9371");
  assert.equal(proof.historicalIssue, 5998);
  assert.equal(proof.historicalPullRequest, 6009);
  assert.equal(proof.originalTestSource.blob, "dbbb3a87f4d634413272312ccbf6bf44640d4e55");
  assert.equal(proof.originalTestSource.revision, fix);
  const original = read(proof.originalTestSource.carrier);
  assert.equal(
    sha256(original),
    "698994f61ca3f5e523807fedeb5e12401169325dcdd84f4f124eba1f0e888a39",
  );
  assert.equal(sha256(original), proof.originalTestSource.sha256);
  assert.deepEqual(proof.selectedFunction, {
    utf8Start: 4559,
    utf8End: 5410,
    sha256: "da4584d4fbb22fc4b06118c3a438022ff6caff31f591e2cd5eb24e18e8e2dcfc",
  });
  assert.equal(sha256(original.subarray(4559, 5410)), proof.selectedFunction.sha256);
  assert(original.subarray(4559, 5410).toString().startsWith(`fn ${proof.originalTest}()`));
  assert.equal(proof.projectHelperSource.blob, "179e050bbc67907b6a20ef67ae362ba1022c4388");
  assert.equal(proof.projectHelperSource.revision, fix);
  assert.equal(
    proof.projectHelperSource.sha256,
    "0f29bfe01013d06a34178218cf8a60796995f797b40ff602b30a1fa5200d80fd",
  );
  assert.deepEqual(proof.projectHelperSource.excerpt, {
    carrier: "original-helper.rs.txt",
    utf8Start: 51034,
    utf8End: 59100,
    sha256: "431c5f5b4a76a5f5a704623916869a8faeedec2316a836d646d35b944b7c0e90",
  });
  const helper = read(proof.projectHelperSource.excerpt.carrier);
  assert.equal(helper.length, 8066);
  assert.equal(sha256(helper), proof.projectHelperSource.excerpt.sha256);
  assert.deepEqual(
    proof.inputs.map(
      (input: { file: string; originalLiteralUtf8Start: number | null; sha256: string }) => [
        input.file,
        input.originalLiteralUtf8Start,
        input.sha256,
      ],
    ),
    [
      ["src/App.vue", 4850, "c25ca8f918e9d83a8f6282cbab0c8a8d5970a5e81bd8de2ca566235c8a0ebeae"],
      ["tsconfig.json", null, "e36b280dce185f4be85cbff1838c0ee06d8d7a46b38b3e63df995c0decd7b13b"],
    ],
  );
  assert.equal(proof.configLiteralUtf8Start, 52824);
  for (const input of proof.inputs) {
    assert.equal(input.source, `original-project/${input.file}.txt`);
    const bytes = read(input.source);
    assert.equal(sha256(bytes), input.sha256);
    const config = input.file === "tsconfig.json";
    const owner = config ? helper : original;
    const offset = config ? 52824 - 51034 : input.originalLiteralUtf8Start;
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
    vueCompilerOptions: { strictTemplates: true },
    include: ["src/**/*"],
  });
  assert.equal(proof.originalCheckerOptions, "default BatchTypeChecker; no extra checker flags");
  assert.equal(proof.reporter.login, "ubugeeei");
  assert.equal(proof.reporter.id, 71201308);
  assert.equal(proof.reporter.publicProfileEmail, null);
  assert.equal(proof.reporter.publicCommitEmail, "ubuge1122@gmail.com");
  assert.equal(proof.reporter.emailSource, `https://github.com/ubugeeei-prod/vize/commit/${fix}`);
});

test("suppression custody retains the complete scoped original patch and original scanner laws", () => {
  const patch = read(proof.originalChange.carrier);
  assert.equal(patch.length, 7618);
  assert.equal(proof.originalChange.utf8Start, 0);
  assert.equal(proof.originalChange.utf8End, 7618);
  assert.deepEqual(proof.originalChange.scope, ["crates/vize_canon", "crates/vize_croquis"]);
  assert.equal(sha256(patch), "2ace1bc53de00b56ad8a89421367cf21d42e91fdd9c4951c2e0eb899c90c8bec");
  assert.equal(sha256(patch), proof.originalChange.sha256);
  assert.equal((patch.toString().match(/^diff --git /gm) ?? []).length, 4);
  assert(!patch.toString().includes("diff --git a/crates/vize_croquis"));
  assert(patch.toString().includes("ts_suppression_comments.rs"));
  // Retention of these historical laws does not prove their current runtime.
  assert.equal((patch.toString().match(/^\+    #\[test\]/gm) ?? []).length, 3);
  assert.deepEqual(proof.remainingObligations, [
    "v-if guard runtime behavior",
    "three original scanner laws independently qualified",
    "independent unused-expect-error diagnostic authority; no original vector supplied",
  ]);
});

test("the complete original suppression oracle is empty without filtering or invented spans", () => {
  const pack = JSON.parse(read("cases.json").toString());
  assert.equal(planned.length, 1);
  assert.equal(planned[0].case, "complete-original-template-suppression");
  assert.equal(pack.historicalIssue, 6009);
  assert.equal(pack.sourceRevision, fix);
  assert.equal(planned[0].sourceRevision, fix);
  assert.equal(planned[0].adapters.native, null);
  assert.equal(pack.checkerOptions, undefined);
  assert.equal(pack.projectOptions, undefined);
  const selected = read(proof.originalTestSource.carrier).subarray(4559, 5410).toString();
  assert(selected.includes("assert_eq!(\n        snapshot,\n        Some(Vec::new()),"));
  assert(selected.includes("snapshot_project_diagnostics(&project_root)"));
  assert(selected.includes("// @ts-ignore\n      items[0].missingProperty"));
  assert(selected.includes("// @ts-expect-error\n      items[0].alsoMissing"));
  assert.deepEqual(planned[0].diagnostics, []);
  assert.deepEqual(planned[0].diagnostics, pack.cases[0].diagnostics);
  assert.equal(planned[0].inputs.length, 2);
  const helper = read(proof.projectHelperSource.excerpt.carrier)
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

test("synthetic suppression transport refuses extra diagnostics, lost exact options and broken custody", () => {
  // Synthetic transport objects supply no original negative runtime oracle.
  const fixture = planned[0];
  const receipt = Buffer.from("synthetic suppression rejection receipt");
  const binarySha256 = sha256(Buffer.from("synthetic executable"));
  const control: Capture = {
    schema: "vize.typechecker-fixture-observation",
    version: 1,
    pack: "template-suppression-original",
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
        diagnostics: [],
        publicResult: { exitCode: 0, success: true, diagnostics: [] },
      },
    ],
  };
  validateTypecheckerCapture(loaded, control, { receipt, binarySha256 });
  const extra = {
    file: "src/App.vue",
    line: 9,
    column: 16,
    severity: 1,
    code: 2339,
    message: "synthetic unexpected row",
  };
  const changes = [
    (value: Capture) => {
      value.cases[0].diagnostics.push(extra);
    },
    (value: Capture) => {
      value.cases[0].inputs.pop();
    },
    (value: Capture) => {
      value.cases[0].inputs[1].sha256 = "0".repeat(64);
    },
    (value: Capture) => {
      value.cases[0].publicResult.diagnostics.push({
        ...extra,
        file: "/synthetic/project/src/App.vue",
        line: 8,
        column: 15,
        blockType: "template",
      });
    },
    (value: Capture) => {
      value.fixturePackSha256 = "0".repeat(64);
    },
    (value: Capture) => {
      value.archiveReceiptSha256 = "0".repeat(64);
    },
    (value: Capture) => {
      value.binarySha256 = "0".repeat(64);
    },
    (value: Capture) => {
      value.cases[0].id = "report-example";
    },
  ];
  for (const change of changes) {
    const changed = structuredClone(control);
    change(changed);
    assert.throws(() => validateTypecheckerCapture(loaded, changed, { receipt, binarySha256 }));
  }
});
