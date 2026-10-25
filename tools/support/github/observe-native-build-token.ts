/** Exact release-recipe witness; this script never publishes or restores a compiled cache. */
import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const json = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
const sourceFiles = [
  "Cargo.lock",
  "Cargo.toml",
  "pnpm-lock.yaml",
  "npm/native/package.json",
  "crates/vize_vitrine/build.rs",
  "crates/vize_vitrine/src/napi/build_token.rs",
  "tools/support/github/build-napi-package.ts",
  "tools/support/github/native-build-receipt.ts",
];
function git(root: string, ...args: string[]) {
  return execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
}
async function producer(directory: string, role: string, expectedSource: string, output: string) {
  assert.equal(process.platform, "darwin");
  assert.equal(process.arch, "arm64", "match the actual release producer host");
  assert.ok(role === "before" || role === "after");
  assert.match(expectedSource, /^[a-f0-9]{40}$/u);
  const root = fs.realpathSync(directory);
  assert.equal(git(root, "rev-parse", "HEAD"), expectedSource);
  assert.equal(git(root, "diff", "--binary", "HEAD"), "", "the producer source must be clean");
  assert.ok(!process.env.CARGO_TARGET_DIR, "the producer must own its new default target");
  assert.equal(fs.existsSync(path.join(root, "target")), false, "no compiled cache restoration");
  fs.mkdirSync(output, { recursive: true });
  const version = JSON.parse(
    fs.readFileSync(path.join(root, "npm/native/package.json"), "utf8"),
  ).version;
  const before = Object.fromEntries(
    sourceFiles.map((file) => [file, hash(fs.readFileSync(path.join(root, file)))]),
  );
  const realCargo = execFileSync("rustup", ["which", "cargo"], {
    cwd: root,
    encoding: "utf8",
  }).trim();
  const bin = path.join(output, "observer-bin");
  fs.mkdirSync(bin);
  const invocation = path.join(output, "cargo-invocation.json");
  // Observe the fresh producer environment, then execute the exact pinned Cargo
  // binary with unchanged arguments and environment. Never mint/replace its token.
  fs.writeFileSync(
    path.join(bin, "cargo"),
    `#!${process.execPath}\n` +
      `const fs=require('node:fs');const cp=require('node:child_process');\n` +
      `fs.writeFileSync(${JSON.stringify(invocation)},JSON.stringify({argv:process.argv.slice(2),cwd:process.cwd(),token:process.env.VIZE_NATIVE_BUILD_TOKEN,cargo:${JSON.stringify(realCargo)}},null,2)+'\\n',{flag:'wx'});\n` +
      `const r=cp.spawnSync(${JSON.stringify(realCargo)},process.argv.slice(2),{env:process.env,stdio:'inherit'});if(r.error)throw r.error;process.exit(r.status??1);\n`,
    { mode: 0o755 },
  );
  const receipt = path.join(output, "build-receipt.json");
  const args = [
    "tools/support/github/build-napi-package.ts",
    "npm/native",
    "../../crates/vize_vitrine/Cargo.toml",
    "vize_vitrine",
    "x86_64-apple-darwin",
  ];
  const child = spawn(process.execPath, args, {
    cwd: root,
    env: {
      ...process.env,
      PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
      VIZE_RELEASE_VERSION: version,
      VIZE_NATIVE_BUILD_RECEIPT: receipt,
    },
  });
  child.stdout.on("data", (bytes: Buffer) => {
    fs.appendFileSync(path.join(output, "build.stdout"), bytes);
    process.stdout.write(bytes);
  });
  child.stderr.on("data", (bytes: Buffer) => {
    fs.appendFileSync(path.join(output, "build.stderr"), bytes);
    process.stderr.write(bytes);
  });
  const { status, signal } = await new Promise<{
    status: number | null;
    signal: NodeJS.Signals | null;
  }>((resolve, reject) => {
    child.once("error", reject);
    child.once("exit", (status, signal) => resolve({ status, signal }));
  });
  const observed = JSON.parse(fs.readFileSync(invocation, "utf8"));
  assert.match(observed.token, /^[0-9a-f]{32}$/u);
  const image = path.join(root, "npm/native/vize-vitrine.darwin-x64.node");
  const bytes = fs.readFileSync(image);
  fs.copyFileSync(image, path.join(output, "image.node"));
  const after = Object.fromEntries(
    sourceFiles.map((file) => [file, hash(fs.readFileSync(path.join(root, file)))]),
  );
  const rawTokenPresent = bytes.includes(Buffer.from(observed.token));
  json(path.join(output, "producer.json"), {
    schema: "vize.nativeTokenLayoutProducer.v1",
    role,
    source: expectedSource,
    tree: git(root, "rev-parse", "HEAD^{tree}"),
    sourceFiles: before,
    sourceFilesAfter: after,
    releaseVersion: version,
    target: "x86_64-apple-darwin",
    node: { executable: process.execPath, version: process.version, arch: process.arch },
    rustc: execFileSync("rustc", ["-vV"], { cwd: root, encoding: "utf8" }),
    cargo: execFileSync(realCargo, ["--version"], { cwd: root, encoding: "utf8" }),
    xcode: execFileSync("xcodebuild", ["-version"], { encoding: "utf8" }),
    command: { executable: process.execPath, argv: args },
    status,
    signal,
    token: observed.token,
    linker: {
      path: execFileSync("xcrun", ["--find", "clang"], { encoding: "utf8" }).trim(),
      version: execFileSync("clang", ["--version"], { encoding: "utf8" }),
    },
    profileEnvironment: Object.fromEntries(
      [
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_PROFILE_RELEASE_OPT_LEVEL",
        "CARGO_PROFILE_RELEASE_LTO",
        "CARGO_PROFILE_RELEASE_CODEGEN_UNITS",
        "CARGO_PROFILE_RELEASE_STRIP",
        "MACOSX_DEPLOYMENT_TARGET",
        "SDKROOT",
        "CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER",
      ].map((key) => [key, process.env[key] ?? null]),
    ),
    image: { bytes: bytes.length, sha256: hash(bytes), rawTokenPresent },
    scope: "Fresh exact-source release recipe; baseline failure is retained, never publishable.",
  });
  assert.deepEqual(after, before, "source or lock bytes changed during compilation");
  assert.equal(git(root, "rev-parse", "HEAD"), expectedSource);
  assert.equal(git(root, "diff", "--binary", "HEAD"), "");
  assert.equal(signal, null);
  if (role === "before") {
    assert.notEqual(status, 0);
    assert.equal(fs.existsSync(receipt), false);
    assert.equal(rawTokenPresent, false);
    assert.match(
      fs.readFileSync(path.join(output, "build.stderr"), "utf8"),
      /does not contain this invocation's build token/u,
    );
  } else {
    assert.equal(status, 0);
    assert.equal(rawTokenPresent, true);
    const actual = JSON.parse(fs.readFileSync(receipt, "utf8"));
    assert.deepEqual(actual, {
      schema: "vize.nativeBuildReceipt.v1",
      source: expectedSource,
      releaseVersion: version,
      rustTarget: "x86_64-apple-darwin",
      nodeFile: "vize-vitrine.darwin-x64.node",
      bytes: bytes.length,
      sha256: hash(bytes),
      nativeBuildToken: observed.token,
    });
  }
}
function consumer(output: string) {
  assert.equal(process.platform, "darwin");
  assert.equal(process.arch, "x64", "load the real x64 image in Intel Node");
  const observed = JSON.parse(fs.readFileSync(path.join(output, "producer.json"), "utf8"));
  assert.equal(observed.schema, "vize.nativeTokenLayoutProducer.v1");
  assert.match(observed.token, /^[0-9a-f]{32}$/u);
  const image = fs.realpathSync(path.join(output, "image.node"));
  const bytes = fs.readFileSync(image);
  assert.equal(hash(bytes), observed.image.sha256);
  assert.equal(bytes.length, observed.image.bytes);
  const require = createRequire(import.meta.url);
  const binding = require(image);
  assert.equal(typeof binding.getNativeBuildToken, "function");
  const actual = binding.getNativeBuildToken();
  json(path.join(output, "consumer.json"), {
    schema: "vize.nativeTokenLayoutConsumer.v1",
    source: observed.source,
    role: observed.role,
    target: observed.target,
    node: { executable: process.execPath, version: process.version, arch: process.arch },
    expected: observed.token,
    actual,
    rawTokenPresent: bytes.includes(Buffer.from(actual ?? "")),
    image: observed.image,
    imageAfterSha256: hash(fs.readFileSync(image)),
  });
  assert.equal(
    actual,
    observed.token,
    "actual Node getter differs from this exact producer invocation",
  );
  assert.equal(hash(fs.readFileSync(image)), observed.image.sha256);
  assert.equal(bytes.includes(Buffer.from(actual)), observed.role === "after");
}

const [mode, ...args] = process.argv.slice(2);
if (mode === "producer" && args.length === 4)
  await producer(args[0]!, args[1]!, args[2]!, args[3]!);
else if (mode === "consumer" && args.length === 1) consumer(args[0]!);
else
  throw new Error(
    "expected producer <source-root> <before|after> <source-sha> <output> or consumer <output>",
  );
