import assert from "node:assert/strict";
import {
  closeSync,
  existsSync,
  mkdirSync,
  openSync,
  readFileSync,
  writeFileSync,
  appendFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { loadavg } from "node:os";
import { spawnSync } from "node:child_process";
import {
  BASE_SHA,
  BENCH_PATH,
  BENCH_SHA256,
  HEAD_SHA,
  LOCK_SHA256,
  SETTINGS,
  TOOLCHAIN,
  buildArgs,
  measureArgs,
  pairPlan,
  sha256,
  validateBuildEnvironment,
  validateFrozenFiles,
  validateRevisionInputs,
} from "./sfc-parse-replay-contract.mjs";
import { renderSummary, summarizeRunner } from "./sfc-parse-replay-summary.mjs";

const [baseDirectory, headDirectory, outputDirectory, runnerText] = process.argv.slice(2);
assert.ok(
  baseDirectory && headDirectory && outputDirectory && runnerText,
  "usage: replay BASE HEAD OUTPUT RUNNER",
);
const runner = Number(runnerText);
assert.ok([1, 2, 3].includes(runner));
const out = resolve(outputDirectory);
mkdirSync(out, { recursive: true });
validateRevisionInputs({
  base: process.env.REPLAY_BASE_SHA,
  head: process.env.REPLAY_HEAD_SHA,
  conflictingModes: process.env.REPLAY_CONFLICTING_MODES !== "false",
});
validateBuildEnvironment(process.env);
const targets = resolve(dirname(out), "sfc-parse-replay-targets");
assert.ok(!existsSync(targets), "replay requires fresh isolated target directories");
mkdirSync(targets);

function command(commandName, args, label, { cwd, env = {} } = {}) {
  const stdoutPath = join(out, `${label}.stdout`);
  const stderrPath = join(out, `${label}.stderr`);
  mkdirSync(dirname(stdoutPath), { recursive: true });
  const stdout = openSync(stdoutPath, "w");
  const stderr = openSync(stderrPath, "w");
  const startedAt = new Date().toISOString();
  const started = process.hrtime.bigint();
  let result;
  try {
    result = spawnSync(commandName, args, {
      cwd,
      env: { ...process.env, ...env },
      stdio: ["ignore", stdout, stderr],
      timeout: 20 * 60 * 1000,
    });
  } finally {
    closeSync(stdout);
    closeSync(stderr);
  }
  appendFileSync(
    join(out, "commands.jsonl"),
    `${JSON.stringify({
      command: commandName,
      args,
      cwd,
      env,
      startedAt,
      finishedAt: new Date().toISOString(),
      elapsedNs: String(process.hrtime.bigint() - started),
      status: result.status,
      error: result.error?.message,
      stdoutPath,
      stderrPath,
    })}\n`,
  );
  assert.equal(result.error, undefined, `${label}: ${result.error?.message}`);
  assert.equal(result.status, 0, `${label}: ${readFileSync(stderrPath, "utf8").slice(-6000)}`);
  return readFileSync(stdoutPath, "utf8");
}

const sides = {
  base: { directory: resolve(baseDirectory), sha: BASE_SHA },
  head: { directory: resolve(headDirectory), sha: HEAD_SHA },
};
for (const [side, data] of Object.entries(sides)) {
  assert.equal(
    command("git", ["rev-parse", "HEAD"], `${side}-sha`, { cwd: data.directory }).trim(),
    data.sha,
  );
  assert.equal(
    command("git", ["status", "--porcelain"], `${side}-status`, { cwd: data.directory }).trim(),
    "",
  );
  validateFrozenFiles(
    side,
    readFileSync(join(data.directory, BENCH_PATH)),
    readFileSync(join(data.directory, "Cargo.lock")),
    readFileSync(join(data.directory, "rust-toolchain.toml"), "utf8"),
    readFileSync(join(data.directory, "Cargo.toml"), "utf8"),
  );
  data.target = join(targets, side);
}
command("git", ["merge-base", "--is-ancestor", BASE_SHA, HEAD_SHA], "ancestry", {
  cwd: sides.head.directory,
});
const compiler = command("rustc", [`+${TOOLCHAIN}`, "--version", "--verbose"], "rustc");
assert.match(compiler, /^release: 1\.98\.0$/mu);
assert.match(compiler, /^host: x86_64-unknown-linux-gnu$/mu);
command("cargo", [`+${TOOLCHAIN}`, "--version", "--verbose"], "cargo");
command("uname", ["-a"], "uname");
command("lscpu", [], "lscpu");
command("clang", ["--version"], "clang");
const wild = join(process.env.RUNNER_TEMP, "wild-install", "wild");
assert.match(command(wild, ["--version"], "wild"), /\b0\.9\.0\b/u);
const globalConfig = readFileSync(join(process.env.CARGO_HOME, "config.toml"), "utf8");
assert.ok(globalConfig.includes(wild), "Wild linker must be configured for the build");
writeFileSync(join(out, "cargo-config.toml"), globalConfig);
writeFileSync(
  join(out, "environment.json"),
  `${JSON.stringify(
    {
      runner,
      node: process.version,
      platform: process.platform,
      arch: process.arch,
      runId: process.env.GITHUB_RUN_ID,
      runAttempt: process.env.GITHUB_RUN_ATTEMPT,
      harnessSha: process.env.GITHUB_SHA,
      runnerName: process.env.RUNNER_NAME,
      addressSpaceRandomization: readFileSync("/proc/sys/kernel/randomize_va_space", "utf8").trim(),
      compiler,
      rustupToolchain: process.env.RUSTUP_TOOLCHAIN,
      cargoIncremental: process.env.CARGO_INCREMENTAL,
      buildOrder: runner === 2 ? ["head", "base"] : ["base", "head"],
      settings: SETTINGS,
      plan: pairPlan(),
    },
    null,
    2,
  )}\n`,
);

// Cargo work and symbol inspection finish before the first measured pair.
for (const side of runner === 2 ? ["head", "base"] : ["base", "head"]) {
  const data = sides[side];
  console.log(`Build ${side} at ${data.sha} with Rust ${TOOLCHAIN}`);
  const jsonLines = command("cargo", buildArgs(data.target), `${side}-build`, {
    cwd: data.directory,
    env: { CARGO_TARGET_DIR: data.target },
  });
  const artifacts = jsonLines
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line))
    .filter(
      (row) =>
        row.reason === "compiler-artifact" && row.target.name === "sfc_parse" && row.executable,
    );
  assert.equal(artifacts.length, 1, "exactly one SFC parse executable is required");
  data.binary = artifacts[0].executable;
  data.binarySha256 = sha256(readFileSync(data.binary));
  data.binaryBytes = readFileSync(data.binary).length;
  const symbols = command("nm", ["-S", "-n", "-C", data.binary], `${side}-symbols`);
  writeFileSync(
    join(out, `${side}-parser-symbols.txt`),
    `${symbols
      .split("\n")
      .filter((line) => line.includes("sfc::parse::"))
      .join("\n")}\n`,
  );
  command("objdump", ["-d", "-C", data.binary], `${side}-disassembly`);
  command(data.binary, ["--bench", "--list"], `${side}-cases`, {
    cwd: data.directory,
    env: { CARGO_TARGET_DIR: data.target, CRITERION_HOME: join(out, `${side}-list`) },
  });
  assert.equal(sha256(readFileSync(join(data.directory, "Cargo.lock"))), LOCK_SHA256[side]);
  writeFileSync(join(out, `${side}-Cargo.toml`), readFileSync(join(data.directory, "Cargo.toml")));
  writeFileSync(join(out, `${side}-Cargo.lock`), readFileSync(join(data.directory, "Cargo.lock")));
  writeFileSync(
    join(out, `${side}-rust-toolchain.toml`),
    readFileSync(join(data.directory, "rust-toolchain.toml")),
  );
  writeFileSync(
    join(out, `${side}-repository-cargo-config.toml`),
    readFileSync(join(data.directory, ".cargo/config.toml")),
  );
}
writeFileSync(
  join(out, "frozen-sfc_parse.rs"),
  readFileSync(join(sides.base.directory, BENCH_PATH)),
);
const report = {
  schema: 1,
  runner,
  baseSha: BASE_SHA,
  headSha: HEAD_SHA,
  benchmarkSha256: BENCH_SHA256,
  compiler,
  settings: SETTINGS,
  binaries: Object.fromEntries(
    Object.entries(sides).map(([side, data]) => [
      side,
      {
        path: data.binary,
        sha256: data.binarySha256,
        bytes: data.binaryBytes,
      },
    ]),
  ),
  startedAt: new Date().toISOString(),
  observations: [],
};
const reportPath = join(out, "samples.json");
writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
for (const { pair, cases, sides: order } of pairPlan()) {
  for (const name of cases) {
    for (const [position, side] of order.entries()) {
      const data = sides[side];
      const relativeHome = `raw/pair-${pair}/${name}/${side}`;
      const home = join(out, relativeHome);
      const startedAt = new Date().toISOString();
      const loadBefore = loadavg();
      console.log(`Pair ${pair + 1}/6 ${name}: ${side} (${position + 1}/2)`);
      command(data.binary, measureArgs(name), `${relativeHome}/execution`, {
        cwd: data.directory,
        env: { CARGO_TARGET_DIR: data.target, CRITERION_HOME: home },
      });
      const baseline = join(home, "sfc_parse", name, "observed");
      const sample = JSON.parse(readFileSync(join(baseline, "sample.json"), "utf8"));
      const estimates = JSON.parse(readFileSync(join(baseline, "estimates.json"), "utf8"));
      const benchmark = JSON.parse(readFileSync(join(baseline, "benchmark.json"), "utf8"));
      assert.equal(sample.iters.length, SETTINGS.sampleSize);
      assert.equal(sample.times.length, SETTINGS.sampleSize);
      assert.ok(sample.iters.every((value) => Number.isFinite(value) && value > 0));
      assert.ok(sample.times.every((value) => Number.isFinite(value) && value > 0));
      assert.equal(benchmark.group_id, "sfc_parse");
      assert.equal(benchmark.function_id, name);
      report.observations.push({
        pair,
        name,
        side,
        position,
        startedAt,
        finishedAt: new Date().toISOString(),
        loadBefore,
        loadAfter: loadavg(),
        criterionHome: relativeHome,
        sample,
        estimates,
        benchmark,
      });
      writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
    }
  }
}
report.finishedAt = new Date().toISOString();
writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
const summary = summarizeRunner(report);
writeFileSync(join(out, "summary.json"), `${JSON.stringify(summary, null, 2)}\n`);
writeFileSync(join(out, "summary.md"), renderSummary(summary));
console.log(renderSummary(summary));
