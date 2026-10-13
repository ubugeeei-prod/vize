import assert from "node:assert/strict";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";

import {
  applePlatformTag,
  binaryName,
  builtAppleLibraryPath,
  nativeBuildEnvironment,
  napiFeatures,
} from "../../tools/support/github/build-napi-package.ts";
import {
  pathWith,
  readRecordedCalls,
  runToolCommand,
  writeRecordingCommand,
} from "./_helpers/tool-commands.ts";

const script = "tools/support/github/build-napi-package.ts";

function workspace(t: TestContext) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(tmpdir(), "github-build-napi-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const packageDir = path.join(root, "npm", "native");
  fs.mkdirSync(packageDir, { recursive: true });
  const bin = path.join(root, "bin");
  const log = path.join(root, "calls.log");
  const run = (args: string[]) =>
    runToolCommand(script, args, {
      cwd: root,
      env: { PATH: pathWith(bin), VIZE_NATIVE_BUILD_TOKEN: "inherited-token" },
    });
  return { root, packageDir, bin, log, run };
}

void test("the package naming tables match the release platform files", () => {
  assert.equal(binaryName("vize_vitrine"), "vize-vitrine");
  assert.equal(binaryName("vize_fresco_native"), "vize-fresco-native");
  assert.equal(napiFeatures("vize_vitrine"), "napi,legacy");
  assert.equal(napiFeatures("vize_fresco"), "napi");
  assert.equal(applePlatformTag("aarch64-apple-darwin"), "darwin-arm64");
  assert.equal(applePlatformTag("x86_64-apple-darwin"), "darwin-x64");
  assert.equal(applePlatformTag("arm64e-apple-darwin"), undefined);
  assert.equal(
    builtAppleLibraryPath("npm/native", "aarch64-apple-darwin", "vize_vitrine"),
    path.join("target", "aarch64-apple-darwin", "release", "libvize_vitrine.dylib"),
  );
});

void test("missing platform entropy refuses to start a native build", (t) => {
  const write = t.mock.method(process.stdout, "write", () => true);
  assert.equal(
    nativeBuildEnvironment({ KEEP: "1" }, () => undefined),
    undefined,
  );
  assert.deepEqual(
    write.mock.calls.map((call) => call.arguments[0]),
    ["Could not allocate native build identity\n"],
  );
});

void test("argument errors are reported before any build starts", (t) => {
  const { bin, log, run } = workspace(t);
  writeRecordingCommand(bin, "vp", log);
  writeRecordingCommand(bin, "cargo", log);
  for (const args of [[], ["a", "b", "c"], ["a", "b", "c", "d", "--cross-compile", "x"]]) {
    const result = run(args);
    assert.equal(result.status, 1);
    assert.match(
      result.stdout,
      /^Usage: .*build-napi-package\.ts <package-dir> <manifest-path> <package-name> <target> \[--cross-compile\]\n$/u,
    );
  }
  const unknown = run(["p", "m", "vize_fresco", "x86_64-unknown-linux-gnu", "--release"]);
  assert.deepEqual([unknown.status, unknown.stdout], [1, "Unknown argument\n"]);
  const apple = run(["p", "m", "vize_fresco", "aarch64-apple-darwin", "--cross-compile"]);
  assert.deepEqual(
    [apple.status, apple.stdout],
    [1, "Apple targets do not support --cross-compile in this helper.\n"],
  );
  assert.deepEqual(readRecordedCalls(log), []);
});

void test("non-Apple targets run napi through vp with a fresh build token", (t) => {
  const { root, packageDir, bin, log, run } = workspace(t);
  writeRecordingCommand(bin, "vp", log, { exitCodes: [0, 42] });
  const plain = run([
    "npm/native",
    "../../crates/vize_vitrine/Cargo.toml",
    "vize_vitrine",
    "x86_64-unknown-linux-gnu",
  ]);
  const cross = run([
    "npm/native",
    "../../crates/vize_fresco/Cargo.toml",
    "vize_fresco",
    "aarch64-unknown-linux-gnu",
    "--cross-compile",
  ]);
  assert.equal(plain.status, 0, plain.stdout + plain.stderr);
  assert.equal(cross.status, 42);
  const calls = readRecordedCalls(log);
  const napi = (manifest: string, crate: string, features: string, target: string) => [
    "exec",
    "napi",
    "build",
    "--platform",
    "--release",
    "--manifest-path",
    manifest,
    "-p",
    crate,
    "--features",
    features,
    "--output-dir",
    ".",
    "--target",
    target,
  ];
  assert.deepEqual(
    calls.map(({ argv, cwd }) => ({ argv, cwd })),
    [
      {
        argv: napi(
          "../../crates/vize_vitrine/Cargo.toml",
          "vize_vitrine",
          "napi,legacy",
          "x86_64-unknown-linux-gnu",
        ),
        cwd: packageDir,
      },
      {
        argv: [
          ...napi(
            "../../crates/vize_fresco/Cargo.toml",
            "vize_fresco",
            "napi",
            "aarch64-unknown-linux-gnu",
          ),
          "--cross-compile",
        ],
        cwd: packageDir,
      },
    ],
  );
  const tokens = calls.map(({ env }) => env.VIZE_NATIVE_BUILD_TOKEN);
  for (const token of tokens) assert.match(token ?? "", /^[0-9a-f]{32}$/u);
  assert.notEqual(tokens[0], tokens[1]);
  assert.equal(calls[0]!.env.PATH, pathWith(bin));
  assert.equal(fs.existsSync(path.join(root, "target")), false);
});

