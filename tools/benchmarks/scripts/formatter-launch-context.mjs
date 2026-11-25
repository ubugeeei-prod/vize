// One build copied into launch contexts, not independent-build qualification.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";
import {
  fixtureDigest,
  GUEST_ENVIRONMENT,
  parseCallgrind,
  parseIdentities,
  validateAllocatorSetup,
} from "./instruction-counts-lib.mjs";

export const AFFECTED = "formatter_script_large";
const PROVIDER = "crates/vize_glyph/benches/formatter_instructions.rs";
const ids = ["formatter_sfc_simple", "formatter_sfc_reuse", AFFECTED, "formatter_template_complex"];
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");

export function replayPlan(temporary) {
  const old = (role) =>
    path.join(
      temporary,
      "release-measure/out",
      role,
      "formatter/formatter-instruction-probes/probe-00",
    );
  const fixed = path.join(temporary, "formatter-launch-fixed/probe-00");
  const paths = {
    "old-primary": old("base"),
    "old-twin": old("base-twin"),
    "fixed-primary": fixed,
    "fixed-twin": fixed,
  };
  const orders = [
    ["old-primary", "old-twin", "fixed-primary", "fixed-twin"],
    ["fixed-twin", "fixed-primary", "old-twin", "old-primary"],
    ["old-primary", "fixed-primary", "old-twin", "fixed-twin"],
  ];
  return orders.flatMap((roles, index) =>
    roles.map((role) => ({
      repetition: index + 1,
      role,
      executable: paths[role],
      method: role.split("-")[0],
    })),
  );
}

export function inspectDump(text, expectedExecutable) {
  const argv0 = text.match(/^cmd:\s*(.+)$/mu)?.[1];
  assert.equal(
    argv0,
    expectedExecutable,
    "observed Callgrind guest command must match argv0 with no guest arguments",
  );
  const dump = parseCallgrind(text);
  assert.ok(dump, "expected named client-request dump");
  return { ...dump, argv0 };
}

export function fixedCounts(runs) {
  const selected = runs.filter((run) => run.method === "fixed");
  assert.equal(selected.length, 6, "retain all three fixed primary/twin pairs");
  return [...new Set(selected.map((run) => run.rows[AFFECTED].instructions))];
}

export function summarizeContexts(runs) {
  const counts = (role) => [
    ...new Set(
      runs.filter((run) => run.role === role).map((run) => run.rows[AFFECTED].instructions),
    ),
  ];
  const oldRoleCounts = { primary: counts("old-primary"), twin: counts("old-twin") };
  const oldAffectedCounts = [...new Set([...oldRoleCounts.primary, ...oldRoleCounts.twin])];
  const fixedAffectedCounts = fixedCounts(runs);
  const oldRoleDelta =
    oldRoleCounts.primary.length === 1 && oldRoleCounts.twin.length === 1
      ? oldRoleCounts.twin[0] - oldRoleCounts.primary[0]
      : null;
  return {
    oldAffectedCounts,
    oldRoleCounts,
    oldRoleDelta,
    oldDriftReproduced: oldRoleDelta === 14,
    fixedAffectedCounts,
    fixedPathStable: fixedAffectedCounts.length === 1,
  };
}

export function prepareProbes(plan, preserved, binaryHash) {
  const fixed = plan.find((run) => run.method === "fixed").executable;
  for (const executable of new Set(plan.map((run) => run.executable))) {
    assert.ok(!fs.existsSync(path.dirname(executable)), "refuse existing launch context");
    fs.mkdirSync(path.dirname(executable), { recursive: true });
    if (executable === fixed) {
      fs.copyFileSync(preserved, executable, fs.constants.COPYFILE_EXCL);
      fs.chmodSync(executable, 0o755);
      assert.ok(fs.lstatSync(executable).isFile() && !fs.lstatSync(executable).isSymbolicLink());
    } else fs.symlinkSync(preserved, executable);
    assert.equal(digest(fs.readFileSync(executable)), binaryHash);
  }
}

