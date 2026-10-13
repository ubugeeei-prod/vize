import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const prefix = "lsp_client::editor_lsp::native_shutdown::owned_image::tests::";
export const names = [
  "absent_ambiguous_completed_or_failed_ownership_never_authenticates_an_image",
  "full_pid_parent_birth_and_live_state_are_required_even_with_parenthesized_names",
  "an_actual_owned_foreign_image_and_a_reaped_child_do_not_get_native_capability",
  "actual_exec_wrapper_uses_only_its_owned_initialized_native_image",
];
const nativeSHA256 = "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4";
export function validatePackets(packets, stdout, runtime) {
  assert.equal(runtime.requireNative, "1");
  assert.equal(runtime.disableNative, null);
  assert.equal(runtime.nativeSHA256, nativeSHA256);
  assert.equal(runtime.platform, "linux");
  assert.equal(runtime.arch, "x64");
  assert.equal(runtime.nativeVersion, "Version 7.0.2");
  assert.equal(runtime.package.name, "@typescript/typescript-linux-x64");
  assert.equal(runtime.package.version, "7.0.2");
  assert.equal(packets.length, 4);
  assert.deepEqual(packets.map(([name]) => name).toSorted(), [
    "foreign.json",
    "life.json",
    "native.json",
    "ownership.json",
  ]);
  for (const name of names) assert.ok(stdout.includes(`test ${prefix}${name} ... ok`));
  assert.ok(stdout.includes("test result: ok. 4 passed; 0 failed; 0 ignored;"));
  assert.equal((stdout.match(/^test .* \.\.\. ok$/gmu) ?? []).length, 4);
  const packet = Object.fromEntries(packets);
  const ownership = packet["ownership.json"];
  assert.equal(ownership.wholeDeniedReceipts.length, 9);
  const live = ownership.original;
  assert.equal(live.pid, 41);
  for (const key of ["completed", "stderr_complete", "timed_out", "kill_requested", "reaped"])
    assert.equal(live[key], false);
  for (const key of ["stderr_error", "exit_status", "process_error"]) assert.equal(live[key], null);
  assert.deepEqual(live.stderr, []);
  assert.deepEqual(
    ownership.wholeDeniedReceipts.map((row) => row.pid),
    [41, 41, 41, 41, 41, 41, 41, 41, 0],
  );
  const fields = [
    "completed",
    "stderr_complete",
    "stderr_error",
    "exit_status",
    "process_error",
    "timed_out",
    "kill_requested",
    "reaped",
    "pid",
  ];
  for (const [index, row] of ownership.wholeDeniedReceipts.entries()) {
    assert.notDeepEqual(row[fields[index]], live[fields[index]]);
    for (const key of Object.keys(live))
      if (key !== fields[index]) assert.deepEqual(row[key], live[key]);
  }
  assert.equal(ownership.absentObserver, false);
  assert.equal(ownership.selfImage, null);
  assert.equal(ownership.missingPid, 4294967295);
  assert.equal(ownership.missingImage, null);
  const life = packet["life.json"];
  assert.equal(life.pid, 41);
  assert.equal(life.birth, 512);
  assert.ok(life.parent > 0);
  assert.ok(life.wholePositiveStat.startsWith(`41 (native (exec)) S ${life.parent} `));
  assert.deepEqual(life.deniedStates, ["Z", "X", "?"]);
  assert.equal(life.foreignParent, life.parent + 1);
  assert.equal(life.deniedBirth, 0);
  assert.equal(life.deniedPid, 42);
  const foreign = packet["foreign.json"];
  assert.ok(foreign.pid > 0);
  assert.ok(foreign.wholeLiveStat.startsWith(`${foreign.pid} (`));
  assert.match(foreign.imageSHA256, /^[a-f0-9]{64}$/u);
  assert.notEqual(foreign.imageSHA256, nativeSHA256);
  assert.equal(foreign.knownNative, false);
  assert.equal(foreign.terminalRaw, 0);
  assert.equal(foreign.reapedImage, null);
  assert.equal(foreign.killRequested, false);
  const native = packet["native.json"];
  assert.ok(native.pid > 0);
  assert.ok(native.wholeLiveStat.startsWith(`${native.pid} (`));
  assert.equal(native.imageSHA256, nativeSHA256);
  assert.equal(fs.realpathSync(native.nativePath), runtime.nativeBinary);
  assert.equal(native.wholeWrapper, runtime.wholeWrapper);
  assert.deepEqual(native.wholeInitialize, runtime.wholeInitialize);
  assert.deepEqual(native.wholeChangedInitialize, {
    ...runtime.wholeInitialize,
    capabilities: { ...runtime.wholeInitialize.capabilities, positionEncoding: "utf-8" },
  });
  assert.equal(native.knownNative, true);
  assert.equal(native.unknownInitialize, false);
  assert.equal(native.absentObserver, false);
  assert.equal(native.reapedImage, null);
  assert.equal(native.before.pid, native.pid);
  assert.equal(native.terminal.pid, native.pid);
  for (const key of ["completed", "stderr_complete", "reaped"]) {
    assert.equal(native.before[key], false);
    assert.equal(native.terminal[key], true);
  }
  for (const row of [native.before, native.terminal]) {
    assert.deepEqual(row.stderr, []);
    for (const key of ["stderr_error", "process_error"]) assert.equal(row[key], null);
    for (const key of ["timed_out", "kill_requested"]) assert.equal(row[key], false);
    assert.match(row.elapsed_nanos, /^\d+$/u);
  }
  assert.equal(native.before.exit_status, null);
  assert.equal(native.terminal.exit_status, 0);
}

