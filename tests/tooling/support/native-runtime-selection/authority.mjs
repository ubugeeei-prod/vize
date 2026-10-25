import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  loadNativeProviders,
  prepareNativeRuntimeConsumer,
  runtimeChildEnvironment,
  captureConsumerInputs,
  assertProviderFiles,
  CORSA_OVERRIDES,
  SOURCES,
  COMPILER_OPTIONS,
  sha256,
} from "./providers.mjs";
import { runStracedCli } from "./exec-custody.mjs";

const observerRoot = fileURLToPath(new URL("../../../../", import.meta.url));
const git = (root, args) => execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
const write = (file, value) => fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
export const AUTHORITY_CASES = [
  "wrapper-configuration",
  ...CORSA_OVERRIDES.map((name) => `wrapper-${name}`),
  "undeclared-canonical",
  "undeclared-canonical-declared-alias",
  "project-cache",
];
function expectedJson(phase) {
  return {
    files: [
      {
        file: "src/App.vue",
        diagnostics:
          phase === "error"
            ? ["error:3:7 [TS2322] Type 'string' is not assignable to type 'number'."]
            : [],
      },
      { file: "src/support.ts", diagnostics: [] },
    ],
    programs: [
      {
        root: ".",
        tsconfig: "tsconfig.json",
        compilerOptions: COMPILER_OPTIONS,
        files: ["src/App.vue", "src/support.ts"],
      },
    ],
    errorCount: phase === "error" ? 1 : 0,
    warningCount: 0,
    fileCount: 2,
  };
}
function prepare(directory, kind, providers) {
  assert(providers.frozen702, "both official native providers are required");
  const wrapper = kind.startsWith("wrapper-");
  const consumer = prepareNativeRuntimeConsumer({
    directory,
    providers,
    kind: wrapper ? "canonical-702" : kind.endsWith("declared-alias") ? "alias-beta" : "absent",
  });
  const manifest = path.join(directory, "package.json");
  const pkg = JSON.parse(fs.readFileSync(manifest, "utf8"));
  const symlink = (target, relative) => {
    const destination = path.join(directory, relative);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.symlinkSync(target, destination);
    return {
      path: destination,
      target,
      physicalPath: fs.realpathSync(destination),
      sha256: sha256(fs.readFileSync(target)),
    };
  };
  const links = [];
  if (wrapper) {
    pkg.devDependencies["typescript-native"] = "npm:typescript@7.1.0-beta";
    fs.symlinkSync(
      providers.beta.meta.path,
      path.join(directory, "node_modules/typescript-native"),
    );
    assert.equal(providers.beta.meta.manifest.bin.tsc, "./bin/tsc");
    consumer.wrapper = path.join(directory, "node_modules/.bin/tsgo");
    links.push(symlink(path.join(providers.beta.meta.path, "bin/tsc"), "node_modules/.bin/tsgo"));
  } else if (kind.startsWith("undeclared-canonical")) {
    delete pkg.devDependencies.typescript;
    fs.rmSync(path.join(directory, "node_modules/typescript"), { force: true });
    fs.symlinkSync(providers.frozen702.meta.path, path.join(directory, "node_modules/typescript"));
  } else if (kind === "project-cache")
    links.push(symlink(providers.frozen702.executable.path, ".cache/tsgo"));
  fs.writeFileSync(manifest, `${JSON.stringify(pkg, null, 2)}\n`);
  consumer.authorityLinks = links;
  consumer.expectedExecutable =
    wrapper || kind.endsWith("declared-alias") ? providers.beta.executable : null;
  return consumer;
}
export async function observeProviderAuthority(env = process.env) {
  assert.equal(env.VIZE_REQUIRE_NATIVE_RUNTIME_SELECTION, "1");
  assert.equal(process.platform, "linux");
  const mode = env.VIZE_RUNTIME_AUTHORITY_EXPECTATION ?? "after";
  assert(["before", "after"].includes(mode));
  const sourceRoot = fs.realpathSync(env.VIZE_RUNTIME_AUTHORITY_SOURCE_ROOT ?? observerRoot);
  const source = git(sourceRoot, ["rev-parse", "HEAD"]);
  assert.equal(git(sourceRoot, ["status", "--porcelain", "--untracked-files=no"]), "");
  if (mode === "before") assert.equal(source, "cbda7c7e410151d440871ecb97f2ad3fda0f8419");
  const observer = git(observerRoot, ["rev-parse", "HEAD"]);
  const observerFiles = ["authority.mjs", "providers.mjs", "exec-custody.mjs"].map((name) => {
    const file = path.join(path.dirname(import.meta.filename), name);
    const relative = path.relative(observerRoot, file);
    const bytes = fs.readFileSync(file);
    assert.equal(
      sha256(execFileSync("git", ["show", `${observer}:${relative}`], { cwd: observerRoot })),
      sha256(bytes),
    );
    return { path: relative, sha256: sha256(bytes) };
  });
  const { expectedBuildIdentity, validateBuildReceipt, BUILD_RECIPE } = await import(
    pathToFileURL(path.join(sourceRoot, "tests/differential/build-receipt.ts")).href
  );
  const cli = fs.realpathSync(env.VIZE_SOURCE_CLI);
  assert.equal(cli, fs.realpathSync(path.join(sourceRoot, "target/ci/vize")));
  const receipt = JSON.parse(fs.readFileSync(`${cli}.differential-build.json`, "utf8"));
  validateBuildReceipt(receipt, expectedBuildIdentity(sourceRoot), BUILD_RECIPE);
  const providers = loadNativeProviders(env);
  const strace = fs.realpathSync(env.VIZE_NATIVE_STRACE);
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "vize-provider-authority-"));
  const output =
    env.VIZE_RUNTIME_AUTHORITY_OUTPUT ??
    path.join(sourceRoot, "target/vize-tests/native-runtime-authority");
  fs.mkdirSync(output, { recursive: true });
  const directory = fs.mkdtempSync(path.join(output, "observation-"));
  const productBlobs = [
    "crates/vize_carton/src/corsa_resolver.rs",
    "crates/vize_carton/src/corsa_resolver/packages.rs",
  ].map((file) => {
    const bytes = fs.readFileSync(path.join(sourceRoot, file));
    if (mode === "before")
      assert.equal(
        sha256(bytes),
        sha256(
          execFileSync("git", ["show", `e2f6018acd74eaf5901228436d2bb8a71927105b:${file}`], {
            cwd: observerRoot,
          }),
        ),
      );
    return { file, sha256: sha256(bytes) };
  });
  write(path.join(directory, "identities.json"), {
    mode,
    producer: {
      root: sourceRoot,
      source,
      tree: git(sourceRoot, ["rev-parse", "HEAD^{tree}"]),
      receipt,
      productBlobs,
    },
    observer: { root: observerRoot, source: observer, files: observerFiles },
    providers,
    strace: { path: strace, sha256: sha256(fs.readFileSync(strace)) },
    scope:
      "Physical CLI provider authority only; before image never grants current-source/API/LSP/performance credit.",
  });
  const cases = [];
  for (const kind of AUTHORITY_CASES) {
    const consumer = prepare(path.join(base, kind), kind, providers);
    const caseOutput = path.join(directory, kind);
    fs.mkdirSync(caseOutput);
    const childEnv = runtimeChildEnvironment(env, path.join(caseOutput, "environment"));
    const args = [
      "check",
      path.join(consumer.workspace, "src"),
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
    if (kind === "wrapper-configuration") args.push("--corsa-path", consumer.wrapper);
    else if (kind.startsWith("wrapper-"))
      childEnv[kind.slice("wrapper-".length)] = consumer.wrapper;
    const expected =
      mode === "before" ? providers.frozen702.executable : consumer.expectedExecutable;
    const row = {
      kind,
      expectedProvider: expected,
      intendedProvider: consumer.expectedExecutable,
      links: consumer.authorityLinks,
      phases: [],
      state: "pending",
    };
    write(path.join(caseOutput, "inputs.json"), {
      files: captureConsumerInputs(consumer),
      links: consumer.authorityLinks,
      childOverrides: Object.fromEntries(
        CORSA_OVERRIDES.map((name) => [name, childEnv[name] ?? null]),
      ),
    });
    try {
      for (const phase of ["valid", "error", "repair"]) {
        fs.writeFileSync(consumer.entry, SOURCES[phase]);
        assertProviderFiles(providers);
        const observed = runStracedCli({
          strace,
          cli,
          args,
          cwd: consumer.workspace,
          env: childEnv,
          directory: path.join(caseOutput, phase),
          expectedExecutable: expected,
        });
        row.phases.push({ phase, inputs: captureConsumerInputs(consumer), observed });
        write(path.join(caseOutput, "comparison.json"), row);
        if (expected) {
          assert.equal(observed.status, phase === "error" ? 1 : 0);
          assert.equal(observed.streams.stderr.text, "");
          assert.deepEqual(JSON.parse(observed.streams.stdout.text), expectedJson(phase));
        } else {
          assert.equal(observed.status, 1);
          assert.equal(observed.streams.stdout.text, "");
          const snapshot = fs
            .readFileSync(
              path.join(
                sourceRoot,
                "crates/vize_canon/src/batch/snapshots/vize_canon__batch__error__tests__corsa_not_found_no_pm.snap",
              ),
              "utf8",
            )
            .split("\n---\n")[1];
          assert.equal(
            observed.streams.stderr.text,
            `Building Corsa virtual project for 2 files under ${consumer.workspace}...\nError: ${snapshot}\n`,
          );
        }
        assertProviderFiles(providers);
        assert.equal(sha256(fs.readFileSync(cli)), receipt.binarySha256);
      }
      row.state = "PASS";
    } catch (error) {
      row.state = "FAILED";
      row.failure = { message: String(error), stack: error.stack };
    }
    write(path.join(caseOutput, "comparison.json"), row);
    cases.push(row);
  }
  write(path.join(directory, "result.json"), { mode, producer: source, observer, cases });
  assert.equal(git(sourceRoot, ["status", "--porcelain", "--untracked-files=no"]), "");
  assert.deepEqual(
    cases.map(({ state }) => state),
    AUTHORITY_CASES.map(() => "PASS"),
    "all exact fixed provider-authority cases must match their independently labelled image expectation",
  );
  console.log(
    JSON.stringify({
      mode,
      source,
      observer,
      cases: cases.length,
      phases: cases.reduce((n, row) => n + row.phases.length, 0),
      directory,
    }),
  );
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await observeProviderAuthority();
