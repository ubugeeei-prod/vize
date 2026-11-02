import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, realpathSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import {
  commandsFor,
  filesFor,
  profileFor,
  requireProfile,
  subsetFor,
  type RegressionProfile,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/contract.ts";
import {
  assessTsrs,
  capture,
  recorder,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/execute.ts";
import { requireOwnedDispatch } from "../../tools/benchmarks/scripts/public-typechecker-q16/run.ts";
import { fixtureNames } from "../../tools/benchmarks/scripts/public-typechecker-q16/fixtures.ts";
const mock: RegressionProfile = {
  subset: "F78",
  fix: "d1ef6f2200afaf18922a052c114bb9ed366846e5",
  source: "d1ef6f2200afaf18922a052c114bb9ed366846e5",
  candidate: "3".repeat(40),
  version: "0.647.0",
  run: "123",
  capsule: "4".repeat(40),
};
test("literal commented condition profile binds the genuine647 containing release and fetched capsule", () => {
  assert.deepEqual(profileFor(), {
    subset: "F78",
    fix: "d1ef6f2200afaf18922a052c114bb9ed366846e5",
    version: "0.647.0",
    source: "d1ef6f2200afaf18922a052c114bb9ed366846e5",
    candidate: "98318e082d05b6db0a6908da2a9cad6dcdf276f1",
    run: "41443199914",
    capsule: "f8719292de81466487e2c9a028d0ff22ae23f389",
  });
});
test("commented condition has one closed project and five independently authored states", () => {
  const selected = subsetFor("F78");
  assert.equal(selected.projects.length, 1);
  const project = selected.projects[0]!;
  assert.equal(project.issue, 11378);
  assert.deepEqual(
    project.states.map((row) => row.phase),
    ["original", "wrong-member", "repair", "restored-wrong", "restored-original"],
  );
  assert.deepEqual(
    project.states.map((row) => row.expectedTsrs.status),
    [0, 2, 0, 2, 0],
  );
  const originals = project.states[0]!.files;
  assert.deepEqual(project.states[2]!.files, originals);
  assert.deepEqual(project.states[4]!.files, originals);
  assert.deepEqual(project.states[1]!.files, project.states[3]!.files);
  const commented = originals.find((file) => file.path === "src/CommentInElseIf.vue")!.source;
  assert.equal(
    originals.find((file) => file.path === "src/NoComment.vue")!.source,
    commented.replace("        // comment\n", ""),
  );
  assert.equal(
    project.states[1]!.files.find((file) => file.path === "src/CommentInElseIf.vue")!.source,
    commented.replace("props.item.kind !== 'b'", "props.item.kind !== 'b' && Number(0).nope"),
  );
  assert(fixtureNames.includes("fixtures-f78-issue-11378.json"));
});
test("constructed package preserves the reported five pins and only vize changes for the public target", () => {
  for (const state of subsetFor("F78").projects[0]!.states) {
    const files = filesFor(state, mock);
    for (const [index, file] of files.entries())
      assert.equal(
        file.source,
        file.path === "package.json"
          ? state.files[index]!.source.replace('"vize": "0.641.0"', '"vize": "0.647.0"')
          : state.files[index]!.source,
      );
    const manifest = JSON.parse(state.files[0]!.source) as { devDependencies: unknown };
    assert.deepEqual(manifest.devDependencies, {
      vize: "0.641.0",
      tsrs: "0.20.1-ts7.1.0-beta",
      vue: "3.5.43",
      "vue-tsc": "3.2.6",
      typescript: "5.9.3",
    });
  }
  assert.equal(requireProfile(mock), mock);
  assert.throws(() => requireProfile(null), assert.AssertionError);
  assert.throws(() => requireProfile({ ...mock, fix: "f".repeat(40) }), assert.AssertionError);
});
test("npm npx adaptation preserves the exact original inner checker flags without fast or noEmit injection", () => {
  const commands = commandsFor("F78");
  assert.deepEqual(commands.install, ["npm", "i"]);
  assert.deepEqual(commands.tsrs, [
    "npx",
    "tsrs",
    "--runExternalCode",
    "-p",
    "tsconfig.json",
    "--pretty",
    "false",
  ]);
  assert.deepEqual(commands.reference, [
    "npx",
    "vue-tsc",
    "--noEmit",
    "-p",
    "tsconfig.json",
    "--pretty",
    "false",
  ]);
  assert.deepEqual(commandsFor("A59").tsrs, [
    "npx",
    "tsrs",
    "--runExternalCode",
    "--noEmit",
    "-p",
    "tsconfig.json",
  ]);
  requireOwnedDispatch({
    GITHUB_EVENT_NAME: "workflow_dispatch",
    GITHUB_REPOSITORY: "ubugeeei-prod/vize",
    GITHUB_REF: "refs/heads/test/published-commented-vif-11378-20261102",
  });
  assert.throws(
    () =>
      requireOwnedDispatch({
        GITHUB_EVENT_NAME: "workflow_dispatch",
        GITHUB_REPOSITORY: "ubugeeei-prod/vize",
        GITHUB_REF: "refs/heads/foreign",
      }),
    assert.AssertionError,
  );
});
test("all five complete streams are retained and only full independent vectors can match", () => {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "q16-f78-")));
  try {
    const record = recorder(root);
    for (const [index, state] of subsetFor("F78").projects[0]!.states.entries()) {
      const packet = capture(
        record,
        "state-" + index,
        ["inert"],
        root,
        () => {},
        () => ({
          status: state.expectedTsrs.status,
          signal: null,
          pid: 123,
          error: null,
          capacityLimited: false,
          stdout: Buffer.from(state.expectedTsrs.stdout),
          stderr: Buffer.from(state.expectedTsrs.stderr),
        }),
      );
      assessTsrs(state, packet);
      assert(readFileSync(packet.stdoutImage.path).equals(Buffer.from(state.expectedTsrs.stdout)));
      for (const change of [
        { status: state.expectedTsrs.status === 0 ? 2 : 0 },
        { stdout: packet.stdout + "extra" },
        { stderr: "extra" },
        { wholeStreamsRetained: false },
      ])
        assert.throws(() => assessTsrs(state, { ...packet, ...change }), assert.AssertionError);
    }
  } finally {
    rmSync(root, { recursive: true });
  }
});
