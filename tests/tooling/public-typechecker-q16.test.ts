import assert from "node:assert/strict";
import {
  mkdtempSync,
  mkdirSync,
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
  afterProfile,
  profileFor,
  requireProfile,
  reportedCommands,
  requireEnvironment,
  subsetFor,
  type Profile,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/contract.ts";
import {
  assessTsrs,
  capture,
  finalCustodiesBeforeAssessment,
  inputImages,
  recorder,
  spawnOptions,
  type ProcessOutput,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/execute.ts";
import { requireOwnedDispatch } from "../../tools/benchmarks/scripts/public-typechecker-q16/run.ts";

const mock: Profile = {
  subset: "D61",
  fix: "1".repeat(40),
  source: "2".repeat(40),
  candidate: "3".repeat(40),
  capsule: "4".repeat(40),
  version: "0.642.0",
  run: "123",
};
const output = (status = 0, stdout = "", stderr = ""): ProcessOutput => ({
  status,
  stdout: Buffer.from(stdout),
  stderr: Buffer.from(stderr),
  signal: null,
  pid: 123,
  error: null,
  capacityLimited: false,
});
test("actual process options omit credential values while retaining original route environment", () => {
  const canary = "PRIVATE_CREDENTIAL_CANARY";
  const options = spawnOptions("/owned", {
    GITHUB_TOKEN: canary,
    npm_TOKEN: canary,
    PRIVATE_SECRET: canary,
    PASSWORD: canary,
    CREDENTIAL: canary,
    PATH: "/declared/path",
    HOME: "/declared/home",
    TSRS_BINARY: "",
  });
  assert.deepEqual(options.env, {
    PATH: "/declared/path",
    HOME: "/declared/home",
    TSRS_BINARY: "",
  });
  assert(!JSON.stringify(options).includes(canary));
  assert.equal(options.cwd, "/owned");
  assert.equal(options.encoding, "buffer");
});
function temporary(run: (root: string) => void) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "q16-pure-")));
  try {
    run(root);
  } finally {
    rmSync(root, { recursive: true });
  }
}
test("closed regression data and noncausal A64 binding retain unknown-profile refusal", () => {
  assert.equal(subsetFor("D61").projects[0]!.states.length, 8);
  assert.equal(subsetFor("C62").projects[0]!.states.length, 4);
  assert.equal(
    subsetFor("B60-68").projects.reduce((sum, row) => sum + row.states.length, 0),
    18,
  );
  for (const id of ["A59-65", "all-ten", "D", "unknown"])
    assert.throws(() => subsetFor(id), assert.AssertionError);
  const declarationProfile: Profile = {
    subset: "A64",
    knownGoodSource: "07d1feb1bb19d862b30aea10fe3465a08a8b1239",
    causalFix: null,
    version: "0.643.0",
    source: "f6f94688127f72fbc1cc1f349df4d66327705985",
    candidate: "6ea5cc7d4c287feaf2ea4fe13012535d8a83e7f2",
    run: "41417714586",
    capsule: "faa59972a4b6877f5d12eb1f24e38a59105e3db5",
  };
  assert.equal(requireProfile(declarationProfile), declarationProfile);
  if (afterProfile === null) assert.throws(() => profileFor(), assert.AssertionError);
  else assert.equal(profileFor(), afterProfile);
  assert.throws(
    () =>
      Reflect.apply(requireProfile, null, [
        { ...declarationProfile, fix: "f6f94688127f72fbc1cc1f349df4d66327705985" },
      ]),
    assert.AssertionError,
  );
  assert.throws(() => subsetFor("A64"), assert.AssertionError);
  const invalid: (Profile | null)[] = [
    null,
    { ...mock, source: "UNKNOWN" },
    { ...mock, capsule: "UNKNOWN" },
    { ...mock, version: "UNKNOWN" },
    { ...mock, run: "UNKNOWN" },
    { ...mock, subset: "A59" },
  ];
  for (const profile of invalid)
    assert.throws(() => requireProfile(profile), assert.AssertionError);
  assert.deepEqual(reportedCommands.tsrs, [
    "npx",
    "tsrs",
    "--runExternalCode",
    "--noEmit",
    "-p",
    "tsconfig.json",
  ]);
});
test("owned feature dispatch is permitted while foreign refs refuse", () => {
  const env = {
    GITHUB_EVENT_NAME: "workflow_dispatch",
    GITHUB_REPOSITORY: "ubugeeei-prod/vize",
    GITHUB_REF: "refs/heads/test/published-q16-first-subset-20261102",
  };
  requireOwnedDispatch(env);
  requireOwnedDispatch({ ...env, GITHUB_REF: "refs/heads/main" });
  for (const changed of [
    { GITHUB_REF: "refs/heads/foreign" },
    { GITHUB_EVENT_NAME: "pull_request" },
    { GITHUB_REPOSITORY: "foreign/project" },
  ])
    assert.throws(() => requireOwnedDispatch({ ...env, ...changed }), assert.AssertionError);
});
test("only the registered package version changes original bytes", () => {
  for (const id of ["D61", "B60-68"])
    for (const project of subsetFor(id).projects)
      for (const state of project.states) {
        const files = filesFor(state, mock);
        assert.equal(files.length, state.files.length);
        for (const [i, file] of files.entries())
          assert.equal(
            file.source,
            file.path === "package.json"
              ? state.files[i]!.source.replace('"vize": "0.638.0"', '"vize": "0.642.0"')
              : state.files[i]!.source,
          );
      }
  const d = subsetFor("D61").projects[0]!.states;
  assert.deepEqual(d[0]!.files, d[4]!.files);
  assert.deepEqual(d[0]!.files, d[7]!.files);
  assert.deepEqual(d[3]!.files, d[6]!.files);
});
test("all override presences including empty are refused without value egress", () =>
  temporary((root) => {
    const secret = "SECRET_CANARY_MUST_NOT_EGRESS",
      record = recorder(root);
    for (const [index, env] of [
      { TSRS_BINARY: "" },
      { TSRS_BINARY: secret },
      { CORSA_PATH: "" },
      { NODE_PATH: secret },
    ].entries()) {
      let executed = 0;
      const row = capture(
        record,
        "override-" + index,
        ["inert"],
        root,
        () => {
          requireEnvironment(env);
        },
        () => {
          executed++;
          return output();
        },
      );
      assert.equal(executed, 0);
      assert.equal(row.executed, false);
      assert(
        !readFileSync(join(root, "override-" + index + ".process.json"), "utf8").includes(secret),
      );
    }
  }));
