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
  commandsFor,
  filesFor,
  requireProfile,
  sha256,
  subsetFor,
  type RegressionProfile,
  type State,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/contract.ts";
import {
  assessTsrs,
  capture,
  inputImages,
  recorder,
  type ProcessOutput,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/execute.ts";
import {
  requireOwnedDispatch,
  writeEmitsInputs,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/run.ts";
const mock: RegressionProfile = {
  subset: "E90-91",
  fix: "1".repeat(40),
  source: "2".repeat(40),
  candidate: "3".repeat(40),
  capsule: "4".repeat(40),
  version: "0.648.0",
  run: "123",
};
const projects = subsetFor("E90-91").projects,
  states = projects.flatMap((project) => project.states);
function temporary(run: (root: string) => void) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "q16-emits-")));
  try {
    run(root);
  } finally {
    rmSync(root, { recursive: true });
  }
}
const output = (status: number, stdout: string, stderr = ""): ProcessOutput => ({
  status,
  stdout: Buffer.from(stdout),
  stderr: Buffer.from(stderr),
  signal: null,
  pid: 123,
  error: null,
  capacityLimited: false,
});
const opaque = (state: State) => state.expectedTsrs.stdoutKind === "OWNED_HEADER_OPAQUE_PRINTER";
test("closed two-report protocol has exactly twenty independent states and restored originals", () => {
  assert.deepEqual(
    projects.map((project) => project.issue),
    [11390, 11391],
  );
  assert.deepEqual(
    projects.map((project) => project.states.length),
    [10, 10],
  );
  assert.equal(states.filter(opaque).length, 2);
  assert.equal(states.filter((state) => !opaque(state)).length, 18);
  for (const project of projects) {
    assert.deepEqual(project.states[0]!.files, project.states[9]!.files);
    assert.deepEqual(project.states[4]!.files, project.states[6]!.files);
    assert.deepEqual(
      project.states.map((state) => state.expectedTsrs.status),
      [0, 0, 0, 0, 0, 2, 0, 2, 0, 0],
    );
  }
  assert.deepEqual(
    states.filter(opaque).map((state) => state.id),
    ["11390/typed-payload-wrong", "11391/runtime-payload-wrong"],
  );
  const typed = projects[0]!.states[0]!.files[2]!.source,
    boundary = projects[0]!.states[1]!.files[2]!.source;
  assert.equal(boundary, typed.replace('defineModel<number>("m15");\n', ""));
  assert(!projects[1]!.states[0]!.files[2]!.source.includes("<template>"));
  assert.equal(
    (projects[0]!.states[8]!.files[2]!.source.match(/defineModel<number>/gu) ?? []).length,
    50,
  );
});
test("exact original package and inner fast command are preserved without vue-tsc injection", () => {
  const commands = commandsFor("E90-91");
  assert.deepEqual(commands.tsrs, [
    "npx",
    "tsrs",
    "--fast",
    "--runExternalCode",
    "-p",
    "tsconfig.json",
  ]);
  assert.equal(commands.reference, null);
  assert.deepEqual(commands.install, ["npm", "i"]);
  assert.deepEqual(commandsFor("A59").tsrs, [
    "npx",
    "tsrs",
    "--runExternalCode",
    "--noEmit",
    "-p",
    "tsconfig.json",
  ]);
  for (const state of states) {
    const changed = filesFor(state, mock);
    for (const [index, file] of changed.entries())
      assert.equal(
        file.source,
        file.path === "package.json"
          ? state.files[index]!.source.replace('"vize": "0.645.0"', '"vize": "0.648.0"')
          : state.files[index]!.source,
      );
    const manifest = JSON.parse(state.files[0]!.source) as { devDependencies: unknown };
    assert.deepEqual(manifest.devDependencies, {
      tsrs: "0.20.1-ts7.1.0-beta",
      typescript: "7.1.0-beta",
      vize: "0.645.0",
      vue: "3.5.44",
    });
  }
  assert.equal(requireProfile(mock), mock);
  assert.throws(() => requireProfile(null), assert.AssertionError);
  requireOwnedDispatch({
    GITHUB_EVENT_NAME: "workflow_dispatch",
    GITHUB_REPOSITORY: "ubugeeei-prod/vize",
    GITHUB_REF: "refs/heads/test/published-emits-11390-11391-20261102",
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
test("eighteen full-stream cases retain their entire independent expected bytes", () =>
  temporary((root) => {
    const saved = recorder(root);
    for (const [index, state] of states.filter((state) => !opaque(state)).entries()) {
      const packet = capture(
        saved,
        "full-" + index,
        ["inert"],
        root,
        () => {},
        () =>
          output(state.expectedTsrs.status, state.expectedTsrs.stdout, state.expectedTsrs.stderr),
      );
      assessTsrs(state, packet);
      assert(readFileSync(packet.stdoutImage.path).equals(Buffer.from(state.expectedTsrs.stdout)));
      for (const change of [
        { stdout: packet.stdout + "unknown" },
        { stderr: "unknown" },
        { status: state.expectedTsrs.status === 0 ? 2 : 0 },
        { wholeStreamsRetained: false },
      ])
        assert.throws(() => assessTsrs(state, { ...packet, ...change }), assert.AssertionError);
    }
  }));
test("two listener intents accept authored opaque printer variants without claiming message text", () =>
  temporary((root) => {
    const saved = recorder(root);
    for (const [index, state] of states.filter(opaque).entries())
      for (const [variant, body] of [
        "Authored opaque mock A.",
        "Authored opaque mock B.\n  Indented continuation.\n    Another opaque continuation.",
      ].entries()) {
        const stdout = state.expectedTsrs.stdout + ": " + body + "\n",
          packet = capture(
            saved,
            "opaque-" + index + "-" + variant,
            ["inert"],
            root,
            () => {},
            () => output(2, stdout),
          ),
          result = assessTsrs(state, packet);
        assert.equal(result.completeTsrsVectorMatchesSourceIntent, false);
        assert.equal("fullPrinterTextOwned" in result && result.fullPrinterTextOwned, false);
        assert(readFileSync(packet.stdoutImage.path).equals(Buffer.from(stdout)));
      }
  }));
test("opaque cases reject missing bodies extra records foreign sites codes statuses and changed inputs", () =>
  temporary((root) => {
    const saved = recorder(root);
    for (const [index, state] of states.filter(opaque).entries()) {
      const header = state.expectedTsrs.stdout,
        packet = capture(
          saved,
          "refuse-" + index,
          ["inert"],
          root,
          () => {},
          () => output(2, header + ": Authored opaque body.\n"),
        );
      for (const change of [
        { stdout: header + ": \n" },
        { stdout: packet.stdout + "unknown extra\n" },
        { stdout: packet.stdout + "  src/Other.ts(1,1): error TS2322: extra\n" },
        { stdout: packet.stdout!.replace("(3,7)", "(3,8)") },
        { stdout: packet.stdout!.replace("TS2322", "TS2339") },
        { status: 0 },
        { stderr: "unknown" },
        { wholeStreamsRetained: false },
        { stdout: header + ": body\n\n" },
      ])
        assert.throws(() => assessTsrs(state, { ...packet, ...change }), assert.AssertionError);
      const files = state.files.map((file) => ({
        ...file,
        source: file.path === "src/Listener.ts" ? file.source + "// unknown\n" : file.source,
      }));
      assert.throws(() => assessTsrs({ ...state, files }, packet), assert.AssertionError);
      assert.throws(
        () =>
          assessTsrs(
            {
              ...state,
              files,
              expectedTsrs: {
                ...state.expectedTsrs,
                sourceInputSha256: sha256(JSON.stringify(files)),
              },
            },
            packet,
          ),
        assert.AssertionError,
      );
      assert.throws(
        () => assessTsrs({ ...state, id: "11390/foreign" }, packet),
        assert.AssertionError,
      );
      const literal = subsetFor("D61").projects[0]!.states[0]!;
      assert.throws(
        () => assessTsrs({ ...literal, expectedTsrs: { ...state.expectedTsrs } }, packet),
        assert.AssertionError,
      );
    }
  }));
test("closed listener transitions create and remove only the independently planned file and config", () =>
  temporary((root) => {
    const original = filesFor(projects[0]!.states[0]!, mock),
      withListener = filesFor(projects[0]!.states[4]!, mock);
    mkdirSync(join(root, "src"));
    for (const file of original) writeFileSync(join(root, file.path), file.source);
    writeEmitsInputs(root, original, withListener);
    inputImages(root, withListener);
    assert(readFileSync(join(root, "package.json")).equals(Buffer.from(original[0]!.source)));
    writeEmitsInputs(root, withListener, original);
    inputImages(root, original);
    assert.throws(
      () =>
        writeEmitsInputs(root, original, [
          ...original,
          { path: "src/unknown.ts", source: "unknown" },
        ]),
      assert.AssertionError,
    );
  }));
test("unknown or symlinked source population refuses before the actual callback without private-name egress", () => {
  for (const mode of ["extra", "link"])
    temporary((root) => {
      const original = filesFor(projects[0]!.states[0]!, mock),
        withListener = filesFor(projects[0]!.states[4]!, mock);
      mkdirSync(join(root, "src"));
      for (const file of original) writeFileSync(join(root, file.path), file.source);
      if (mode === "extra") writeFileSync(join(root, "src/PRIVATE_NAME_CANARY"), "unknown");
      else symlinkSync(join(root, "src/Probe.vue"), join(root, "src/Listener.ts"));
      const evidence = join(root, "evidence");
      mkdirSync(evidence);
      let calls = 0;
      const packet = capture(
        recorder(evidence),
        "guard",
        ["inert"],
        root,
        () => writeEmitsInputs(root, original, withListener),
        () => {
          calls++;
          return output(0, "");
        },
      );
      assert.equal(calls, 0);
      assert.equal(packet.executed, false);
      assert(!JSON.stringify(packet).includes("PRIVATE_NAME_CANARY"));
    });
});