function execute(binary, args, cwd, env, prefix) {
  const result = spawnSync(binary, args, { cwd, env, maxBuffer: 128 * 1024 * 1024 });
  fs.writeFileSync(`${prefix}.stdout`, result.stdout ?? Buffer.alloc(0));
  fs.writeFileSync(`${prefix}.stderr`, result.stderr ?? Buffer.alloc(0));
  fs.writeFileSync(
    `${prefix}.process.json`,
    JSON.stringify(
      {
        binary,
        args,
        cwd,
        environment: env === GUEST_ENVIRONMENT ? env : undefined,
        status: result.status,
        signal: result.signal,
        error: result.error?.message,
      },
      null,
      2,
    ),
  );
  assert.equal(result.status, 0, `${binary} failed; complete streams retained at ${prefix}`);
  return { stdout: result.stdout.toString(), stderr: result.stderr.toString() };
}

export function main(source, out, temporary, sha, epoch) {
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.match(sha, /^[0-9a-f]{40}$/u);
  assert.match(epoch, /^\d+$/u);
  assert.ok(!fs.existsSync(out), "refuse existing diagnostic output");
  fs.mkdirSync(out, { recursive: true });
  const query = (args, label = args[0]) =>
    execute("git", args, source, process.env, path.join(out, `git-${label}`)).stdout.trim();
  assert.equal(query(["rev-parse", "HEAD"]), sha);
  assert.equal(
    query(["status", "--porcelain"], "status-initial"),
    "",
    "source checkout must be clean",
  );
  const toolchain = execute(
    "rustc",
    ["+1.100.0", "--version"],
    source,
    process.env,
    path.join(out, "rustc"),
  ).stdout.trim();
  assert.equal(toolchain, "rustc 1.100.0 (0ef1bc386 2026-11-09)");
  assert.equal(
    execute(
      "valgrind",
      ["--version"],
      source,
      process.env,
      path.join(out, "valgrind"),
    ).stdout.trim(),
    "valgrind-3.22.0",
  );
  const libc = execute(
    "getconf",
    ["GNU_LIBC_VERSION"],
    source,
    process.env,
    path.join(out, "libc"),
  ).stdout.trim();
  assert.equal(libc, "glibc 2.39");
  const buildEnvironment = {
    ...process.env,
    CARGO_TARGET_DIR: path.join(out, "target"),
    CARGO_INCREMENTAL: "0",
    SOURCE_DATE_EPOCH: epoch,
    RUSTFLAGS: "-C target-cpu=x86-64",
    CARGO_PROFILE_CI_OPT_LTO: "fat",
    CARGO_PROFILE_CI_OPT_CODEGEN_UNITS: "1",
  };
  const build = execute(
    "cargo",
    [
      "+1.100.0",
      "bench",
      "--locked",
      "--no-run",
      "--profile",
      "ci-opt",
      "--features",
      "davinci_harness/instruction-counts",
      "--message-format=json",
      "-p",
      "vize_glyph",
      "--bench",
      "formatter_instructions",
    ],
    source,
    buildEnvironment,
    path.join(out, "build"),
  );
  const artifacts = build.stdout
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line))
    .filter(
      (row) =>
        row.reason === "compiler-artifact" &&
        row.executable &&
        path.resolve(row.target.src_path) === path.join(source, PROVIDER),
    );
  assert.equal(artifacts.length, 1);
  assert.equal(artifacts[0].fresh, false, "the single probe must be newly built");
  const original = artifacts[0].executable;
  const binaryHash = digest(fs.readFileSync(original));
  const preserved = path.join(out, "formatter-probe");
  fs.copyFileSync(original, preserved, fs.constants.COPYFILE_EXCL);
  const plan = replayPlan(temporary);
  prepareProbes(plan, preserved, binaryHash);
  const captureDirectory = path.join(out, "capture-current");
  assert.ok(!fs.existsSync(captureDirectory), "refuse existing raw capture directory");
  fs.mkdirSync(captureDirectory);
  // Keep even the Valgrind output argument fixed; move completed dumps only
  // after each process exits, so the execution-path treatment is isolated.
  const callgrindPrefix = path.join(captureDirectory, "formatter.callgrind");
  const providerFiles = query([
    "ls-files",
    "tools/benchmarks/crates/davinci_harness/src",
    "crates/vize_glyph/benches",
    "upstream/forks/oxc_formatter/src/source_text.rs",
    "Cargo.lock",
  ])
    .split("\n")
    .filter(Boolean);
  const receipt = {
    schema: "vize.formatter-launch-context-diagnostic.v1",
    source: sha,
    scope:
      "one build with copied bytes; no lost-binary recovery, independent-build or whole-campaign credit",
    sourceDateEpoch: epoch,
    toolchain,
    libc,
    binaryHash,
    builtExecutable: original,
    guestEnvironment: GUEST_ENVIRONMENT,
    buildProfile: {
      name: "ci-opt",
      lto: "fat",
      codegenUnits: 1,
      rustflags: buildEnvironment.RUSTFLAGS,
    },
    cwd: source,
    plan,
    providerHash: digest(fs.readFileSync(path.join(source, PROVIDER))),
    providerHashes: Object.fromEntries(
      providerFiles.map((file) => [file, digest(fs.readFileSync(path.join(source, file)))]),
    ),
    fixtureIdentities: {},
    runs: [],
  };
  const retain = () =>
    fs.writeFileSync(path.join(out, "receipt.json"), JSON.stringify(receipt, null, 2));
  retain();
  for (const [index, run] of plan.entries()) {
    const directory = path.join(out, `${String(index + 1).padStart(2, "0")}-${run.role}`);
    fs.mkdirSync(directory);
    assert.equal(digest(fs.readFileSync(run.executable)), binaryHash);
    assert.deepEqual(fs.readdirSync(captureDirectory), []);
    const result = execute(
      "valgrind",
      [
        "--tool=callgrind",
        "--instr-atstart=no",
        "--cache-sim=no",
        "--branch-sim=no",
        `--callgrind-out-file=${callgrindPrefix}`,
        "--error-exitcode=97",
        run.executable,
      ],
      source,
      GUEST_ENVIRONMENT,
      path.join(directory, "process"),
    );
    for (const file of fs.readdirSync(captureDirectory))
      fs.renameSync(path.join(captureDirectory, file), path.join(directory, file));
    validateAllocatorSetup(result.stderr);
    const identities = parseIdentities(result.stderr);
    assert.deepEqual([...identities.keys()].sort(), [...ids].sort());
    const rows = {};
    for (const file of fs
      .readdirSync(directory)
      .filter((name) => /^formatter\.callgrind\.\d+$/u.test(name))) {
      const dump = inspectDump(fs.readFileSync(path.join(directory, file), "utf8"), run.executable);
      const identity = identities.get(dump.bench_id);
      assert.ok(identity && !Object.hasOwn(rows, dump.bench_id));
      const fixtureHash = fixtureDigest(source, identity.fixture, path.join(source, PROVIDER));
      const input = { fixtureHash, fixture: identity.fixture, window: identity.window };
      receipt.fixtureIdentities[dump.bench_id] ??= input;
      assert.deepEqual(input, receipt.fixtureIdentities[dump.bench_id]);
      rows[dump.bench_id] = {
        instructions: dump.instructions,
        fixtureHash,
        window: identity.window,
        observedArgv0: dump.argv0,
        argv0Bytes: Buffer.byteLength(dump.argv0),
        raw: file,
      };
    }
    assert.deepEqual(Object.keys(rows).sort(), [...ids].sort());
    assert.equal(digest(fs.readFileSync(run.executable)), binaryHash);
    receipt.runs.push({ ...run, rows });
    retain();
  }
  Object.assign(receipt, summarizeContexts(receipt.runs));
  assert.equal(
    query(["status", "--porcelain"], "status-final"),
    "",
    "diagnosis must leave source clean",
  );
  assert.equal(digest(fs.readFileSync(original)), binaryHash);
  retain();
  console.log(
    JSON.stringify({
      source: sha,
      binaryHash,
      oldAffectedCounts: receipt.oldAffectedCounts,
      oldRoleDelta: receipt.oldRoleDelta,
      oldDriftReproduced: receipt.oldDriftReproduced,
      fixedPathStable: receipt.fixedPathStable,
      fixedAffectedCounts: receipt.fixedAffectedCounts,
      observations: receipt.runs.map((run) => ({
        role: run.role,
        repetition: run.repetition,
        instructions: run.rows[AFFECTED].instructions,
        argv0: run.rows[AFFECTED].observedArgv0,
      })),
    }),
  );
  assert.equal(
    receipt.fixedAffectedCounts.length,
    1,
    "fixed-path A/A still varies; hypothesis incomplete",
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const source = path.resolve(process.argv[2]);
  main(
    source,
    path.join(process.env.RUNNER_TEMP, "formatter-launch-diagnostic"),
    process.env.RUNNER_TEMP,
    process.env.SOURCE_SHA,
    process.env.SOURCE_DATE_EPOCH,
  );
}