test("canonical physical exact inputs precede callback and arbitrary source never enters evidence", () =>
  temporary((root) => {
    const project = join(root, "project"),
      evidence = join(root, "evidence");
    mkdirSync(project);
    mkdirSync(evidence);
    mkdirSync(join(project, "src"));
    const files = filesFor(subsetFor("D61").projects[0]!.states[0]!, mock);
    for (const file of files) writeFileSync(join(project, file.path), file.source);
    inputImages(project, files);
    const secret = "PRIVATE_SOURCE_CANARY";
    writeFileSync(join(project, "src/Probe.vue"), secret);
    let executed = 0;
    const row = capture(
      recorder(evidence),
      "changed",
      ["inert"],
      project,
      () => {
        inputImages(project, files);
      },
      () => {
        executed++;
        return output();
      },
    );
    assert.equal(executed, 0);
    assert.equal(row.executed, false);
    assert(!readFileSync(join(evidence, "changed.process.json"), "utf8").includes(secret));
    const held = join(root, "same.vue");
    writeFileSync(held, files.find((file) => file.path === "src/Probe.vue")!.source);
    rmSync(join(project, "src/Probe.vue"));
    symlinkSync(held, join(project, "src/Probe.vue"));
    assert.throws(() => inputImages(project, files), assert.AssertionError);
  }));
test("both full streams persist before any source oracle assessment", () =>
  temporary((root) => {
    const state = subsetFor("D61").projects[0]!.states[2]!,
      expected = state.expectedTsrs;
    const row = capture(
      recorder(root),
      "whole",
      ["inert"],
      root,
      () => {},
      () => output(expected.status, expected.stdout, expected.stderr),
    );
    assert.deepEqual(readFileSync(row.stdoutImage.path), Buffer.from(expected.stdout));
    assert.equal(readFileSync(row.stderrImage.path).length, 0);
    assert.equal(row.qualification, false);
    assessTsrs(state, row);
    for (const mutation of [
      { stdout: row.stdout + "extra" },
      { stderr: "extra" },
      { status: 0 },
      { signal: "SIGTERM" },
      { processError: "error" },
      { wholeStreamsRetained: false },
    ])
      assert.throws(() => assessTsrs(state, { ...row, ...mutation }), assert.AssertionError);
  }));
test("incomplete launch/capacity/invalid UTF8 stays raw and unqualified", () =>
  temporary((root) => {
    const record = recorder(root),
      state = subsetFor("D61").projects[0]!.states[0]!;
    const rows = [
      capture(
        record,
        "throw",
        ["inert"],
        root,
        () => {},
        () => {
          throw new Error("PRIVATE");
        },
      ),
      capture(
        record,
        "capacity",
        ["inert"],
        root,
        () => {},
        () => ({ ...output(), capacityLimited: true }),
      ),
      capture(
        record,
        "utf8",
        ["inert"],
        root,
        () => {},
        () => ({ ...output(), stdout: Buffer.from([255]) }),
      ),
    ];
    for (const row of rows) assert.throws(() => assessTsrs(state, row), assert.AssertionError);
    assert.equal(rows[0]!.executed, true);
    assert.equal(rows[0]!.wholeStreamsRetained, false);
    assert.equal(rows[2]!.stdout, null);
    assert.deepEqual(readFileSync(rows[2]!.stdoutImage.path), Buffer.from([255]));
  }));
test("every selected final custody result is saved despite first failure", () =>
  temporary((root) => {
    const order: string[] = [];
    const rows = finalCustodiesBeforeAssessment(
      [
        {
          id: "one",
          verify() {
            order.push("one");
            throw new Error("SECRET");
          },
        },
        {
          id: "two",
          verify() {
            order.push("two");
          },
        },
      ],
      recorder(root),
    );
    assert.deepEqual(order, ["one", "two"]);
    assert.equal(rows[0]!.retainedRouteAndInputs, false);
    assert.equal(rows[1]!.retainedRouteAndInputs, true);
    const saved = readFileSync(join(root, "all-final-custody.json"), "utf8");
    assert(!saved.includes("SECRET"));
    assert(saved.includes('"publicAcceptance": false'));
  }));
