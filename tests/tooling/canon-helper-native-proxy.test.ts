import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
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
  sourceMapperSdk,
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
void test("source mapper shadows only the helper SDK needed by the fixed source image", () => {
  const root = mkdtempSync(join(tmpdir(), "canon source sdk "));
  const previousSourceSdk = process.env.VIZE_SOURCE_SDK_ROOT;
  try {
    const provider = join(root, "provider");
    const modules = join(provider, "node_modules");
    const publishedVize = join(modules, "vize");
    const publishedGenerated = join(publishedVize, "generated");
    const sourceSdk = join(root, "source-sdk");
    const sourceGenerated = join(sourceSdk, "generated");
    const project = join(root, "project");
    mkdirSync(join(modules, "vue"), { recursive: true });
    mkdirSync(publishedGenerated, { recursive: true });
    mkdirSync(sourceGenerated, { recursive: true });
    mkdirSync(project);
    writeFileSync(join(modules, "vue", "package.json"), '{"name":"vue"}\n');
    writeFileSync(
      join(publishedVize, "package.json"),
      JSON.stringify({ typescript: { contentMapper: { exec: ["published", "content-mapper"] } } }),
    );
    for (const declaration of [
      "content-mapper-helpers.d.cts",
      "content-mapper-helpers.d.mts",
      "content-mapper-helpers.d.ts",
    ]) {
      writeFileSync(join(publishedVize, declaration), "published helper\n");
      writeFileSync(join(sourceSdk, declaration), `source ${declaration}\n`);
    }
    writeFileSync(join(publishedGenerated, "content-mapper-types.d.ts"), "published types\n");
    writeFileSync(join(sourceGenerated, "content-mapper-types.d.ts"), "source types\n");
    symlinkSync(modules, join(project, "node_modules"), "dir");
    process.env.VIZE_SOURCE_SDK_ROOT = sourceSdk;

    const receipt = sourceMapperSdk(project, provider, ["source-vize", "content-mapper"]);
    const shadow = join(project, "node_modules", "vize");
    assert.equal(
      readFileSync(join(shadow, "content-mapper-helpers.d.ts"), "utf8"),
      "source content-mapper-helpers.d.ts\n",
    );
    assert.equal(
      readFileSync(join(shadow, "generated", "content-mapper-types.d.ts"), "utf8"),
      "source types\n",
    );
    assert.deepEqual(
      JSON.parse(readFileSync(join(shadow, "package.json"), "utf8")).typescript.contentMapper.exec,
      ["source-vize", "content-mapper"],
    );
    assert.equal(
      readFileSync(join(publishedGenerated, "content-mapper-types.d.ts"), "utf8"),
      "published types\n",
    );
    assert.ok(receipt.sourceSdk);
  } finally {
    if (previousSourceSdk === undefined) delete process.env.VIZE_SOURCE_SDK_ROOT;
    else process.env.VIZE_SOURCE_SDK_ROOT = previousSourceSdk;
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
