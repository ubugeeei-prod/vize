import assert from "node:assert/strict";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";

import {
  cargoEnvironment,
  macosLibraryPath,
  resolveMacosSdkRoot,
  wasmBindgenBinary,
  wasmBindgenVersion,
  wasmPackCache,
  wasmRustflags,
} from "../../tools/support/tasks/build-vitrine-wasm.ts";
import {
  pathWith,
  readRecordedCalls,
  runToolCommand,
  writeRecordingCommand,
} from "./_helpers/tool-commands.ts";

const vitrine = "tools/support/tasks/build-vitrine-wasm.ts";
const wasmPackage = "tools/support/tasks/build-vize-wasm-package.ts";
const bindgenRoot = `.cache/wasm-pack/.wasm-bindgen-cargo-install-${wasmBindgenVersion}`;

function workspace(t: TestContext) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(tmpdir(), "vize-task-commands-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const bin = path.join(root, "bin");
  const log = path.join(root, "calls.log");
  return { root, bin, log, env: { PATH: pathWith(bin), SDKROOT: "sdk", RUSTFLAGS: undefined } };
}

void test("WASM flags, caches and macOS SDK paths follow the inherited environment", () => {
  assert.equal(wasmRustflags({}), '--cfg getrandom_backend="wasm_js"');
  assert.equal(wasmRustflags({ RUSTFLAGS: " \t\n\r" }), '--cfg getrandom_backend="wasm_js"');
  assert.equal(wasmRustflags({ RUSTFLAGS: "-Cx" }), '-Cx --cfg getrandom_backend="wasm_js"');
  assert.equal(wasmPackCache("/repo", {}), path.join("/repo", ".cache/wasm-pack"));
  assert.equal(wasmPackCache("/repo", { WASM_PACK_CACHE: "" }), process.cwd());
  assert.equal(
    wasmBindgenBinary("/repo", { WASM_PACK_CACHE: "/cache/" }),
    path.join("/cache", `.wasm-bindgen-cargo-install-${wasmBindgenVersion}`, "bin", "wasm-bindgen"),
  );
  assert.equal(macosLibraryPath("/sdk", {}), path.join("/sdk", "usr/lib"));
  assert.equal(macosLibraryPath("/sdk", { LIBRARY_PATH: "" }), path.join("/sdk", "usr/lib"));
  assert.equal(
    macosLibraryPath("/sdk", { LIBRARY_PATH: "/a:/b" }),
    `${path.join("/sdk", "usr/lib")}:/a:/b`,
  );
  assert.equal(resolveMacosSdkRoot("/repo", { SDKROOT: "/sdk/" }), path.resolve("/sdk"));
  assert.deepEqual(cargoEnvironment("/repo", ["install"], { SDKROOT: "/sdk", KEEP: "1" }), {
    SDKROOT: path.resolve("/sdk"),
    KEEP: "1",
    LIBRARY_PATH: path.join(path.resolve("/sdk"), "usr/lib"),
    CC: "clang",
    CXX: "clang++",
  });
});

void test("the macOS SDK falls back from xcrun to the standard install locations", (t) => {
  const { root, bin, log } = workspace(t);
  const candidate = path.join(root, "MacOSX.sdk");
  fs.mkdirSync(candidate);
  writeRecordingCommand(bin, "xcrun", log, {
    script: "if (process.argv.includes('--sdk')) process.exit(1);",
    stdout: "  /from/xcrun \n",
  });
  const originalPath = process.env.PATH;
  try {
    process.env.PATH = pathWith(bin);
    assert.equal(
      resolveMacosSdkRoot(root, {}, [path.join(root, "absent"), candidate]),
      "/from/xcrun",
    );
    assert.deepEqual(
      readRecordedCalls(log).map(({ argv, cwd }) => ({ argv, cwd })),
      [
        { argv: ["--sdk", "macosx", "--show-sdk-path"], cwd: root },
        { argv: ["--show-sdk-path"], cwd: root },
      ],
    );
    process.env.PATH = path.join(root, "no-commands");
    assert.equal(
      resolveMacosSdkRoot(root, { SDKROOT: " " }, [path.join(root, "absent"), candidate]),
      candidate,
    );
    assert.equal(resolveMacosSdkRoot(root, {}, [path.join(root, "absent")]), undefined);
  } finally {
    process.env.PATH = originalPath;
  }
});

