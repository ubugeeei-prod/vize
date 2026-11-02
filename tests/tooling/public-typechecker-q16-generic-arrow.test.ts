import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import {
  afterProfile,
  requireProfile,
  sha256,
  subsetFor,
  type RegressionProfile,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/contract.ts";
import {
  capture,
  recorder,
  assessTsrs,
  type ProcessOutput,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/execute.ts";
import {
  genericArrowArgv,
  genericArrowData,
  genericArrowFiles,
  genericArrowFix,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/generic-arrow-data.ts";
import {
  assessGenericArrow,
  genericArrowLimitations,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/generic-arrow.ts";
import { assertTscModuleResolution } from "../../tools/benchmarks/scripts/public-typechecker-q16/generic-arrow-route.ts";
import { requireOwnedDispatch } from "../../tools/benchmarks/scripts/public-typechecker-q16/run.ts";

const mock: RegressionProfile = {
  subset: "GArrowTsc",
  fix: genericArrowFix,
  source: "1".repeat(40),
  candidate: "2".repeat(40),
  capsule: "3".repeat(40),
  version: "0.652.0",
  run: "123",
};
const data = genericArrowData();
function temporary(body: (root: string) => void) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "generic-arrow-pure-")));
  try {
    body(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
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
test("the exact five lines and all ten independent syntax and typed controls remain whole", () => {
  assert.equal(data.states.length, 10);
  const original = data.states[0]!.files[2]!.source;
  assert.equal(
    sha256(original),
    "14d54e14493903d7f65a7b2d98865c70dd427801c15da860233a1a70f6c3cd30",
  );
  assert.equal(data.states[1]!.files[2]!.source, original.replace("<T>", "<T,>"));
  assert.deepEqual(data.states[2]!.files, data.states[4]!.files);
  assert.deepEqual(data.states[0]!.files, data.states[3]!.files);
  assert.deepEqual(data.states[0]!.files, data.states[6]!.files);
  assert(data.states[7]!.files[2]!.source.includes("\r\n"));
  assert.deepEqual(
    data.states.map((row) => row.expectedTsc.exit),
    [
      "ZERO",
      "ZERO",
      "NONZERO_DIAGNOSTIC",
      "ZERO",
      "NONZERO_DIAGNOSTIC",
      "NONZERO_DIAGNOSTIC",
      "ZERO",
      "ZERO",
      "ZERO",
      "NONZERO_DIAGNOSTIC",
    ],
  );
  assert.deepEqual(genericArrowArgv, [
    "--runExternalCode",
    "-p",
    "tsconfig.json",
    "--pretty",
    "false",
  ]);
  assert.equal(genericArrowLimitations.privateAppOrPerformanceQualified, false);
  assert.equal(genericArrowLimitations.officialAlpha3ParityQualified, false);
  assert.equal(genericArrowLimitations.sourceDev71OrOtherVueOutputsTransferred, false);
});
test("only Vize changes in the explicitly constructed TSC project and unknown profiles refuse", () => {
  assert.deepEqual(afterProfile, {
    subset: "GArrowTsc",
    fix: "ba01953b4429fabdd652d3497d89a8cf1737697b",
    version: "0.652.0",
    source: "ba01953b4429fabdd652d3497d89a8cf1737697b",
    candidate: "6a9b36fbb8c21b0d928583e3732410f5c2afb12a",
    run: "41523763531",
    capsule: "b567cd114991ae7a9d3b3f6a398bf78246e50182",
  });
  assert.equal(requireProfile(afterProfile), afterProfile);
  assert.equal(requireProfile(mock), mock);
  assert.throws(() => requireProfile(null), assert.AssertionError);
  assert.throws(() => requireProfile({ ...mock, fix: "f".repeat(40) }), assert.AssertionError);
  for (const state of data.states) {
    const files = genericArrowFiles(state, mock);
    assert.deepEqual(JSON.parse(files[0]!.source).devDependencies, {
      typescript: "7.1.0-beta",
      vize: "0.652.0",
      vue: "3.5.44",
    });
    assert.equal(
      files[0]!.source,
      state.files[0]!.source.replace('"vize": "0.649.0"', '"vize": "0.652.0"'),
    );
    assert.deepEqual(files.slice(1), state.files.slice(1));
    const config = JSON.parse(files[1]!.source);
    assert.deepEqual(config.contentMappers, [{ package: "vize", extensions: [".vue"] }]);
    assert.equal(config.compilerOptions.noEmit, true);
    assert(!Object.hasOwn(config, "vueCompilerOptions"));
  }
});
test("prospective nonzero diagnostics retain literal actual status and exact whole streams", () =>
  temporary((root) => {
    const record = recorder(root);
    for (const [index, state] of data.states.entries()) {
      for (const status of state.expectedTsc.exit === "ZERO" ? [0] : [1, 2]) {
        const packet = capture(
          record,
          "state-" + index + "-" + status,
          ["inert"],
          root,
          () => {},
          () => output(status, state.expectedTsc.stdout),
        );
        const result = assessGenericArrow(state, packet);
        assert.equal(result.actualExitStatus, status);
        assert.equal(result.exactNonzeroNumericStatusOwned, false);
        assert(readFileSync(packet.stdoutImage.path).equals(Buffer.from(state.expectedTsc.stdout)));
        for (const change of [
          { stdout: packet.stdout + "unknown" },
          { stderr: "unknown" },
          { wholeStreamsRetained: false },
          { signal: "SIGTERM" as const },
          { status: null },
          { status: -1 },
          { status: state.expectedTsc.exit === "ZERO" ? 2 : 0 },
        ])
          assert.throws(
            () => assessGenericArrow(state, { ...packet, ...change }),
            assert.AssertionError,
          );
      }
    }
    // Older modes retain their own exact numeric law; this new role does not relax them.
    const older = subsetFor("B60-68").projects[0]!.states.find(
      (row) => row.expectedTsrs.status === 2,
    )!;
    const packet = capture(
      record,
      "older",
      ["inert"],
      root,
      () => {},
      () => output(2, older.expectedTsrs.stdout),
    );
    assessTsrs(older, packet);
    assert.throws(() => assessTsrs(older, { ...packet, status: 1 }), assert.AssertionError);
  }));
test("actual module lookup guard refuses scopes conditional exports and prioritized package shadows without execution", () =>
  temporary((root) => {
    const parent = join(root, "node_modules/typescript"),
      target = join(root, "node_modules/@typescript/typescript-linux-x64");
    mkdirSync(join(parent, "lib"), { recursive: true });
    mkdirSync(target, { recursive: true });
    const manifest = {
      name: "typescript",
      version: "7.1.0-beta",
      type: "module",
      bin: { tsc: "./bin/tsc" },
      imports: { "#getExePath": "./lib/getExePath.js" },
      optionalDependencies: { "@typescript/typescript-linux-x64": "7.1.0-beta" },
    };
    const platform = {
      name: "@typescript/typescript-linux-x64",
      version: "7.1.0-beta",
      exports: { "./package.json": "./package.json" },
      os: ["linux"],
      cpu: ["x64"],
    };
    const save = () => {
      writeFileSync(join(parent, "package.json"), JSON.stringify(manifest));
      writeFileSync(join(target, "package.json"), JSON.stringify(platform));
    };
    save();
    assertTscModuleResolution(root, parent, target);
    for (const scope of ["lib", "bin"]) {
      mkdirSync(join(parent, scope), { recursive: true });
      writeFileSync(join(parent, scope, "package.json"), '{"type":"module"}');
      assert.throws(() => assertTscModuleResolution(root, parent, target), assert.AssertionError);
      rmSync(join(parent, scope, "package.json"));
    }
    writeFileSync(
      join(target, "package.json"),
      JSON.stringify({
        ...platform,
        exports: { "./package.json": { import: "./other.json", require: "./package.json" } },
      }),
    );
    assert.throws(() => assertTscModuleResolution(root, parent, target), assert.AssertionError);
    save();
    const shadow = join(parent, "lib/node_modules/@typescript/typescript-linux-x64");
    mkdirSync(shadow, { recursive: true });
    assert.throws(() => assertTscModuleResolution(root, parent, target), assert.AssertionError);
    rmSync(join(parent, "lib/node_modules"), { recursive: true });
    const sourceParent = join(root, "packages/typescript");
    mkdirSync(join(sourceParent, "lib"), { recursive: true });
    writeFileSync(join(sourceParent, "package.json"), JSON.stringify(manifest));
    assert.throws(
      () => assertTscModuleResolution(root, sourceParent, target),
      assert.AssertionError,
    );
  }));
test("stock runner adaptation is closed to the one new manual ref and preserves the entire historical workflow", () => {
  const workflow = readFileSync(
    new URL("../../.github/workflows/public-typechecker-consumer.yml", import.meta.url),
    "utf8",
  );
  const ref = "refs/heads/test/published-generic-arrow-tsx-20261103";
  const expression =
    "${{ github.event_name == 'workflow_dispatch' && inputs.consumer == 'q16-first-subset' && github.ref == '" +
    ref +
    "' && 'ubuntu-24.04' || 'blacksmith-8vcpu-ubuntu-2404' }}";
  let before = workflow;
  for (const job of ["contracts-q16", "public-q16"]) {
    const marker = "  " + job + ":\n";
    const start = before.indexOf(marker);
    assert(start >= 0);
    const at = before.indexOf("    runs-on: " + expression + "\n", start);
    assert(at >= 0);
    before =
      before.slice(0, at) +
      before
        .slice(at)
        .replace(
          "    runs-on: " + expression + "\n",
          "    runs-on: blacksmith-8vcpu-ubuntu-2404\n",
        );
  }
  before = before.replace(" || github.ref == '" + ref + "'", "");
  assert.equal(sha256(before), "48ecddfe5681cf93a7d67a37c42677bcf21d79fc136d1c56c4dada71dabaa943");
  for (const [event, consumer, selectedRef, runner] of [
    ["workflow_dispatch", "q16-first-subset", ref, "ubuntu-24.04"],
    ["pull_request", "q16-first-subset", ref, "blacksmith-8vcpu-ubuntu-2404"],
    ["workflow_dispatch", "other", ref, "blacksmith-8vcpu-ubuntu-2404"],
    ["workflow_dispatch", "q16-first-subset", "refs/heads/main", "blacksmith-8vcpu-ubuntu-2404"],
  ])
    assert.equal(
      event === "workflow_dispatch" && consumer === "q16-first-subset" && selectedRef === ref
        ? "ubuntu-24.04"
        : "blacksmith-8vcpu-ubuntu-2404",
      runner,
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
});