function run() {
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  process.chdir(process.env.NATIVE_PHASE_SOURCE_ROOT);
  const root = path.join(process.env.RUNNER_TEMP, "native-source-recipe-owned-image");
  assert.equal(fs.existsSync(root), false);
  fs.mkdirSync(root);
  const git = (args) => execFileSync("git", ["--no-replace-objects", ...args]);
  assert.equal(git(["for-each-ref", "--format=%(refname)", "refs/replace"]).length, 0);
  assert.equal(git(["status", "--porcelain", "--untracked-files=no"]).length, 0);
  const source = git(["rev-parse", "HEAD"]).toString().trim();
  assert.equal(source, process.env.SOURCE_SHA);
  const tree = git(["rev-parse", "HEAD^{tree}"]).toString().trim();
  const env = {
    ...process.env,
    GIT_NO_REPLACE_OBJECTS: "1",
    VIZE_TEST_REQUIRE_TSGO: "1",
    VIZE_NATIVE_OWNED_IMAGE_CAPTURE: path.join(root, "controls"),
  };
  delete env.VIZE_TEST_DISABLE_TSGO;
  const execute = (name, argv) => {
    fs.writeFileSync(path.join(root, `${name}.argv.nul`), argv.join("\0") + "\0");
    const result = spawnSync(argv[0], argv.slice(1), { env, maxBuffer: 128 * 1024 * 1024 });
    fs.writeFileSync(path.join(root, `${name}.stdout`), result.stdout ?? Buffer.alloc(0));
    fs.writeFileSync(path.join(root, `${name}.stderr`), result.stderr ?? Buffer.alloc(0));
    fs.writeFileSync(
      path.join(root, `${name}.status.json`),
      JSON.stringify({
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
      }),
    );
    if (name !== "cargo") process.stdout.write(result.stdout ?? Buffer.alloc(0));
    process.stderr.write(result.stderr ?? Buffer.alloc(0));
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0);
    return result;
  };
  const cargo = execute("cargo", [
    "cargo",
    "test",
    "--locked",
    "--profile",
    "ci-opt",
    "-p",
    "vize_canon",
    "--lib",
    "--no-run",
    "--message-format=json",
    "--config",
    'profile.ci-opt.inherits="release"',
    "--config",
    'profile.ci-opt.lto="thin"',
    "--config",
    "profile.ci-opt.codegen-units=16",
  ]);
  const artifacts = cargo.stdout
    .toString()
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line))
    .filter(
      (row) =>
        row.reason === "compiler-artifact" &&
        row.target.name === "vize_canon" &&
        row.target.kind.join() === "lib" &&
        row.profile.test &&
        row.executable,
    );
  assert.equal(artifacts.length, 1);
  const artifact = artifacts[0];
  assert.equal(
    fs.realpathSync(artifact.target.src_path),
    path.join(fs.realpathSync(process.cwd()), "crates/vize_canon/src/lib.rs"),
  );
  assert.deepEqual(
    [...artifact.features].sort((a, b) => a.localeCompare(b)),
    ["default", "native"],
  );
  assert.equal(artifact.profile.opt_level, "3");
  assert.equal(artifact.profile.debug_assertions, false);
  const testBinary = fs.realpathSync(artifact.executable);
  const rustc = execute("rustc", ["rustc", "-vV"]);
  assert.ok(rustc.stdout.toString().includes("release: 1.99.0\n"));
  assert.ok(rustc.stdout.toString().includes("host: x86_64-unknown-linux-gnu\n"));
  const cargoVersion = execute("cargo-version", ["cargo", "--version"]);
  assert.match(cargoVersion.stdout.toString(), /^cargo 1\.99\.0 /u);
  const result = execute("laws", [testBinary, prefix, "--nocapture", "--test-threads=1"]);
  assert.equal(result.stderr.length, 0);
  const packets = fs
    .readdirSync(env.VIZE_NATIVE_OWNED_IMAGE_CAPTURE)
    .map((name) => [
      name,
      JSON.parse(fs.readFileSync(path.join(env.VIZE_NATIVE_OWNED_IMAGE_CAPTURE, name), "utf8")),
    ]);
  const nativeBinary = fs.realpathSync(process.env.CORSA_PATH);
  const nativePackage = JSON.parse(
    fs.readFileSync(path.resolve(path.dirname(nativeBinary), "../package.json"), "utf8"),
  );
  const version = execute("version", [nativeBinary, "--version"]);
  assert.equal(version.stderr.length, 0);
  const original = JSON.parse(
    fs.readFileSync(path.join(process.env.VIZE_DEFAULT_PROP_CAPTURE, "receipts.json"), "utf8"),
  );
  assert.equal(original.sourceSha, source);
  assert.equal(original.matchedCompleteExpectations, true);
  assert.equal(original.cases.length, 3);
  for (const row of original.cases) {
    assert.equal(row.sourceSha, source);
    assert.equal(fs.realpathSync(row.nativeBinary), nativeBinary);
    assert.equal(row.nativeSha256, hash(fs.readFileSync(nativeBinary)));
  }
  const nativeSource = fs.readFileSync(
    "crates/vize_canon/src/lsp_client/editor_lsp/native_shutdown.rs",
    "utf8",
  );
  const contract = nativeSource.match(
    /fn known_contract\(\) -> Value \{\s*serde_json::json!\(([\s\S]*?)\)\s*\}\s*fn known_binary/u,
  );
  assert.ok(contract);
  const initialize = JSON.parse(contract[1]);
  const corpus = fs.readFileSync(
    "tests/_fixtures/differential/typechecker/native-owned-exec/exec-wrapper.sh.txt",
    "utf8",
  );
  const runtime = {
    requireNative: env.VIZE_TEST_REQUIRE_TSGO,
    disableNative: env.VIZE_TEST_DISABLE_TSGO ?? null,
    nativeSHA256: hash(fs.readFileSync(nativeBinary)),
    platform: process.platform,
    arch: process.arch,
    nativeVersion: version.stdout.toString().trim(),
    package: nativePackage,
    nativeBinary,
    wholeInitialize: initialize,
    wholeWrapper: corpus.replace("__BACKEND__", nativeBinary.replaceAll("'", "'\\''")),
  };
  validatePackets(packets, result.stdout.toString(), runtime);
  const bodies = [
    "crates/vize_canon/src/lsp_client/editor_lsp.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/native_shutdown.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/native_shutdown/owned_image.rs",
    "tests/_fixtures/differential/typechecker/native-owned-exec/exec-wrapper.sh.txt",
    "tools/support/compat/github/native-owned-image.mjs",
    "Cargo.toml",
    "Cargo.lock",
    "crates/vize_canon/Cargo.toml",
    "rust-toolchain.toml",
  ];
  const sourceBodies = bodies.map((file) => {
    const bytes = fs.readFileSync(file);
    assert.deepEqual(bytes, git(["show", `${source}:${file}`]));
    fs.mkdirSync(path.dirname(path.join(root, "source", file)), { recursive: true });
    fs.writeFileSync(path.join(root, "source", file), bytes);
    return {
      file,
      byteLength: bytes.length,
      sha256: hash(bytes),
      modeAndBlob: git(["ls-tree", source, "--", file]).toString().trim(),
    };
  });
  fs.writeFileSync(
    path.join(root, "receipt.json"),
    JSON.stringify(
      {
        source,
        tree,
        sourceBodies,
        runtime,
        cargoArtifact: artifact,
        toolVersions: { rustc: rustc.stdout.toString(), cargo: cargoVersion.stdout.toString() },
        testBinary: { path: testBinary, sha256: hash(fs.readFileSync(testBinary)) },
        originalReceipts: original,
        actualLaws: names,
        wholePackets: packets,
        matchedCompleteExpectations: true,
      },
      null,
      2,
    ),
  );
}

if (process.argv[1] && fileURLToPath(import.meta.url) === fs.realpathSync(process.argv[1])) run();