void test("build-vitrine-wasm installs the pinned bindgen once and binds into the output", (t) => {
  const { root, bin, log, env } = workspace(t);
  writeRecordingCommand(bin, "cargo", log, {
    script: `if (process.argv[2] === 'install') { const p = require('node:path').join(process.argv[process.argv.length - 1], 'bin'); fs.mkdirSync(p, { recursive: true }); fs.copyFileSync(${JSON.stringify(path.join(bin, "wasm-bindgen"))}, require('node:path').join(p, 'wasm-bindgen')); fs.copyFileSync(${JSON.stringify(path.join(bin, ".wasm-bindgen.cjs"))}, require('node:path').join(p, '.wasm-bindgen.cjs')); }`,
  });
  writeRecordingCommand(bin, "wasm-bindgen", log, { exitCodes: [5] });

  const result = runToolCommand(vitrine, ["web", "playground/src/wasm"], { cwd: root, env });

  assert.equal(result.status, 5, result.stdout + result.stderr);
  assert.ok(fs.statSync(path.join(root, "playground/src/wasm")).isDirectory());
  const calls = readRecordedCalls(log);
  assert.deepEqual(
    calls.map(({ name, argv, cwd }) => [name, cwd, ...argv]),
    [
      [
        "cargo",
        root,
        "install",
        "--force",
        "wasm-bindgen-cli",
        "--version",
        wasmBindgenVersion,
        "--root",
        path.join(root, bindgenRoot),
      ],
      [
        "cargo",
        root,
        "build",
        "--manifest-path",
        path.join(root, "Cargo.toml"),
        "--release",
        "-p",
        "vize_vitrine",
        "--no-default-features",
        "--features",
        "wasm",
        "--target",
        "wasm32-unknown-unknown",
      ],
      [
        "wasm-bindgen",
        root,
        path.join(root, "target/wasm32-unknown-unknown/release/vize_vitrine.wasm"),
        "--out-dir",
        path.join(root, "playground/src/wasm"),
        "--target",
        "web",
      ],
    ],
  );
  const [install, build] = calls;
  assert.equal(install!.env.RUSTFLAGS, undefined);
  assert.equal(install!.env.CARGO_PROFILE_RELEASE_PANIC, undefined);
  assert.equal(build!.env.RUSTFLAGS, '--cfg getrandom_backend="wasm_js"');
  assert.equal(build!.env.CARGO_PROFILE_RELEASE_PANIC, "abort");
  for (const call of [install!, build!]) {
    assert.deepEqual(
      [call.env.SDKROOT, call.env.CC, call.env.CXX, call.env.LIBRARY_PATH],
      [path.join(root, "sdk"), "clang", "clang++", path.join(root, "sdk", "usr/lib")],
    );
  }
  assert.equal(calls[2]!.env.SDKROOT, "sdk");

  fs.rmSync(log);
  assert.equal(runToolCommand(vitrine, ["nodejs", "out"], { cwd: root, env }).status, 5);
  assert.deepEqual(
    readRecordedCalls(log).map(({ name, argv }) => [name, argv[0]]),
    [
      ["cargo", "build"],
      ["wasm-bindgen", path.join(root, "target/wasm32-unknown-unknown/release/vize_vitrine.wasm")],
    ],
  );
});

void test("build-vitrine-wasm stops at usage, install and build failures", (t) => {
  const { root, bin, log, env } = workspace(t);
  writeRecordingCommand(bin, "cargo", log, { exitCodes: [7, 0, 0, 17] });
  for (const args of [[], ["web"], ["web", "out", "extra"]]) {
    const usage = runToolCommand(vitrine, args, { cwd: root, env });
    assert.equal(usage.status, 1);
    assert.match(usage.stdout, /^Usage: .*build-vitrine-wasm\.ts <target> <out-dir>\n$/u);
  }
  assert.equal(runToolCommand(vitrine, ["web", "out"], { cwd: root, env }).status, 7);
  const missingBindgen = runToolCommand(vitrine, ["web", "out"], {
    cwd: root,
    env: { ...env, WASM_PACK_CACHE: "cache" },
  });
  assert.equal(missingBindgen.status, 1);
  assert.match(missingBindgen.stderr, /ENOENT/u);
  fs.mkdirSync(path.join(root, bindgenRoot, "bin"), { recursive: true });
  writeRecordingCommand(path.join(root, bindgenRoot, "bin"), "wasm-bindgen", log);
  assert.equal(runToolCommand(vitrine, ["web", "out"], { cwd: root, env }).status, 17);
  assert.deepEqual(
    readRecordedCalls(log).map(({ name, argv }) => [name, argv[0]]),
    [
      ["cargo", "install"],
      ["cargo", "install"],
      ["cargo", "build"],
      ["cargo", "build"],
    ],
  );
  assert.equal(
    readRecordedCalls(log)[1]!.argv.at(-1),
    path.join(root, "cache", `.wasm-bindgen-cargo-install-${wasmBindgenVersion}`),
  );
});

