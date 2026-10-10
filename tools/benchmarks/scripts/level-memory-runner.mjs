import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { parseTomlLite } from "../../support/compat/davinci/toml-lite.mjs";
import {
  fixtureDigest,
  GUEST_ENVIRONMENT,
  parseCallgrind,
  parseIdentities,
  reconcile,
  validateAllocatorSetup,
} from "./instruction-counts-lib.mjs";
import {
  collectFreshReports,
  MEMORY_ENVIRONMENT,
  memorySuites,
  NORMAL_ARGUMENTS,
  REPORT_DIRECTORY,
  sha256,
} from "./level-memory-lib.mjs";

export function writeJson(file, value) {
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
}

function executableIdentity(executable, cwd, env) {
  const candidates = executable.includes(path.sep)
    ? [path.resolve(cwd, executable)]
    : (env.PATH ?? "").split(path.delimiter).map((directory) => path.join(directory, executable));
  const file = candidates.find((candidate) => {
    try {
      fs.accessSync(candidate, fs.constants.X_OK);
      return fs.statSync(candidate).isFile();
    } catch {
      return false;
    }
  });
  if (!file) return null;
  const resolved = fs.realpathSync(file);
  return { requested: executable, path: file, resolved, sha256: sha256(fs.readFileSync(resolved)) };
}

// Save complete process streams and status before any exit or JSON assertion.
// The buffer ceiling matches the existing source-built instruction collector.
export function run(executable, args, cwd, directory, env = process.env) {
  fs.mkdirSync(directory, { recursive: true });
  const started = new Date().toISOString();
  const before = executableIdentity(executable, cwd, env);
  const result = spawnSync(executable, args, { cwd, env, maxBuffer: 128 * 1024 * 1024 });
  const stdout = result.stdout ?? Buffer.alloc(0);
  const stderr = result.stderr ?? Buffer.alloc(0);
  fs.writeFileSync(path.join(directory, "stdout.bin"), stdout);
  fs.writeFileSync(path.join(directory, "stderr.bin"), stderr);
  // Runner credentials are not recipe inputs and must never enter artifacts.
  const recipeKeys = [
    "PATH",
    "LANG",
    "LC_ALL",
    "RUSTUP_TOOLCHAIN",
    "RUSTFLAGS",
    "CARGO_INCREMENTAL",
    "RUSTC",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "VIZE_INSTRUCTION_COUNTS",
    "MIMALLOC_RESERVE_OS_MEMORY",
    "MIMALLOC_PURGE_DELAY",
    "MIMALLOC_VERBOSE",
    "GLIBC_TUNABLES",
  ];
  const recipeEnvironment = Object.fromEntries(
    recipeKeys.filter((key) => Object.hasOwn(env, key)).map((key) => [key, env[key]]),
  );
  const after = executableIdentity(executable, cwd, env);
  writeJson(path.join(directory, "process.json"), {
    executable,
    args,
    cwd,
    recipeEnvironment,
    imageBefore: before,
    imageAfter: after,
    started,
    finished: new Date().toISOString(),
    status: result.status,
    signal: result.signal,
    pid: result.pid,
    error: result.error
      ? { name: result.error.name, message: result.error.message, code: result.error.code }
      : null,
    stdout: { bytes: stdout.length, sha256: sha256(stdout) },
    stderr: { bytes: stderr.length, sha256: sha256(stderr) },
  });
  assert.equal(
    result.error,
    undefined,
    `${executable}: spawn/buffer failure; raw process retained`,
  );
  assert.equal(result.signal, null, `${executable}: signal ${result.signal}; raw process retained`);
  assert.equal(result.status, 0, `${executable}: exit ${result.status}; raw process retained`);
  assert.ok(before, `${executable}: executable resolution missing`);
  assert.deepEqual(after, before, `${executable}: executable image changed`);
  return { stdout: stdout.toString(), stderr: stderr.toString() };
}

