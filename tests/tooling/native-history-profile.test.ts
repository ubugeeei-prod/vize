import assert from "node:assert/strict";
import childProcess, { type ExecFileSyncOptions } from "node:child_process";
import fs from "node:fs";
import { syncBuiltinESMExports } from "node:module";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import {
  bytes,
  captureNativeHistoryBuild,
  emittedNativeArtifact,
  nativeHistoryBuildEnvironment,
  nativeHistoryCargoOptions,
  nativeHistorySource,
  validateNativeHistoryBuild,
} from "../../npm/native/scripts/formatter-history-build.mts";

// Pure synthetic admission controls: Git/source/filesystem custody is real;
// addon bytes and toolchain replies are authored fixtures, never executed.
const profile = {
  test: false,
  opt_level: "0",
  debuginfo: "line-tables-only",
  debug_assertions: true,
  overflow_checks: true,
};
const wrongProfiles = [
  { ...profile, test: true },
  { ...profile, opt_level: "1" },
  { ...profile, opt_level: 0 },
  { ...profile, debuginfo: 2 },
  { ...profile, debuginfo: 1 },
  { ...profile, debuginfo: null },
  { ...profile, debuginfo: "line-directives-only" },
  { ...profile, debug_assertions: false },
  { ...profile, overflow_checks: false },
];

const profileKeys = [
  "test",
  "opt_level",
  "debuginfo",
  "debug_assertions",
  "overflow_checks",
] as const;
function profileRefusal(action: () => unknown, wrong: (typeof wrongProfiles)[number]) {
  const key = profileKeys.find((field) => wrong[field] !== profile[field]);
  assert(key);
  assert.throws(action, {
    code: "ERR_ASSERTION",
    actual: wrong[key],
    expected: profile[key],
    operator: "strictEqual",
  });
}

function fixture(context: TestContext) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "native-profile-admission-"));
  context.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const nativeDir = path.join(root, "npm/native");
  function write(name: string, value: string | Buffer) {
    const file = path.join(root, name);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, value);
  }
  write(".gitignore", "node_modules/\n.artifacts/\n");
  write("Cargo.lock", "# synthetic source identity\n");
  write("pnpm-lock.yaml", "# synthetic source identity\n");
  write("npm/native/package.json", '{"name":"synthetic-native-fixture"}\n');
  write("crates/vize_vitrine/src/lib.rs", "// synthetic source identity; never compiled\n");
  write("crates/vize_glyph/src/lib.rs", "// synthetic source identity; never compiled\n");
  write(
    "npm/native/node_modules/@napi-rs/cli/package.json",
    '{"name":"@napi-rs/cli","version":"0.0.0-fixture","main":"dist/cli.js"}\n',
  );
  write("npm/native/node_modules/@napi-rs/cli/dist/cli.js", "// identity only; never executed\n");
  const actualExec = childProcess.execFileSync;
  const git = (...args: string[]) => actualExec("git", args, { cwd: root });
  git("init", "--quiet");
  git("add", ".");
  git(
    "-c",
    "user.name=Fixture",
    "-c",
    "user.email=fixture@example.invalid",
    "commit",
    "--quiet",
    "-m",
    "fixture",
  );
  context.mock.method(
    childProcess,
    "execFileSync",
    (file: string, args?: readonly string[], options?: ExecFileSyncOptions) => {
      if (file === "rustc" || file === (process.env.CARGO ?? "cargo")) {
        assert.deepEqual(args, file === "rustc" ? ["-Vv"] : ["-V"]);
        return Buffer.from(`${file === "rustc" ? "rustc" : "cargo"} authored fixture identity\n`);
      }
      assert.equal(file, "git", "the pure control must never launch a build or native provider");
      return actualExec(file, args ?? [], options);
    },
  );
  syncBuiltinESMExports();
  context.after(() => {
    context.mock.restoreAll();
    syncBuiltinESMExports();
  });
  const output = path.join(nativeDir, ".artifacts/native");
  const image = Buffer.from("synthetic addon image; never dlopen\0\xff", "utf8");
  write("npm/native/.artifacts/native/emitted.so", image);
  write("npm/native/.artifacts/native/vize-vitrine.synthetic.node", image);
  const command = [
    "pnpm",
    "exec",
    "napi",
    "build",
    "--platform",
    "--manifest-path",
    "../../crates/vize_vitrine/Cargo.toml",
    "-p",
    "vize_vitrine",
    "--features",
    "napi,legacy",
    "--output-dir",
    output,
    "--no-js",
    "--",
    "--message-format=json-render-diagnostics",
    "--locked",
    "--config",
    'profile.dev.debug="line-tables-only"',
  ];
  const before = {
    source: nativeHistorySource(nativeDir),
    environment: nativeHistoryBuildEnvironment(),
  };
  const stdout = (actualProfile: unknown) =>
    Buffer.from(
      [
        JSON.stringify({
          reason: "compiler-artifact",
          target: {
            name: "vize_vitrine",
            kind: ["cdylib", "rlib"],
            src_path: path.join(root, "crates/vize_vitrine/src/lib.rs"),
          },
          profile: actualProfile,
          features: ["napi", "legacy"],
          filenames: [path.join(output, "emitted.so")],
        }),
        JSON.stringify({ reason: "build-finished", success: true }),
        "",
      ].join("\n"),
    );
  const result = (actualProfile: unknown) => ({
    pid: 1,
    output: [],
    stdout: stdout(actualProfile),
    stderr: Buffer.from("whole authored stderr\0\n"),
    status: 0,
    signal: null,
  });
  const capture = (actualProfile: unknown = profile, invocation = command) =>
    captureNativeHistoryBuild(nativeDir, before, invocation, result(actualProfile));
  return { nativeDir, before, command, stdout, result, capture };
}

