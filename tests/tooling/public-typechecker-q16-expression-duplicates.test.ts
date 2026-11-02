import assert from "node:assert/strict";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import test from "node:test";
import {
  afterProfile,
  expressionDuplicatesFix,
  requireProfile,
  sha256,
  type RegressionProfile,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/contract.ts";
import {
  capture,
  inputImages,
  recorder,
  type ProcessOutput,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/execute.ts";
import {
  expressionDuplicatesArgs,
  expressionDuplicatesData,
  expressionDuplicatesFiles,
  expressionDuplicatesReferenceArgs,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/expression-duplicates-data.ts";
import {
  assessExpressionDuplicates,
  expressionDuplicatesLimitations,
  writeExpressionDuplicatesInputs,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/expression-duplicates.ts";
import {
  main,
  requireOwnedDispatch,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/run.ts";

const mock: RegressionProfile = {
  subset: "ExpressionDuplicates11374",
  fix: "1".repeat(40),
  source: "2".repeat(40),
  candidate: "3".repeat(40),
  capsule: "4".repeat(40),
  run: "123",
  version: "0.999.0",
};
const data = expressionDuplicatesData();
const reference =
  "src/MultilineProp.vue(12,16): error TS2532: Object is possibly 'undefined'.\nsrc/MultilineVIf.vue(10,7): error TS2532: Object is possibly 'undefined'.\nsrc/NestedVIf.vue(6,14): error TS2532: Object is possibly 'undefined'.\n";
function temporary(body: (root: string) => void) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "expression-pure-")));
  try {
    body(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
function output(status: number, stdout: string): ProcessOutput {
  return {
    status,
    stdout: Buffer.from(stdout),
    stderr: Buffer.alloc(0),
    signal: null,
    pid: 123,
    error: null,
    capacityLimited: false,
  };
}
test("exact six original fences and only four independent source states remain whole", () => {
  assert.deepEqual(
    data.states.map((s) => s.id),
    ["original", "repair-all", "independent-script-error", "restored-original"],
  );
  assert.deepEqual(
    data.states.map((s) => s.status),
    [2, 0, 2, 2],
  );
  assert.equal(
    data.states.reduce((sum, s) => sum + s.files.length, 0),
    25,
  );
  assert.equal(data.states[0]!.stdout, reference);
  assert.deepEqual(data.states[0]!.files, data.states[3]!.files);
  assert.equal(data.states[1]!.stdout, "");
  assert.equal(
    data.states[2]!.stdout,
    "src/Control.ts(1,7): error TS2322: Type 'number' is not assignable to type 'string'.\n",
  );
  for (const f of data.states[0]!.files.slice(3))
    assert.equal(
      data.states[1]!.files.find((n) => n.path === f.path)!.source,
      f.source.replace("items[0].label", "items[0]!.label"),
    );
  assert.deepEqual(data.states[2]!.files.slice(0, 6), data.states[1]!.files);
  assert.equal(data.states[2]!.files[6]!.source, "const wrong: string = 1; void wrong;\n");
  const files = expressionDuplicatesFiles(data.states[0]!, mock);
  assert.deepEqual(files.slice(1), data.states[0]!.files.slice(1));
  assert.equal(
    files[0]!.source,
    data.states[0]!.files[0]!.source.replace('"vize": "0.640.0"', '"vize": "0.999.0"'),
  );
  assert.deepEqual(JSON.parse(files[0]!.source).devDependencies, {
    tsrs: "0.20.1-ts7.1.0-beta",
    typescript: "5.9.3",
    vize: "0.999.0",
    vue: "3.5.43",
    "vue-tsc": "3.2.6",
  });
  const config = JSON.parse(files[1]!.source);
  assert.equal(config.compilerOptions.noEmit, true);
  assert.equal(config.compilerOptions.noUncheckedIndexedAccess, true);
  assert.equal(config.compilerOptions.noUnusedLocals, true);
  assert.deepEqual(config.include, ["src/**/*.ts", "src/**/*.vue"]);
  assert.equal(data.states[2]!.files[6]!.path, "src/Control.ts");
  assert(!Object.hasOwn(config, "vueCompilerOptions"));
  assert.deepEqual(expressionDuplicatesArgs, [
    "--runExternalCode",
    "-p",
    "tsconfig.json",
    "--pretty",
    "false",
  ]);
  assert.deepEqual(expressionDuplicatesReferenceArgs, [
    "--noEmit",
    "-p",
    "tsconfig.json",
    "--pretty",
    "false",
  ]);
  assert.equal(expressionDuplicatesLimitations.sourceObservationsTransferred, false);
});
test("unknown containing source authority refuses before output allocation or public operations", async () => {
  assert.equal(afterProfile, null);
  assert.equal(expressionDuplicatesFix, null);
  assert.throws(() => requireProfile(null), assert.AssertionError);
  assert.throws(() => requireProfile(mock), assert.AssertionError);
  const root = realpathSync(mkdtempSync(join(tmpdir(), "expression-unknown-")));
  try {
    const out = join(root, "never-created");
    await assert.rejects(
      main(["--out", out, "--npm", "/unexecuted/npm", "--npx", "/unexecuted/npx"]),
      assert.AssertionError,
    );
    assert(!existsSync(out));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
test("literal complete streams reject duplicates virtual locations extra diagnostics and wrong outcomes", () =>
  temporary((root) => {
    const record = recorder(root);
    for (const [i, state] of data.states.entries()) {
      const packet = capture(
        record,
        "state-" + i,
        ["inert"],
        root,
        () => {},
        () => output(state.status, state.stdout),
      );
      const result = assessExpressionDuplicates(state, packet);
      assert.equal(result.wholeStreamsOwned, true);
      assert(readFileSync(packet.stdoutImage.path).equals(Buffer.from(state.stdout)));
      for (const delta of [
        { stdout: state.stdout + reference.split("\n")[0] + "\n" },
        { stdout: state.stdout + "foreign\n" },
        { stdout: state.stdout.replace("(12,16)", "(54,16)") + " " },
        { stdout: state.stdout.replace("TS2532", "TS2339") + " " },
        { stderr: "foreign" },
        { status: state.status === 0 ? 2 : 1 },
        { status: null },
        { signal: "SIGTERM" as const },
        { wholeStreamsRetained: false },
      ])
        assert.throws(
          () => assessExpressionDuplicates(state, { ...packet, ...delta }),
          assert.AssertionError,
        );
    }
  }));
test("actual input writes restore originals remove only the owned Control and refuse foreign or linked sources", () =>
  temporary((root) => {
    let files = expressionDuplicatesFiles(data.states[0]!, mock);
    for (const f of files) {
      mkdirSync(dirname(join(root, f.path)), { recursive: true });
      writeFileSync(join(root, f.path), f.source);
    }
    for (const state of data.states) {
      const next = expressionDuplicatesFiles(state, mock);
      writeExpressionDuplicatesInputs(root, files, next, mock);
      files = next;
      assert.deepEqual(
        inputImages(root, files).map((f) => ({ path: f.path, source: f.source })),
        files,
      );
    }
    assert(!existsSync(join(root, "src/Control.ts")));
    const canary = "PRIVATE_EXPRESSION_INPUT_CANARY";
    for (const target of ["package.json", "src/NestedVIf.vue"]) {
      const forged = files.map((f) =>
        f.path === target ? { ...f, source: f.source + canary } : f,
      );
      let calls = 0,
        failure: Error | undefined;
      assert.throws(() => {
        try {
          writeExpressionDuplicatesInputs(root, files, forged, mock);
          calls++;
        } catch (e) {
          failure = e as Error;
          throw e;
        }
      }, assert.AssertionError);
      assert.equal(calls, 0);
      assert(!String(failure?.message).includes(canary));
    }
    const target = join(root, "src/NestedVIf.vue"),
      saved = join(root, "owned-copy.vue");
    writeFileSync(saved, files.find((f) => f.path === "src/NestedVIf.vue")!.source);
    rmSync(target);
    symlinkSync(saved, target);
    let calls = 0;
    assert.throws(() => {
      writeExpressionDuplicatesInputs(root, files, files, mock);
      calls++;
    }, assert.AssertionError);
    assert.equal(calls, 0);
  }));
test("only the closed role ref runner and Node seams alter the full frozen workflow", () => {
  const workflow = readFileSync(
    new URL("../../.github/workflows/public-typechecker-consumer.yml", import.meta.url),
    "utf8",
  );
  const ref = "refs/heads/test/published-expression-duplicates-11374-20261103";
  assert.equal(workflow.split(ref).length, 5);
  assert.equal(
    sha256(workflow.replaceAll(" || github.ref == '" + ref + "'", "")),
    "0ae41e50f9a693bab2d5d44c0193ced6d34d4730c20b6db241f84691dc3931ef",
  );
  requireOwnedDispatch({
    GITHUB_EVENT_NAME: "workflow_dispatch",
    GITHUB_REPOSITORY: "ubugeeei-prod/vize",
    GITHUB_REF: ref,
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
  const source = readFileSync(
    new URL("../../tools/benchmarks/scripts/public-typechecker-q16/source.ts", import.meta.url),
    "utf8",
  );
  assert(source.includes("ns + expressionDuplicatesFileName"));
});
