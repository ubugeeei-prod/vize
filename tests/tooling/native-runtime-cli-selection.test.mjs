import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  binaryRelativePath,
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../differential/build-receipt.ts";
import {
  loadNativeProviders,
  prepareNativeRuntimeConsumer,
  runtimeChildEnvironment,
  captureConsumerInputs,
  assertProviderFiles,
  SOURCES,
  COMPILER_OPTIONS,
  CORSA_OVERRIDES,
  sha256,
} from "./support/native-runtime-selection/providers.mjs";
import { parseExecTrace, runStracedCli } from "./support/native-runtime-selection/exec-custody.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
function expectedCheckJson(consumer, phase) {
  const relative = (file) => path.relative(consumer.workspace, file).split(path.sep).join("/");
  const app = relative(consumer.entry),
    support = relative(path.join(consumer.configRoot, "src/support.ts"));
  return {
    files: [
      {
        file: app,
        diagnostics:
          phase === "error"
            ? ["error:3:7 [TS2322] Type 'string' is not assignable to type 'number'."]
            : [],
      },
      { file: support, diagnostics: [] },
    ],
    programs: [
      {
        root: relative(consumer.configRoot) || ".",
        tsconfig: relative(consumer.configFile),
        compilerOptions: COMPILER_OPTIONS,
        files: [app, support],
      },
    ],
    errorCount: phase === "error" ? 1 : 0,
    warningCount: 0,
    fileCount: 2,
  };
}

void test("native CLI exec custody preserves full argv and rejects incomplete trace framing", () => {
  const line =
    'execve("/actual/compiler", ["/actual/compiler", "--project", "/project/tsconfig.json"], 0x1 /* 4 vars */) = 0';
  assert.deepEqual(parseExecTrace(line), [
    {
      line: 1,
      raw: line,
      executable: "/actual/compiler",
      argv: ["/actual/compiler", "--project", "/project/tsconfig.json"],
      success: true,
      result: "0",
    },
  ]);
  assert.throws(() => parseExecTrace('execve("/actual/compiler", ["..."], <unfinished ...>'));
  const cEscaped = String.raw`execve("/actual/compiler", ["/actual/compiler", "/project/\345\210\251\347\224\250\350\200\205/App.vue", "quote\"slash\\tab\t", "\xef\xbb\xbfvalue"], 0x1) = 0`;
  assert.deepEqual(parseExecTrace(cEscaped)[0].argv, [
    "/actual/compiler",
    "/project/利用者/App.vue",
    'quote"slash\\tab\t',
    "\ufeffvalue",
  ]);
  for (const argv of [
    '["ok", ...]',
    '["ok",]',
    '["ok" "missing-comma"]',
    String.raw`["\q"]`,
    String.raw`["\777"]`,
    String.raw`["\xff"]`,
    String.raw`["\x1"]`,
    String.raw`["\0"]`,
  ])
    assert.throws(() => parseExecTrace(`execve("/actual/compiler", ${argv}, 0x1) = 0`));
  assert.throws(() => loadNativeProviders({}), /VIZE_NATIVE_BETA_ROOT/u);
  const invalid = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-provider-refusal-"));
  try {
    fs.mkdirSync(path.join(invalid, "node_modules/typescript"), { recursive: true });
    fs.writeFileSync(
      path.join(invalid, "node_modules/typescript/package.json"),
      JSON.stringify({ name: "typescript", version: "6.0.3" }),
    );
    assert.throws(
      () =>
        loadNativeProviders({ VIZE_NATIVE_BETA_ROOT: invalid, VIZE_TYPESCRIPT_SIX_ROOT: invalid }),
      /7\.1\.0-beta/u,
    );
  } finally {
    fs.rmSync(invalid, { recursive: true, force: true });
  }
  const isolated = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-child-env-"));
  const originalHome = process.env.HOME;
  try {
    const original = { HOME: "parent-home", CORSA_PATH: "forbidden" };
    const child = runtimeChildEnvironment(original, isolated);
    assert.equal(child.HOME, path.join(isolated, "empty-home"));
    assert.equal(fs.statSync(child.HOME).isDirectory(), true);
    assert.equal(child.CORSA_PATH, undefined);
    assert.deepEqual(fs.readdirSync(child.PATH), [
      process.platform === "win32" ? "node.exe" : "node",
    ]);
    assert.equal(
      fs.realpathSync(path.join(child.PATH, process.platform === "win32" ? "node.exe" : "node")),
      fs.realpathSync(process.execPath),
    );
    const identity = JSON.parse(fs.readFileSync(path.join(isolated, "node-identity.json"), "utf8"));
    assert.equal(identity.physicalPath, fs.realpathSync(process.execPath));
    assert.equal(identity.version, process.version);
    assert.equal(identity.sha256, sha256(fs.readFileSync(process.execPath)));
    assert.equal(original.HOME, "parent-home");
    assert.equal(process.env.HOME, originalHome);
  } finally {
    fs.rmSync(isolated, { recursive: true, force: true });
  }
  const dummy = {
    workspace: "/project",
    configRoot: "/project",
    entry: "/project/src/App.vue",
    configFile: "/project/tsconfig.json",
  };
  const before = expectedCheckJson(dummy, "error");
  const changed = structuredClone(before);
  changed.opaque = "unapproved";
  assert.throws(() => assert.deepEqual(changed, before));
  assert.deepEqual(
    runtimeChildEnvironment(Object.fromEntries(CORSA_OVERRIDES.map((key) => [key, "forbidden"]))),
    { VIZE_CORSA_DEV_PATHS: "0", NO_COLOR: "1", FORCE_COLOR: "0" },
  );
});

