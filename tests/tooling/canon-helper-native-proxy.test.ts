import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { PassThrough, Writable } from "node:stream";
import { finished } from "node:stream/promises";
import {
  nativeProxyLauncher,
  classifyNativeInvocation,
  nativeObserverCommand,
  teeNativeBytes,
} from "../../tools/benchmarks/scripts/canon-helper-native-proxy.ts";
import {
  admitSourceRegeneration,
  historicalBinarySha256,
  sourceImageBuildRecipe,
  sourceRevision,
  sourceTree,
} from "../../tools/benchmarks/scripts/canon-helper-baseline-provider.ts";
import { hash } from "../../tools/benchmarks/scripts/canon-instance-directives-io.ts";
void test("native stats distinguish compile, probe and explicit empty options validation", () => {
  const cfg = { include: ["src/**/*"] };
  assert.equal(classifyNativeInvocation(["-p", "tsconfig.json"], cfg, true).actualProgram, true);
  assert.equal(classifyNativeInvocation(["--version"], cfg, true).actualProgram, false);
  assert.equal(
    classifyNativeInvocation(["--showConfig", "-p", "x"], cfg, true).actualProgram,
    false,
  );
  assert.equal(
    classifyNativeInvocation(["-p", "x"], { files: [], include: [] }, true).classification,
    "options-validation",
  );
  assert.equal(classifyNativeInvocation(["-p", "x"], null, false).actualProgram, false);
  for (const option of ["--api", "--lsp=stdio", "--stdio", "--server=127.0.0.1"]) {
    const classified = classifyNativeInvocation([option, "-p", "x"], cfg, true);
    assert.deepEqual(classified, { classification: "server", actualProgram: false });
    assert.deepEqual(nativeObserverCommand([option], {}, classified).args, [option]);
  }
  const args = ["--checkers", "1", "-p", "tsconfig.json"],
    env = {
      GOGC: "75",
      GOMEMLIMIT: "2GiB",
      GODEBUG: "madvdontneed=1,gctrace=0",
    },
    argsBefore = [...args],
    envBefore = { ...env };
  const observed = nativeObserverCommand(args, env, { actualProgram: true });
  assert.deepEqual(observed.args, [...args, "--extendedDiagnostics"]);
  assert.equal(observed.env.GODEBUG, "madvdontneed=1,gctrace=1");
  assert.deepEqual(args, argsBefore);
  assert.deepEqual(env, envBefore);
  assert.deepEqual(nativeObserverCommand(["--version"], env, { actualProgram: false }).args, [
    "--version",
  ]);
  assert.deepEqual(nativeObserverCommand(["--version"], env, { actualProgram: false }).env, env);
});
void test("proxy tee forwards exact binary bytes with backpressure independently to both sinks", async () => {
  const input = new PassThrough(),
    received: Buffer[] = [],
    recorded: Buffer[] = [];
  const sink = new Writable({
    highWaterMark: 1,
    write(chunk: Buffer, _encoding, done) {
      received.push(Buffer.from(chunk));
      setTimeout(done, 1);
    },
  });
  const archive = new Writable({
    highWaterMark: 1,
    write(chunk: Buffer, _encoding, done) {
      recorded.push(Buffer.from(chunk));
      setTimeout(done, 2);
    },
  });
  const bytes = Buffer.concat([
    Buffer.from([0, 255, 13, 10]),
    Buffer.from("Content-Length: 7\r\n\r\n雪👋\n"),
  ]);
  const tee = teeNativeBytes(input, sink, archive, true);
  for (let index = 0; index < bytes.length; index++) input.write(bytes.subarray(index, index + 1));
  input.end();
  await Promise.all([tee.done, finished(sink)]);
  assert.deepEqual(Buffer.concat(received), bytes);
  assert.deepEqual(Buffer.concat(recorded), bytes);
  assert.equal(tee.snapshot().bytes, bytes.length);
  assert.match(tee.snapshot().sha256, /^[0-9a-f]{64}$/u);
});
void test("source regeneration receipt binds producer bytes, toolchain and recipe", () => {
  const root = mkdtempSync(join(tmpdir(), "canon source receipt "));
  try {
    const binary = join(root, "vize"),
      sidecarFile = binary + ".differential-build.json",
      linker = join(root, "wild"),
      sourceBuildFile = join(root, "source-build.json"),
      regenerationFile = join(root, "regeneration.json"),
      binaryBytes = Buffer.from("fresh source binary"),
      linkerBytes = Buffer.from("pinned wild linker"),
      cargoVersion = "cargo 1.99.1\nrelease: 1.99.1\nhost: x86_64-unknown-linux-gnu\n",
      rustcVersion = "rustc 1.99.1\nrelease: 1.99.1\nLLVM version: 23.1.1\n";
    const writeJson = (file: string, value: unknown) =>
      writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
    writeFileSync(binary, binaryBytes);
    writeFileSync(linker, linkerBytes);
    const binarySha256 = hash(binaryBytes);
    writeJson(sidecarFile, { sourceRevision, binarySha256 });
    const differentialReceiptSha256 = hash(readFileSync(sidecarFile));
    writeJson(sourceBuildFile, {
      schema: "vize.cold-source-build",
      version: 1,
      sourceSha: sourceRevision,
      sourceTree,
      targetStartedAbsent: true,
      profile: "release",
      binarySha256,
      binaryBytes: binaryBytes.length,
      differentialReceiptSha256,
      cargo: { args: sourceImageBuildRecipe, version: cargoVersion },
      rustc: rustcVersion,
    });
    writeJson(regenerationFile, {
      schema: "vize.canon-helper-source-regeneration",
      version: 1,
      sourceRevision,
      sourceTree,
      binarySha256,
      binaryBytes: binaryBytes.length,
      differentialReceiptSha256,
      sourceBuildReceipt: {
        file: "source-build.json",
        sha256: hash(readFileSync(sourceBuildFile)),
      },
      buildRecipe: sourceImageBuildRecipe,
      toolchain: {
        rust: "1.99.1",
        cargoVersion,
        rustcVersion,
        linker: {
          name: "wild",
          version: "0.9.0",
          actionRevision: "0bbbfa5df4380cab8e63cb8505a1ce65e1d10203",
          path: linker,
          bytes: linkerBytes.length,
          sha256: hash(linkerBytes),
          versionOutput: "wild 0.9.0",
        },
      },
      historicalBinarySha256,
    });
    const admitted = admitSourceRegeneration(binary, regenerationFile);
    assert.equal(admitted.binaryIdentity.sha256, binarySha256);
    assert.equal(admitted.linkerIdentity.sha256, hash(linkerBytes));
    writeFileSync(binary, "tampered source binary");
    assert.throws(() => admitSourceRegeneration(binary, regenerationFile));
    writeFileSync(binary, binaryBytes);
    writeFileSync(linker, "tampered linker");
    assert.throws(() => admitSourceRegeneration(binary, regenerationFile));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
void test("POSIX launchers keep detection basenames separate and preserve exact argv", () => {
  const root = mkdtempSync(join(tmpdir(), "canon native launcher '"));
  try {
    const node = join(root, "node stub's"),
      observed = join(root, "observed argv"),
      module = join(root, "proxy module's.ts"),
      out = join(root, "capture output"),
      executable = join(root, "sdk detection", "tsc"),
      invocation = ["argument with spaces", `quotes 'and' "both"`];
    writeFileSync(
      node,
      '#!/bin/sh\n: "${NATIVE_LAUNCHER_ARGS:?}"\nprintf \'%s\\n\' "$@" > "$NATIVE_LAUNCHER_ARGS"\n',
    );
    chmodSync(node, 0o755);
    for (const mode of ["stats", "rpc"] as const) {
      const launcher = join(root, mode === "stats" ? "tsgo" : "mapper");
      writeFileSync(launcher, nativeProxyLauncher({ node, module, out, executable, mode }));
      chmodSync(launcher, 0o755);
      const result = spawnSync(launcher, invocation, {
        env: { ...process.env, NATIVE_LAUNCHER_ARGS: observed },
        encoding: "utf8",
      });
      assert.equal(result.status, 0, result.stderr);
      assert.deepEqual(readFileSync(observed, "utf8").trimEnd().split("\n"), [
        module,
        mode,
        out,
        executable,
        ...(mode === "stats" ? invocation : []),
      ]);
    }
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