void test("Apple targets report cargo failures, unknown platforms and missing libraries", (t) => {
  const { root, packageDir, bin, log, run } = workspace(t);
  writeRecordingCommand(bin, "cargo", log, { exitCodes: [23, 0] });
  const failed = run(["npm/native", "m.toml", "vize_vitrine", "aarch64-apple-darwin"]);
  assert.deepEqual([failed.status, failed.stdout], [23, ""]);
  const unsupported = run(["npm/native", "m.toml", "vize_vitrine", "arm64e-apple-darwin"]);
  assert.deepEqual(
    [unsupported.status, unsupported.stdout],
    [1, "Unsupported Apple target: arm64e-apple-darwin\n"],
  );
  const missing = run(["./npm/native/", "m.toml", "vize_fresco", "x86_64-apple-darwin"]);
  assert.deepEqual(
    [missing.status, missing.stdout],
    [
      1,
      `Built library not found: ${path.join("target", "x86_64-apple-darwin", "release", "libvize_fresco.dylib")}\n`,
    ],
  );
  fs.mkdirSync(
    path.join(root, "target", "x86_64-apple-darwin", "release", "libvize_fresco.dylib"),
    { recursive: true },
  );
  const directory = run([packageDir, "m.toml", "vize_fresco", "x86_64-apple-darwin"]);
  assert.deepEqual(
    [directory.status, directory.stdout],
    [
      1,
      `Built library not found: ${path.join(root, "target", "x86_64-apple-darwin", "release", "libvize_fresco.dylib")}\n`,
    ],
  );
  const calls = readRecordedCalls(log);
  assert.equal(calls.length, 4);
  assert.deepEqual(calls[2]!.argv, [
    "build",
    "--release",
    "--manifest-path",
    "m.toml",
    "-p",
    "vize_fresco",
    "--features",
    "napi",
    "--target",
    "x86_64-apple-darwin",
  ]);
  for (const call of calls) assert.match(call.env.VIZE_NATIVE_BUILD_TOKEN ?? "", /^[0-9a-f]{32}$/u);
});

void test("Apple targets replace an existing platform binary in place", (t) => {
  const { root, packageDir, bin, log, run } = workspace(t);
  const library = path.join(
    root,
    "target",
    "x86_64-apple-darwin",
    "release",
    "libvize_vitrine.dylib",
  );
  writeRecordingCommand(bin, "cargo", log, {
    script: `fs.mkdirSync(${JSON.stringify(path.dirname(library))}, { recursive: true }); fs.writeFileSync(${JSON.stringify(library)}, 'fresh');`,
  });
  const output = path.join(packageDir, "vize-vitrine.darwin-x64.node");
  fs.writeFileSync(output, "stale-binary-with-more-bytes");
  fs.chmodSync(output, 0o600);
  const result = run(["npm/native", "m.toml", "vize_vitrine", "x86_64-apple-darwin"]);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  assert.equal(fs.readFileSync(output, "utf8"), "fresh");
  if (process.platform !== "win32") assert.equal(fs.statSync(output).mode & 0o777, 0o600);
});

void test("github/build-vitrine-wasm builds once and binds every output directory", (t) => {
  const { root, bin, log } = workspace(t);
  writeRecordingCommand(bin, "cargo", log);
  writeRecordingCommand(bin, "wasm-bindgen", log, { exitCodes: [0, 8] });
  const run = (args: string[], env: Record<string, string | undefined>) =>
    runToolCommand("tools/support/github/build-vitrine-wasm.ts", args, {
      cwd: root,
      env: { PATH: pathWith(bin), ...env },
    });
  const usage = run([], {});
  assert.equal(usage.status, 1);
  assert.match(usage.stdout, /^Usage: .*build-vitrine-wasm\.ts <out-dir> \[out-dir\.\.\.\]\n$/u);
  assert.equal(
    run(["npm/wasm", "playground/src/wasm", "never"], { RUSTFLAGS: undefined }).status,
    8,
  );
  assert.equal(run(["out"], { RUSTFLAGS: "--cfg inherited" }).status, 8);
  const calls = readRecordedCalls(log);
  const bindgen = (outDir: string) => [
    "target/wasm32-unknown-unknown/release/vize_vitrine.wasm",
    "--out-dir",
    outDir,
    "--target",
    "web",
  ];
  assert.deepEqual(
    calls.map(({ name, argv }) => [name, ...argv]),
    [
      [
        "cargo",
        "build",
        "--release",
        "-p",
        "vize_vitrine",
        "--no-default-features",
        "--features",
        "wasm",
        "--target",
        "wasm32-unknown-unknown",
      ],
      ["wasm-bindgen", ...bindgen("npm/wasm")],
      ["wasm-bindgen", ...bindgen("playground/src/wasm")],
      [
        "cargo",
        "build",
        "--release",
        "-p",
        "vize_vitrine",
        "--no-default-features",
        "--features",
        "wasm",
        "--target",
        "wasm32-unknown-unknown",
      ],
      ["wasm-bindgen", ...bindgen("out")],
    ],
  );
  assert.equal(calls[0]!.env.RUSTFLAGS, '--cfg getrandom_backend="wasm_js"');
  // An inherited value keeps precedence over the appended flag, as before.
  assert.equal(calls[3]!.env.RUSTFLAGS, "--cfg inherited");
  for (const call of calls) assert.equal(call.cwd, root);
});
