import assert from "node:assert/strict";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import {
  filesFor,
  requireProfile,
  subsetFor,
  type RegressionProfile,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/contract.ts";
import {
  a65Data,
  a65Inputs,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/a-data.ts";
import {
  assessA65,
  expectedA65Json,
  resetA65Outputs,
  retainA65Outputs,
  requireSameA65Declarations,
  type A65Snapshot,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/a65-results.ts";
import {
  assessTsrs,
  capture,
  recorder,
  type ProcessOutput,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/execute.ts";
const profile: RegressionProfile = {
  subset: "A59-65",
  fix: "a321c37109d86c03afeacc833b1760c85ad4a32c",
  version: "0.646.0",
  source: "a321c37109d86c03afeacc833b1760c85ad4a32c",
  candidate: "46f7f273d05bb7ea9d94bfdfb22bd792f6075f95",
  run: "41438403319",
  capsule: "1".repeat(40),
};
test("historical A profile validates retained release fields", () => {
  assert.deepEqual(
    requireProfile({ ...profile, capsule: "21dc2dcb31c73131a945cd87ec8c2d6c58887bac" }),
    {
      subset: "A59-65",
      fix: "a321c37109d86c03afeacc833b1760c85ad4a32c",
      version: "0.646.0",
      source: "a321c37109d86c03afeacc833b1760c85ad4a32c",
      candidate: "46f7f273d05bb7ea9d94bfdfb22bd792f6075f95",
      run: "41438403319",
      capsule: "21dc2dcb31c73131a945cd87ec8c2d6c58887bac",
    },
  );
});
function temporary(run: (root: string) => void) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "q16-a-")));
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
function expectedStdout(index: number, cwd: string, color = false) {
  const state = a65Data.states[index]!;
  if (state.json) return JSON.stringify(expectedA65Json(state), null, 2) + "\n";
  const green = (text: string) => (color ? "\x1b[32m" + text + "\x1b[0m" : text);
  return (
    "\n" +
    green("✓") +
    " Type checked 2 files in 1.00ms (collect: 0.00ns, imports: 0.00ns, gen: 0.50ms, corsa: 0.40ms, dts: 0.10ms)\n  " +
    green("No type errors found!") +
    "\n  " +
    green("Emitted 3 declaration file(s)") +
    " to " +
    join(cwd, "out") +
    "\n"
  );
}
function progress(cwd: string, negative = false) {
  return (
    "Building Corsa virtual project for 2 files under " +
    cwd +
    "...\nRunning Corsa diagnostics for 2 files...\n" +
    (negative ? "Skipping declaration emit because type errors were reported.\n" : "")
  );
}
function snapshot(
  root: string,
  record: ReturnType<typeof recorder>,
  label: string,
  clean: boolean,
  body = "export type Authored = unknown;\n",
) {
  if (clean) {
    mkdirSync(join(root, "out"));
    for (const name of a65Data.outputFiles) writeFileSync(join(root, "out", name), body);
  }
  return retainA65Outputs(root, record, label);
}
test("closed A roles preserve actual single declaration original and source-owned seven controls", () => {
  assert.equal(requireProfile(profile), profile);
  assert.throws(() => requireProfile(null), assert.AssertionError);
  assert.throws(() => requireProfile({ ...profile, fix: "f".repeat(40) }), assert.AssertionError);
  assert.equal(subsetFor("A59").projects[0]!.states.length, 3);
  assert.deepEqual(
    subsetFor("A59").projects[0]!.states.map((row) => row.expectedTsrs.status),
    [0, 2, 0],
  );
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
  assert.deepEqual(
    a65Data.states.map((row) => row.json),
    [false, true, true, true, true, true, true],
  );
  assert.deepEqual(
    a65Data.states.map((row) => row.status),
    [0, 0, 0, 0, 1, 1, 0],
  );
  assert.deepEqual(a65Data.states[0]!.files, a65Data.states[6]!.files);
  const tsconfig = JSON.parse(
    a65Data.states[0]!.files.find((row) => row.path === "tsconfig.json")!.source,
  ) as { compilerOptions: { noUnusedLocals?: boolean } };
  assert.equal(tsconfig.compilerOptions.noUnusedLocals, true);
  assert(!a65Data.originalArgs.includes("--format"));
  assert(!a65Data.originalArgs.includes("vue-tsc"));
});
test("both A original projects change only the registered public vize version", () => {
  const pairs = [
    ...subsetFor("A59").projects[0]!.states.map((state) => ({
      before: state.files,
      after: filesFor(state, profile),
    })),
    ...a65Data.states.map((state) => ({ before: state.files, after: a65Inputs(state, profile) })),
  ];
  for (const pair of pairs)
    for (const [index, file] of pair.after.entries())
      assert.equal(
        file.source,
        file.path === "package.json"
          ? pair.before[index]!.source.replace('"vize": "0.638.0"', '"vize": "0.646.0"')
          : pair.before[index]!.source,
      );
  assert(!JSON.parse(a65Inputs(a65Data.states[0]!, profile)[0]!.source).devDependencies["vue-tsc"]);
});
test("A59 preserves full independent streams and refuses unknown output", () =>
  temporary((root) => {
    const record = recorder(root);
    for (const [index, state] of subsetFor("A59").projects[0]!.states.entries()) {
      const row = capture(
        record,
        "state-" + index,
        ["inert"],
        root,
        () => {},
        () =>
          output(state.expectedTsrs.status, state.expectedTsrs.stdout, state.expectedTsrs.stderr),
      );
      assessTsrs(state, row);
      assert.throws(
        () => assessTsrs(state, { ...row, stdout: row.stdout + "unknown" }),
        assert.AssertionError,
      );
    }
  }));
test("A65 retains all raw declarations and applies only source-owned intent to seven complete packets", () =>
  temporary((root) => {
    const record = recorder(root),
      held: A65Snapshot[] = [];
    for (const [index, state] of a65Data.states.entries()) {
      const packet = capture(
          record,
          "case-" + index,
          ["inert"],
          root,
          () => {},
          () =>
            output(state.status, expectedStdout(index, root), progress(root, state.status !== 0)),
        ),
        files = snapshot(root, record, "case-" + index, state.status === 0);
      held.push(files);
      assessA65(state, packet, files);
      assert.deepEqual(readFileSync(packet.stdoutImage.path), Buffer.from(packet.stdout!));
      for (const file of files.files) assert.deepEqual(readFileSync(file.image.path), file.raw);
      for (const mutation of [
        { status: state.status === 0 ? 1 : 0 },
        { stderr: packet.stderr + "extra" },
        { stdout: packet.stdout + "extra" },
        { wholeStreamsRetained: false },
      ])
        assert.throws(
          () => assessA65(state, { ...packet, ...mutation }, files),
          assert.AssertionError,
        );
      resetA65Outputs(root, files);
    }
    requireSameA65Declarations(held[0]!, held[1]!);
    requireSameA65Declarations(held[0]!, held[6]!);
  }));
test("original text accepts both literal source color branches without stripping any raw bytes", () =>
  temporary((root) => {
    const record = recorder(root),
      files = snapshot(root, record, "colors", true);
    for (const [index, color] of [false, true].entries()) {
      const packet = capture(
        record,
        "color-" + index,
        ["inert"],
        root,
        () => {},
        () => output(0, expectedStdout(0, root, color), progress(root)),
      );
      assessA65(a65Data.states[0]!, packet, files);
      assert.throws(
        () =>
          assessA65(
            a65Data.states[0]!,
            { ...packet, stdout: packet.stdout!.replace("Type checked 2", "Type checked 3") },
            files,
          ),
        assert.AssertionError,
      );
    }
  }));
test("wrong extra symlinked and leaked outputs remain retained/unqualified before another callback", () => {
  for (const mode of ["extra", "link", "virtual"]) {
    temporary((root) => {
      const record = recorder(root),
        files = snapshot(
          root,
          record,
          "before",
          true,
          mode === "virtual" ? "vize:virtual-ts/private" : "export type A = unknown;\n",
        );
      if (mode === "extra")
        writeFileSync(join(root, "out", "PRIVATE_OUTPUT_CANARY"), "unrequested");
      if (mode === "link") {
        const path = join(root, "out", a65Data.outputFiles[0]!);
        rmSync(path);
        symlinkSync(files.files[0]!.image.path, path);
      }
      const changed = retainA65Outputs(root, record, "changed");
      if (mode === "virtual") {
        const packet = capture(
          record,
          "virtual",
          ["inert"],
          root,
          () => {},
          () => output(0, expectedStdout(0, root), progress(root)),
        );
        assert.throws(() => assessA65(a65Data.states[0]!, packet, changed), assert.AssertionError);
      } else {
        let calls = 0;
        const packet = capture(
          record,
          "refuse",
          ["inert"],
          root,
          () => resetA65Outputs(root, changed),
          () => {
            calls++;
            return output(0, "", "");
          },
        );
        assert.equal(calls, 0);
        assert.equal(packet.executed, false);
        assert(
          !readFileSync(join(root, "changed.outputs.json"), "utf8").includes(
            "PRIVATE_OUTPUT_CANARY",
          ),
        );
      }
    });
  }
});
test("identical-input declaration restoration requires actual whole byte equality without a golden", () =>
  temporary((root) => {
    const record = recorder(root),
      one = snapshot(root, record, "one", true, "export type One = unknown;\n");
    resetA65Outputs(root, one);
    const two = snapshot(root, record, "two", true, "export type Two = unknown;\n");
    assert.throws(() => requireSameA65Declarations(one, two), assert.AssertionError);
  }));
test("source-owned JSON population and authored diagnostic field changes refuse", () =>
  temporary((root) => {
    const record = recorder(root),
      state = a65Data.states[4]!,
      files = snapshot(root, record, "bad", false),
      good = expectedA65Json(state),
      packet = capture(
        record,
        "diagnostic",
        ["inert"],
        root,
        () => {},
        () => output(1, JSON.stringify(good), progress(root, true)),
      );
    const mutants = [
      { ...good, errorCount: 0 },
      { ...good, files: good.files.slice(0, 1) },
      { ...good, programs: [] },
      { ...good, unexpected: true },
      {
        ...good,
        files: [
          good.files[0],
          { file: "src/Probe.vue", diagnostics: ["error:7:8 [TS2322] changed"] },
        ],
      },
    ];
    for (const mutation of mutants)
      assert.throws(
        () => assessA65(state, { ...packet, stdout: JSON.stringify(mutation) }, files),
        assert.AssertionError,
      );
  }));