void test("source CLI selects actual user-installed native packages and retains whole diagnostic results", async (t) => {
  const selected =
    process.env.VIZE_REQUIRE_NATIVE_RUNTIME_SELECTION === "1" ||
    process.env.VIZE_SOURCE_CLI !== undefined;
  if (!selected) {
    t.skip("explicit source CLI/provider qualification not selected; no runtime credit");
    return;
  }
  assert.equal(process.platform, "linux", "actual Linux execve custody is mandatory");
  assert.ok(process.env.VIZE_SOURCE_CLI, "VIZE_SOURCE_CLI is mandatory; no source/public fallback");
  const cli = fs.realpathSync(process.env.VIZE_SOURCE_CLI),
    expected = expectedBuildIdentity(root);
  assert.equal(cli, fs.realpathSync(path.join(root, binaryRelativePath())));
  const receipt = JSON.parse(fs.readFileSync(`${cli}.differential-build.json`, "utf8"));
  validateBuildReceipt(receipt, expected, receipt.recipe);
  const version = spawnSync(cli, ["--version"], { encoding: "utf8" });
  assert.equal(version.status, 0);
  assert.equal(version.signal, null);
  assert.equal(version.error, undefined);
  assert.equal(version.stdout, `${expected.cliVersion}\n`);
  assert.equal(version.stderr, "");
  assert.ok(
    process.env.VIZE_NATIVE_STRACE,
    "VIZE_NATIVE_STRACE is mandatory; no global tool fallback",
  );
  const strace = fs.realpathSync(process.env.VIZE_NATIVE_STRACE),
    providers = loadNativeProviders();
  const straceProbe = spawnSync(strace, ["--version"], { encoding: "utf8" });
  assert.equal(straceProbe.status, 0);
  assert.equal(straceProbe.signal, null);
  assert.equal(straceProbe.error, undefined);
  const straceIdentity = {
    path: strace,
    sha256: sha256(fs.readFileSync(strace)),
    version: straceProbe.stdout,
    stderr: straceProbe.stderr,
  };
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-runtime-cli-10694-"));
  const archive = path.join(root, "target/vize-tests/native-runtime-cli-selection-10694");
  fs.mkdirSync(archive, { recursive: true });
  const output = fs.mkdtempSync(path.join(archive, "source-"));
  const write = (file, value) => fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
  write(path.join(output, "identities.json"), {
    source: receipt,
    cliVersionProbe: { status: version.status, stdout: version.stdout, stderr: version.stderr },
    strace: straceIdentity,
    providers,
    consumerBase: base,
    observerSourceSha256: sha256(fs.readFileSync(fileURLToPath(import.meta.url))),
  });
  const cases = [];
  for (const kind of [
    "absent",
    "canonical-beta",
    "alias-beta",
    "explicit-beta",
    "selected-config",
    "stale-decoys",
  ]) {
    await t.test(kind, () => {
      const consumer = prepareNativeRuntimeConsumer({
        directory: path.join(base, kind),
        kind,
        providers,
      });
      const caseOutput = path.join(output, kind);
      fs.mkdirSync(caseOutput);
      const env = runtimeChildEnvironment(process.env, path.join(caseOutput, "environment"));
      assertProviderFiles(providers);
      assert.equal(sha256(fs.readFileSync(cli)), expected.binarySha256);
      const phaseRows = [];
      write(path.join(caseOutput, "inputs.json"), consumer.inputFiles);
      if (!consumer.expectedExecutable) {
        const formatted = runStracedCli({
          strace,
          cli,
          args: ["fmt", consumer.entry, "--write", "--no-config"],
          cwd: consumer.workspace,
          env,
          directory: path.join(caseOutput, "fmt"),
          expectedExecutable: null,
        });
        assert.equal(formatted.status, 0);
        fs.writeFileSync(consumer.entry, SOURCES.valid);
      }
      for (const phase of consumer.expectedExecutable ? ["valid", "error", "repair"] : ["valid"]) {
        fs.writeFileSync(consumer.entry, SOURCES[phase]);
        const args = [
          "check",
          path.join(consumer.configRoot, "src"),
          "--tsconfig",
          consumer.configFile,
          "--format",
          "json",
          "--servers",
          "1",
          "--checkers",
          "1",
          "--no-config",
        ];
        if (consumer.explicitExecutable)
          args.push("--corsa-path", consumer.explicitExecutable.path);
        const observed = runStracedCli({
          strace,
          cli,
          args,
          cwd: consumer.workspace,
          env,
          directory: path.join(caseOutput, phase),
          expectedExecutable: consumer.expectedExecutable,
        });
        phaseRows.push({ phase, inputs: captureConsumerInputs(consumer), observed });
        write(path.join(caseOutput, "phases.json"), phaseRows);
        if (consumer.expectedExecutable) {
          assert.equal(observed.status, phase === "error" ? 1 : 0);
          assert.deepEqual(
            JSON.parse(observed.streams.stdout.text),
            expectedCheckJson(consumer, phase),
            "entire JSON envelope, compiler options, files and diagnostics remain exact",
          );
        } else {
          assert.equal(observed.status, 1);
          assert.equal(observed.streams.stdout.text, "");
          assert.match(observed.streams.stderr.text, /native TypeScript runtime not found/u);
          assert.match(
            observed.streams.stderr.text,
            /typescript-native@npm:typescript@7\.1\.0-beta/u,
          );
        }
        assertProviderFiles(providers);
        assert.equal(sha256(fs.readFileSync(cli)), expected.binarySha256);
      }
      cases.push({
        kind,
        wholeDiagnosticPhases: phaseRows.length,
        expectedExecutable: consumer.expectedExecutable,
        scope: "actual CLI only; no native API or clean LSP lifecycle credit",
      });
    });
  }
  assert.equal(cases.length, 6);
  write(path.join(output, "result.json"), {
    state: "PASS",
    source: receipt,
    cases,
    scope:
      "Real user-installed package selection and complete CLI JSON diagnostics only. No API/lifecycle/performance credit; strace observer is present.",
  });
});