void test("captured dev admission preserves one complete source, command, profile and image receipt", (context) => {
  const f = fixture(context);
  const receipt = f.capture();
  assert.deepEqual(receipt.source, f.before.source);
  assert.deepEqual(receipt.process.command, f.command);
  assert.deepEqual(receipt.emitted.cargo.profile, profile);
  assert.deepEqual(validateNativeHistoryBuild(f.nativeDir, receipt), receipt);
});

void test("wrong captured profile is refused after whole raw process custody without emitting a receipt", (context) => {
  const f = fixture(context);
  for (const wrong of wrongProfiles) {
    profileRefusal(() => f.capture(wrong), wrong);
    const directory = path.join(f.nativeDir, ".artifacts/native/formatter-history");
    const raw = fs.readFileSync(path.join(directory, "cargo.stdout.bin"));
    assert.deepEqual(raw, f.stdout(wrong));
    const processPacket = JSON.parse(
      fs.readFileSync(path.join(directory, "build-process.json"), "utf8"),
    );
    assert.deepEqual(processPacket.command, f.command);
    assert.deepEqual(processPacket.stdout, bytes(raw));
    assert.equal(processPacket.exitStatus, 0);
    assert.equal(fs.existsSync(path.join(directory, "build-receipt.json")), false);
    assert.equal(fs.existsSync(path.join(directory, "formatter-observe.node")), false);
  }
});

void test("revalidation rejects self-consistent wrong profiles after genuine source and image custody", (context) => {
  const f = fixture(context);
  const receipt = f.capture();
  assert.deepEqual(validateNativeHistoryBuild(f.nativeDir, receipt), receipt);
  for (const wrong of wrongProfiles) {
    const raw = f.stdout(wrong);
    const mutated = {
      ...receipt,
      process: { ...receipt.process, stdout: bytes(raw) },
      emitted: { ...receipt.emitted, cargo: { ...receipt.emitted.cargo, profile: wrong } },
    };
    fs.writeFileSync(
      path.join(f.nativeDir, ".artifacts/native/formatter-history/cargo.stdout.bin"),
      raw,
    );
    // These refreshed consistency fields and unchanged image/source passed the positive control.
    assert.deepEqual(mutated.source, nativeHistorySource(f.nativeDir));
    assert.deepEqual(mutated.emitted.cargo.profile, wrong);
    profileRefusal(() => validateNativeHistoryBuild(f.nativeDir, mutated), wrong);
  }
});

void test("capture and revalidation refuse changed invocation with otherwise complete valid custody", (context) => {
  const f = fixture(context);
  const wrong = f.command.slice(0, -2);
  assert.throws(() => f.capture(profile, wrong), {
    code: "ERR_ASSERTION",
    actual: wrong,
    expected: f.command,
    operator: "deepStrictEqual",
  });
  const processFile = path.join(
    f.nativeDir,
    ".artifacts/native/formatter-history/build-process.json",
  );
  assert.deepEqual(JSON.parse(fs.readFileSync(processFile, "utf8")).command, wrong);
  const receipt = f.capture();
  assert.deepEqual(validateNativeHistoryBuild(f.nativeDir, receipt), receipt);
  const mutated = { ...receipt, process: { ...receipt.process, command: wrong } };
  assert.throws(() => validateNativeHistoryBuild(f.nativeDir, mutated), {
    code: "ERR_ASSERTION",
    actual: wrong,
    expected: f.command,
    operator: "deepStrictEqual",
  });
});

void test("typed Cargo admission retains complete objects and never drops a malformed matching candidate", () => {
  const artifact = {
    reason: "compiler-artifact",
    target: { name: "vize_vitrine", kind: ["cdylib"], src_path: "/authored/source.rs" },
    profile,
    features: ["napi", "legacy"],
    filenames: ["/authored/image.so"],
    authored_unknown_field: { unicode: "日本語", bytes: [0, 255] },
  };
  const finished = { reason: "build-finished", success: true };
  const stream = (rows: unknown[]) =>
    Buffer.from(rows.map((row) => JSON.stringify(row)).join("\n") + "\n");
  assert.deepEqual(emittedNativeArtifact(stream([artifact, finished])).cargo, artifact);
  for (const malformed of [
    null,
    { reason: "compiler-artifact", target: { name: "vize_vitrine", kind: null } },
    { reason: "compiler-artifact", target: { name: "vize_vitrine" } },
  ]) {
    assert.throws(() => emittedNativeArtifact(stream([artifact, malformed, finished])));
  }
});

void test("the shared authored Cargo command policy cannot be mutated by an importing caller", () => {
  const expected = [
    "--message-format=json-render-diagnostics",
    "--locked",
    "--config",
    'profile.dev.debug="line-tables-only"',
  ];
  assert.deepEqual(nativeHistoryCargoOptions, expected);
  assert.equal(Object.isFrozen(nativeHistoryCargoOptions), true);
  assert.equal(Reflect.set(nativeHistoryCargoOptions, 1, "--offline"), false);
  assert.deepEqual(nativeHistoryCargoOptions, expected);
});