export function build(root, target, out) {
  const toolchain = parseTomlLite(fs.readFileSync(path.join(root, "rust-toolchain.toml"), "utf8"))
    .toolchain.channel;
  assert.equal(toolchain, "1.99.0", "same-input memory recipe requires pinned Rust 1.99.0");
  const compiler = run(
    "rustup",
    ["which", "--toolchain", toolchain, "rustc"],
    root,
    path.join(out, "resolve-rustc"),
  ).stdout.trim();
  const cargo = run(
    "rustup",
    ["which", "--toolchain", toolchain, "cargo"],
    root,
    path.join(out, "resolve-cargo"),
  ).stdout.trim();
  assert.ok(
    path.isAbsolute(compiler) && path.isAbsolute(cargo),
    "pinned toolchain image resolution missing",
  );
  const rust = run(compiler, ["-Vv"], root, path.join(out, "rustc"));
  assert.match(rust.stdout, /^rustc 1\.99\.0 /);
  const suites = memorySuites(root);
  assert.equal(suites.length, 13, "original twelve level and one formatter providers required");
  const args = [
    "bench",
    "--locked",
    "--no-run",
    "--profile",
    "ci-opt",
    "--features",
    "davinci_harness/instruction-counts",
    "--message-format=json",
    "--target-dir",
    target,
  ];
  for (const pkg of new Set(suites.map(({ pkg }) => pkg))) args.push("-p", pkg);
  for (const bench of new Set(suites.map(({ bench }) => bench))) args.push("--bench", bench);
  const built = run(cargo, args, root, path.join(out, "cargo"), {
    ...process.env,
    RUSTC: compiler,
    RUSTC_WRAPPER: "",
    RUSTC_WORKSPACE_WRAPPER: "",
    RUSTUP_TOOLCHAIN: toolchain,
    CARGO_INCREMENTAL: "0",
    RUSTFLAGS: "-C target-cpu=x86-64",
  });
  const artifacts = built.stdout
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line))
    .filter(
      (row) =>
        row.reason === "compiler-artifact" && row.executable && row.target.kind.includes("bench"),
    );
  const binaries = suites.map(({ plane, pkg, bench }) => {
    const directory =
      pkg === "davinci_harness"
        ? "tools/benchmarks/crates"
        : fs.existsSync(path.join(root, "davinci", pkg))
          ? "davinci"
          : "crates";
    const source = path.join(root, directory, pkg, "benches", `${bench}.rs`);
    const matches = artifacts.filter((row) => path.resolve(row.target.src_path) === source);
    assert.equal(matches.length, 1, `${pkg}/${bench}: missing or duplicate source-built artifact`);
    const artifact = matches[0];
    const executable = fs.realpathSync(artifact.executable);
    assert.ok(
      executable.startsWith(`${target}${path.sep}`),
      "executable outside isolated Cargo target",
    );
    assert.ok(fs.statSync(executable).isFile(), "benchmark executable is not regular");
    return {
      plane,
      pkg,
      bench,
      source,
      executable,
      fresh: artifact.fresh,
      source_sha256: sha256(fs.readFileSync(source)),
      sha256: sha256(fs.readFileSync(executable)),
    };
  });
  writeJson(path.join(out, "binaries.json"), binaries);
  return binaries;
}

export function bindProbe(suite, index, directory) {
  const file = path.join(directory, `probe-${String(index).padStart(2, "0")}`);
  if (fs.existsSync(file)) {
    assert.ok(fs.lstatSync(file).isSymbolicLink(), "refusing to replace foreign probe");
    fs.unlinkSync(file);
  }
  fs.symlinkSync(suite.executable, file);
  assert.equal(
    sha256(fs.readFileSync(file)),
    suite.sha256,
    "probe differs from source-built executable",
  );
  return file;
}