void test("build-vize-wasm-package reports each failed stage and restores package metadata", (t) => {
  const { root, bin, log, env } = workspace(t);
  const manifest = '{\n  "name": "@vizejs/wasm",\n  "description": "日本語"\n}\n';
  writeRecordingCommand(bin, "cargo", log, { exitCodes: [3, 0] });
  writeRecordingCommand(bin, "wasm-bindgen", log, { exitCodes: [2, 0] });
  writeRecordingCommand(bin, "git", log, { exitCodes: [128, 0], stdout: manifest });
  fs.mkdirSync(path.join(root, "npm/wasm"), { recursive: true });
  fs.writeFileSync(path.join(root, "npm/wasm/package.json"), "generated");
  const run = () =>
    runToolCommand(wasmPackage, [], { cwd: root, env: { ...env, RUSTFLAGS: "-Cx" } });

  for (const message of [
    "Failed to build the WASM crate\n",
    "Failed to run wasm-bindgen\n",
    "Failed to restore npm/wasm/package.json from HEAD\n",
  ]) {
    assert.deepEqual(
      [run().stdout, fs.readFileSync(path.join(root, "npm/wasm/package.json"), "utf8")],
      [message, "generated"],
    );
  }
  const restored = run();
  assert.deepEqual([restored.status, restored.stdout], [0, ""]);
  assert.equal(fs.readFileSync(path.join(root, "npm/wasm/package.json"), "utf8"), manifest);
  const calls = readRecordedCalls(log);
  assert.deepEqual(
    calls.slice(-3).map(({ name, argv }) => [name, ...argv]),
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
      [
        "wasm-bindgen",
        "target/wasm32-unknown-unknown/release/vize_vitrine.wasm",
        "--out-dir",
        "npm/wasm",
        "--target",
        "web",
      ],
      ["git", "show", "HEAD:npm/wasm/package.json"],
    ],
  );
  assert.equal(calls.at(-3)!.env.RUSTFLAGS, '-Cx --cfg getrandom_backend="wasm_js"');
  assert.equal(calls.at(-2)!.env.RUSTFLAGS, "-Cx");
});

void test("build-vize-wasm-package --check type-checks the release configuration only", (t) => {
  const { root, bin, log, env } = workspace(t);
  writeRecordingCommand(bin, "cargo", log, { exitCodes: [4, 0] });
  writeRecordingCommand(bin, "wasm-bindgen", log, { exitCodes: [0] });
  writeRecordingCommand(bin, "git", log, { exitCodes: [0] });
  const run = (args: string[]) =>
    runToolCommand(wasmPackage, args, { cwd: root, env: { ...env, RUSTFLAGS: "-Cx" } });
  assert.equal(run(["--check"]).status, 4);
  assert.equal(run(["--check"]).status, 0);
  assert.equal(run(["--bogus"]).status, 1);
  const calls = readRecordedCalls(log);
  assert.deepEqual(
    calls.map(({ name, argv }) => [name, ...argv]),
    Array.from({ length: 2 }, () => [
      "cargo",
      "check",
      "--release",
      "-p",
      "vize_vitrine",
      "--no-default-features",
      "--features",
      "wasm",
      "--target",
      "wasm32-unknown-unknown",
    ]),
  );
  assert.equal(calls[0]!.env.RUSTFLAGS, '-Cx --cfg getrandom_backend="wasm_js"');
});

void test("dev-app and generate-expected forward to their Node support scripts", (t) => {
  const { root, bin, log, env } = workspace(t);
  writeRecordingCommand(bin, "node", log, { exitCodes: [0, 5, 0, 3] });
  const run = (script: string, args: string[], extra: Record<string, string | undefined> = {}) =>
    runToolCommand(`tools/support/tasks/${script}.ts`, args, {
      cwd: root,
      env: { ...env, usage_target: undefined, ...extra },
    });

  assert.equal(run("dev-app", []).status, 0);
  assert.equal(run("dev-app", ["misskey"]).status, 5);
  assert.equal(run("dev-app", ["elk"], { usage_target: "npmx" }).status, 0);
  const usage = run("dev-app", ["a", "b"]);
  assert.equal(usage.status, 1);
  assert.match(usage.stdout, /^Usage: .*dev-app\.ts \[target\]\n$/u);
  assert.equal(run("generate-expected", ["--mode", "vapor", "sfc/basic"]).status, 3);
  const calls = readRecordedCalls(log);
  assert.deepEqual(
    calls.map(({ argv, cwd, env: childEnv }) => [cwd, childEnv.usage_target, ...argv]),
    [
      [root, undefined, "tests/tooling/support/dev-app.ts"],
      [root, "misskey", "tests/tooling/support/dev-app.ts"],
      // An inherited target keeps precedence, as it did for the MoonBit wrapper.
      [root, "npmx", "tests/tooling/support/dev-app.ts"],
      [
        root,
        undefined,
        "tests/tooling/support/generate-expected.ts",
        "--mode",
        "vapor",
        "sfc/basic",
      ],
    ],
  );
});
