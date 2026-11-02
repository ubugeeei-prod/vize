import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import {
  filesFor,
  reportedCommands,
  subsetFor,
  type RegressionProfile,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/contract.ts";
import {
  assessTsrs,
  capture,
  inputImages,
  recorder,
  type ProcessOutput,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/execute.ts";
import { fixtureNames } from "../../tools/benchmarks/scripts/public-typechecker-q16/fixtures.ts";
const profile: RegressionProfile = {
  subset: "C4",
  version: "0.645.0",
  source: "1".repeat(40),
  fix: "1".repeat(40),
  candidate: "2".repeat(40),
  run: "123",
  capsule: "3".repeat(40),
};
function temporary(run: (root: string) => void) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "q16-c4-")));
  try {
    run(root);
  } finally {
    rmSync(root, { recursive: true });
  }
}
const output = (status: number, stdout: string, stderr: string): ProcessOutput => ({
  status,
  stdout: Buffer.from(stdout),
  stderr: Buffer.from(stderr),
  signal: null,
  pid: 12,
  error: null,
  capacityLimited: false,
});
test("C4 covers all four original reports and exact sixteen frozen intended states", () => {
  const projects = subsetFor("C4").projects;
  assert.deepEqual(
    projects.map((row) => row.issue),
    [11362, 11363, 11366, 11367],
  );
  assert.deepEqual(
    projects.map((row) => row.states.map((state) => state.expectedTsrs.status)),
    [
      [0, 2, 0, 2],
      [2, 0, 2, 2],
      [0, 2, 0, 2],
      [0, 2, 0, 2],
    ],
  );
  assert.equal(
    projects.reduce((n, row) => n + row.states.length, 0),
    16,
  );
  assert.deepEqual(
    projects.map((row) => row.states[0]!.files.length),
    [3, 5, 4, 5],
  );
  assert.equal(fixtureNames.filter((name) => name.startsWith("fixtures-c4-")).length, 3);
  for (const project of projects) {
    assert.deepEqual(project.states[0]!.files, project.states[2]!.files);
    if (project.issue !== 11363)
      assert.deepEqual(project.states[1]!.files, project.states[3]!.files);
  }
  const dotted = projects[1]!.states;
  assert.notDeepEqual(dotted[0]!.files, dotted[3]!.files);
  assert.equal(
    dotted[0]!.expectedTsrs.stdout,
    "src/Probe.vue(6,14): error TS2322: Type 'string' is not assignable to type 'number'.\n",
  );
  assert.equal(
    dotted[3]!.expectedTsrs.stdout,
    "src/Probe.vue(6,8): error TS2322: Type 'string' is not assignable to type 'number'.\n",
  );
  assert.deepEqual(reportedCommands.tsrs, [
    "npx",
    "tsrs",
    "--runExternalCode",
    "--noEmit",
    "-p",
    "tsconfig.json",
  ]);
});
test("C4 instantiation changes only the registered public package version", () => {
  for (const project of subsetFor("C4").projects)
    for (const state of project.states) {
      const files = filesFor(state, profile);
      assert.equal(files.length, state.files.length);
      for (const [i, file] of files.entries())
        assert.equal(
          file.source,
          file.path === "package.json"
            ? state.files[i]!.source.replace('"vize": "0.638.0"', '"vize": "0.645.0"')
            : state.files[i]!.source,
        );
    }
});
test("all C4 whole intended streams precede exact-vector assessment and extra output refuses", () =>
  temporary((root) => {
    const record = recorder(root);
    let i = 0;
    for (const project of subsetFor("C4").projects)
      for (const state of project.states) {
        const expected = state.expectedTsrs,
          label = "state-" + i++,
          row = capture(
            record,
            label,
            ["inert"],
            root,
            () => {},
            () => output(expected.status, expected.stdout, expected.stderr),
          );
        assert.deepEqual(readFileSync(row.stdoutImage.path), Buffer.from(expected.stdout));
        assert.deepEqual(readFileSync(row.stderrImage.path), Buffer.from(expected.stderr));
        assessTsrs(state, row);
        for (const mutation of [
          { stdout: row.stdout + "unknown" },
          { stderr: "unknown" },
          { status: expected.status === 0 ? 2 : 0 },
          { wholeStreamsRetained: false },
        ])
          assert.throws(() => assessTsrs(state, { ...row, ...mutation }), assert.AssertionError);
      }
    assert.equal(i, 16);
  }));
test("each complete C4 source roster is physical and exact before callback", () =>
  temporary((root) => {
    for (const project of subsetFor("C4").projects) {
      const directory = join(root, String(project.issue)),
        evidence = join(root, "evidence-" + project.issue);
      mkdirSync(directory);
      mkdirSync(join(directory, "src"));
      mkdirSync(evidence);
      const files = filesFor(project.states[0]!, profile);
      for (const file of files) writeFileSync(join(directory, file.path), file.source);
      inputImages(directory, files);
      writeFileSync(join(directory, "src/Probe.vue"), "PRIVATE_C4_INPUT_CANARY");
      let calls = 0;
      const packet = capture(
        recorder(evidence),
        "refused",
        ["inert"],
        directory,
        () => {
          inputImages(directory, files);
        },
        () => {
          calls++;
          return output(0, "", "");
        },
      );
      assert.equal(calls, 0);
      assert.equal(packet.executed, false);
      assert(
        !readFileSync(join(evidence, "refused.process.json"), "utf8").includes(
          "PRIVATE_C4_INPUT_CANARY",
        ),
      );
    }
  }));