export function collectIdentities(root, suite, probe, out, budgets) {
  fs.mkdirSync(out, { recursive: true });
  const prefix = path.join(out, "identity.callgrind");
  const result = run(
    "valgrind",
    [
      "--tool=callgrind",
      "--instr-atstart=no",
      "--cache-sim=no",
      "--branch-sim=no",
      `--callgrind-out-file=${prefix}`,
      "--error-exitcode=97",
      probe,
    ],
    root,
    path.join(out, "process"),
    GUEST_ENVIRONMENT,
  );
  validateAllocatorSetup(result.stderr);
  const raw = parseIdentities(result.stderr);
  const dumps = new Map();
  for (const file of fs
    .readdirSync(out)
    .filter((name) => name === "identity.callgrind" || /^identity\.callgrind\.\d+$/.test(name))) {
    const dump = parseCallgrind(fs.readFileSync(path.join(out, file), "utf8"));
    if (!dump) continue;
    assert.ok(!dumps.has(dump.bench_id), "duplicate identity dump");
    dumps.set(dump.bench_id, dump.instructions);
  }
  reconcile(new Set(dumps.keys()), new Set(raw.keys()), "actual source/window Callgrind dumps");
  const identities = new Map();
  for (const [id, identity] of raw) {
    const limit = budgets.instruction[id];
    assert.ok(limit, `unregistered actual identity ${id}`);
    const digest = fixtureDigest(root, identity.fixture, suite.source);
    assert.equal(digest, limit.fixture_sha256, `${id}: original input bytes changed`);
    assert.equal(identity.window, limit.window, `${id}: original window changed`);
    identities.set(id, { ...identity, fixture_sha256: digest });
  }
  writeJson(path.join(out, "identities.json"), Object.fromEntries(identities));
  // This one pass authenticates identities; the separate three-run instruction
  // gate remains mandatory and is the only instruction qualification authority.
  return identities;
}

export function collectNormal(root, suite, probe, out, identities) {
  fs.mkdirSync(out, { recursive: true });
  const result = run(probe, NORMAL_ARGUMENTS, root, path.join(out, "process"), MEMORY_ENVIRONMENT);
  assert.doesNotMatch(
    result.stderr,
    /^VIZE_INSTRUCTION_(BENCH|ALLOCATOR) /m,
    "instruction mode leaked into normal memory run",
  );
  assert.match(
    result.stderr,
    /option 'reserve_os_memory': 131072 KiB/,
    "actual normal allocator reservation option missing",
  );
  assert.match(
    result.stderr,
    /option 'purge_delay': -1\s/,
    "actual normal allocator purge option changed",
  );
  const rows = collectFreshReports(root, result.stderr, identities);
  const reports = path.join(out, "reports");
  fs.mkdirSync(reports);
  for (const [id, row] of rows) {
    const rawFile = path.join(reports, `${id}.json`);
    fs.copyFileSync(row.file, rawFile);
    assert.equal(sha256(fs.readFileSync(rawFile)), row.raw_sha256, "raw report copy changed");
    row.raw_file = rawFile;
  }
  assert.equal(sha256(fs.readFileSync(suite.executable)), suite.sha256, "executed binary changed");
  assert.equal(
    sha256(fs.readFileSync(suite.source)),
    suite.source_sha256,
    "benchmark provider changed",
  );
  writeJson(path.join(out, "rows.json"), Object.fromEntries(rows));
  return rows;
}

export function preserveExistingReports(root, out) {
  const directory = path.join(root, REPORT_DIRECTORY);
  fs.mkdirSync(out, { recursive: true });
  if (!fs.existsSync(directory)) return;
  assert.ok(!fs.lstatSync(directory).isSymbolicLink(), "preexisting report directory is symlinked");
  for (const file of fs.readdirSync(directory).filter((name) => name.endsWith(".json"))) {
    const source = path.join(directory, file);
    assert.ok(
      fs.lstatSync(source).isFile() && !fs.lstatSync(source).isSymbolicLink(),
      "preexisting report is not regular",
    );
    fs.copyFileSync(source, path.join(out, file));
  }
}
