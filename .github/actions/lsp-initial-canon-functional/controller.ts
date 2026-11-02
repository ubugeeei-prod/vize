import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  CANDIDATE_SOURCE,
  CANDIDATE_TREE,
  PROVIDER_SHA256,
  SOURCE_PINS,
  TOOLING_PATHS,
} from "./source-pins.ts";
import {
  TESTS,
  defaultCliArtifact,
  FIXTURE_PATH,
  admittedFixture,
  checkAmbient,
  checkParent,
  childEnv,
  checkExecution,
  checkNativeExit,
  checkPackets,
  libraryArtifact,
  object,
  sdkArtifacts,
  type Obj,
} from "./laws.ts";

type Exit = { status: number | null; signal: string | null; error: string | null };
const SHA = /^[0-9a-f]{40}$/;
const hash = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
function save(file: string, value: unknown): void {
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`, { flag: "wx", mode: 0o600 });
}
function read(file: string, maximum = 32 * 2 ** 20): Buffer {
  assert.ok(fs.lstatSync(file).isFile() && !fs.lstatSync(file).isSymbolicLink());
  assert.ok(fs.statSync(file).size <= maximum, "raw evidence exceeds declared read cap");
  return fs.readFileSync(file);
}
function identity(file: string): Obj {
  const realpath = fs.realpathSync(file);
  const stat = fs.statSync(realpath, { bigint: true });
  assert.ok(stat.isFile());
  const bytes = read(realpath, 512 * 2 ** 20);
  assert.deepEqual(bytes.subarray(0, 4), Buffer.from([127, 69, 76, 70]));
  return {
    realpath,
    bytes: bytes.length,
    dev: String(stat.dev),
    ino: String(stat.ino),
    mtimeNs: String(stat.mtimeNs),
    ctimeNs: String(stat.ctimeNs),
    sha256: hash(bytes),
  };
}
function run(
  repo: string,
  out: string,
  label: string,
  executable: string,
  args: string[],
  env: NodeJS.ProcessEnv,
  timeout: number,
): Exit {
  const stdout = fs.openSync(path.join(out, `${label}.stdout`), "wx", 0o600);
  const stderr = fs.openSync(path.join(out, `${label}.stderr`), "wx", 0o600);
  const startNs = String(process.hrtime.bigint());
  let result: Exit;
  try {
    const child = spawnSync(executable, args, {
      cwd: repo,
      env,
      stdio: ["ignore", stdout, stderr],
      timeout,
      killSignal: "SIGTERM",
    });
    result = { status: child.status, signal: child.signal, error: child.error?.message ?? null };
  } finally {
    fs.closeSync(stdout);
    fs.closeSync(stderr);
  }
  save(path.join(out, `${label}.exit.json`), {
    executable,
    args,
    cwd: repo,
    startNs,
    endNs: String(process.hrtime.bigint()),
    ...result,
  });
  return result;
}
function success(exit: Exit): void {
  assert.deepEqual(exit, { status: 0, signal: null, error: null });
}
function git(repo: string, ...args: string[]): string {
  const result = spawnSync("git", args, { cwd: repo, encoding: "utf8", maxBuffer: 16 * 2 ** 20 });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}
function sourceSnapshot(repo: string): Obj {
  assert.equal(git(repo, "rev-parse", "HEAD"), CANDIDATE_SOURCE);
  assert.equal(git(repo, "rev-parse", "HEAD^{tree}"), CANDIDATE_TREE);
  assert.equal(git(repo, "status", "--porcelain=v1", "--untracked-files=all"), "");
  const inputs = Object.entries(SOURCE_PINS).map(([name, expected]) => {
    const actual = hash(read(path.join(repo, name), 16 * 2 ** 20));
    assert.equal(actual, expected, `source input ${name}`);
    return { path: name, sha256: actual };
  });
  return { source: CANDIDATE_SOURCE, tree: CANDIDATE_TREE, inputs };
}
function rows(file: string): Obj[] {
  return read(file)
    .toString("utf8")
    .split("\n")
    .filter(Boolean)
    .map((line) => object(JSON.parse(line)));
}
function restore(first: string, actual: string, expected: string): Obj {
  if (fs.existsSync(actual))
    assert.ok(fs.lstatSync(actual).isFile() && !fs.lstatSync(actual).isSymbolicLink());
  fs.copyFileSync(first, actual);
  fs.chmodSync(actual, 0o700);
  const restored = identity(actual);
  assert.equal(restored.sha256, expected);
  return restored;
}
function capacity(location: string): Obj {
  const checkedPath = fs.realpathSync(location);
  const stat = fs.statfsSync(checkedPath);
  const value = {
    checkedPath,
    freeDiskBytes: stat.bavail * stat.bsize,
    memoryBytes: os.totalmem(),
    cpus: os.cpus().length,
    availableParallelism: os.availableParallelism(),
  };
  assert.ok(
    value.freeDiskBytes >= 120 * 2 ** 30 && value.memoryBytes >= 32 * 2 ** 30 && value.cpus >= 32,
  );
  return value;
}

async function main(): Promise<void> {
  const repo = fs.realpathSync(process.cwd());
  const temp = fs.realpathSync(process.env.RUNNER_TEMP ?? "");
  const out = path.join(
    temp,
    `initial-canon-functional-${process.env.GITHUB_RUN_ID}-${process.env.GITHUB_RUN_ATTEMPT}`,
  );
  fs.mkdirSync(out, { mode: 0o700 });
  const report: Obj = {
    schemaVersion: 1,
    workflow: process.env.GITHUB_SHA,
    source: CANDIDATE_SOURCE,
    complete: false,
    accepted: false,
    attempts: [],
    errors: [],
  };
  const errors = report.errors as string[];
  let first: string | undefined;
  let actual: string | undefined;
  let firstDigest: string | undefined;
  try {
    const workflow = process.env.GITHUB_SHA ?? "";
    assert.match(workflow, SHA);
    assert.equal(workflow, process.env.EXPECTED_WORKFLOW);
    assert.equal(git(repo, "rev-parse", "HEAD"), workflow);
    checkParent(git(repo, "show", "-s", "--format=%P", "HEAD"), CANDIDATE_SOURCE);
    assert.deepEqual(
      git(repo, "diff", "--name-only", CANDIDATE_SOURCE, workflow).split("\n").sort(),
      [...TOOLING_PATHS].sort(),
    );
    checkAmbient(process.env);
    assert.equal(process.env.NODE_OPTIONS, "--disable-warning=DEP0040 --disable-warning=DEP0169");
    assert.equal(process.platform, "linux");
    assert.equal(process.arch, "x64");
    const helper = path.dirname(fileURLToPath(import.meta.url));
    const helperHashes: Obj[] = [];
    for (const name of TOOLING_PATHS.filter(
      (name) => name.endsWith(".ts") || name.endsWith("README.md"),
    )) {
      const staged = read(path.join(helper, path.basename(name)));
      const committed = spawnSync("git", ["show", `${workflow}:${name}`], { cwd: repo });
      assert.equal(committed.status, 0);
      assert.deepEqual(staged, committed.stdout);
      helperHashes.push({ path: name, sha256: hash(staged) });
    }
    save(path.join(out, "helper-hashes.json"), helperHashes);
    git(repo, "checkout", "--detach", CANDIDATE_SOURCE);
    const before = sourceSnapshot(repo);
    save(path.join(out, "source-before.json"), before);
    save(path.join(out, "capacity-before.json"), capacity(temp));
    const native = fs.realpathSync(
      path.join(repo, "node_modules/@typescript/typescript-linux-x64/lib/tsc"),
    );
    const provider = identity(native);
    assert.equal(provider.sha256, PROVIDER_SHA256);
    const manifest = object(
      JSON.parse(read(path.join(path.dirname(native), "../package.json")).toString("utf8")),
    );
    assert.equal(manifest.name, "@typescript/typescript-linux-x64");
    assert.equal(manifest.version, "7.0.2");
    save(path.join(out, "provider-before.json"), { ...provider, manifest });
    const target = path.join(
      temp,
      `initial-canon-target-${process.env.GITHUB_RUN_ID}-${process.env.GITHUB_RUN_ATTEMPT}`,
    );
    assert.ok(!fs.existsSync(target));
    const env = childEnv({ ...process.env, CARGO_TARGET_DIR: target }, native);
    report.environment = {
      CORSA_PATH: native,
      VIZE_TEST_REQUIRE_TSGO: env.VIZE_TEST_REQUIRE_TSGO,
      DISABLE: "UNDEFINED",
      CARGO_INCREMENTAL: env.CARGO_INCREMENTAL,
      CARGO_BUILD_JOBS: env.CARGO_BUILD_JOBS,
      RAYON_NUM_THREADS: env.RAYON_NUM_THREADS,
      profiling: false,
    };
    const version = run(repo, out, "native-version", native, ["--version"], env, 30_000);
    success(version);
    assert.equal(read(path.join(out, "native-version.stdout")).toString("utf8"), "Version 7.0.2\n");
    assert.equal(read(path.join(out, "native-version.stderr")).length, 0);
    const tools: Obj = { node: identity(process.execPath) };
    for (const [name, args] of [
      ["rustc", ["-vV"]],
      ["cargo", ["--version"]],
    ] as const) {
      success(run(repo, out, `${name}-version`, name, [...args], env, 30_000));
      success(run(repo, out, `${name}-selected`, "rustup", ["which", name], env, 30_000));
      const selected = read(path.join(out, `${name}-selected.stdout`)).toString("utf8");
      assert.ok(
        selected.endsWith("\n") &&
          !selected.slice(0, -1).includes("\n") &&
          path.isAbsolute(selected.trimEnd()),
      );
      tools[name] = identity(selected.trimEnd());
    }
    save(path.join(out, "tools-before.json"), tools);
    success(
      run(
        repo,
        out,
        "first-cli-build",
        "cargo",
        ["build", "--release", "-p", "vize", "--message-format=json-render-diagnostics"],
        env,
        60 * 60_000,
      ),
    );
    actual = path.join(target, "release/vize");
    const releaseRows = rows(path.join(out, "first-cli-build.stdout"));
    const binary = defaultCliArtifact(releaseRows, target, repo);
    const firstIdentity = identity(actual);
    firstDigest = String(firstIdentity.sha256);
    first = path.join(
      temp,
      `initial-canon-first-cli-${process.env.GITHUB_RUN_ID}-${process.env.GITHUB_RUN_ATTEMPT}`,
    );
    fs.copyFileSync(actual, first, fs.constants.COPYFILE_EXCL);
    fs.chmodSync(first, 0o500);
    save(path.join(out, "first-cli.json"), {
      source: CANDIDATE_SOURCE,
      recipe: "cargo build --release -p vize",
      binary,
      first: identity(first),
      actual: firstIdentity,
      sdk: sdkArtifacts(releaseRows),
      context: "default optimized CLI; distinct from ci library test",
    });
    const build = run(
      repo,
      out,
      "library-build",
      "cargo",
      [
        "test",
        "--locked",
        "--profile",
        "ci",
        "-p",
        "vize_canon",
        "--lib",
        "--features",
        "native",
        "--no-run",
        "--message-format=json-render-diagnostics",
      ],
      env,
      60 * 60_000,
    );
    save(path.join(out, "restored-after-library-build.json"), {
      exit: build,
      restored: restore(first, actual, firstDigest),
    });
    success(build);
    const cargoRows = rows(path.join(out, "library-build.stdout"));
    const library = libraryArtifact(cargoRows, target, repo);
    const executable = String(library.executable);
    const image = identity(executable);
    save(path.join(out, "library.json"), {
      source: CANDIDATE_SOURCE,
      library,
      image,
      sdk: sdkArtifacts(cargoRows),
      context: "ci libtest native-only functional qualification; no CLI or timing transfer",
    });
    const afterBuild = sourceSnapshot(repo);
    save(path.join(out, "source-after-build.json"), afterBuild);
    assert.deepEqual(
      afterBuild,
      before,
      "source and lock closure must hold before native execution",
    );
    const fixture = admittedFixture(read(path.join(repo, FIXTURE_PATH)));
    for (const test of TESTS) {
      assert.deepEqual(identity(executable), image);
      assert.deepEqual(identity(native), provider);
      const exit = run(
        repo,
        out,
        test.label,
        executable,
        [test.fqn, "--exact", "--nocapture", "--test-threads=1"],
        env,
        10 * 60_000,
      );
      const attempt: Obj = { ...test, exit, packets: [], errors: [] };
      (report.attempts as Obj[]).push(attempt);
      const after: Obj = {
        exit,
        image: identity(executable),
        provider: identity(native),
        source: sourceSnapshot(repo),
        restoredFirst: restore(first, actual, firstDigest),
      };
      // Complete stdout/stderr/status and post identities exist before verdicts.
      save(path.join(out, `${test.label}.after.json`), after);
      const failure: string[] = [];
      let packets: Obj[] = [];
      try {
        success(exit);
        assert.deepEqual(after.image, image);
        assert.deepEqual(after.provider, provider);
        assert.deepEqual(after.source, before);
        checkNativeExit(exit, read(path.join(out, `${test.label}.stderr`)).toString("utf8"));
        const stdout = read(path.join(out, `${test.label}.stdout`), 4 * 2 ** 20).toString("utf8");
        checkExecution(stdout, test.fqn);
        packets = checkPackets(test.label, stdout, native, fixture);
      } catch (error) {
        failure.push(error instanceof Error ? error.message : String(error));
      }
      attempt.packets = packets;
      attempt.errors = failure;
      errors.push(...failure.map((message) => `${test.label}: ${message}`));
      // A changed executable/provider/source is unsafe for dependent execution.
      assert.deepEqual(after.image, image);
      assert.deepEqual(after.provider, provider);
      assert.deepEqual(after.source, before);
    }
    report.complete = true;
    report.accepted = errors.length === 0;
    assert.equal(
      report.accepted,
      true,
      "both exact native controls and all six whole packets required",
    );
  } catch (error) {
    errors.push(error instanceof Error ? (error.stack ?? error.message) : String(error));
    process.exitCode = 1;
  } finally {
    if (first && actual && firstDigest) {
      try {
        save(path.join(out, "final-restored-first.json"), restore(first, actual, firstDigest));
      } catch (error) {
        errors.push(String(error));
        report.accepted = false;
        process.exitCode = 1;
      }
    }
    save(path.join(out, "outcome.json"), report);
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await main();
