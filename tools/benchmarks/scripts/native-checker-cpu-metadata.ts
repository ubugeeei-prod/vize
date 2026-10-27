/** Inspect the exact captured native checker image before interpreting any Go PCs. */
import assert from "node:assert/strict";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { parseArgs } from "node:util";
import { pathToFileURL } from "node:url";
import { retainedNormalIdentity } from "./source-symbolizer-smoke.ts";
import { boundedCommand, digest } from "./source-symbolizer-render.ts";
import { renderSymbolCpu } from "./source-symbolizer-symbols.ts";
import { decodeNativeCheckerCpu } from "./native-checker-cpu-decode.ts";
import { collectRustCallchains } from "./native-checker-cpu-rust-callers.ts";
export const nativeCheckerImage = {
  sha256: "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
  path: "/home/runner/_work/_temp/provider/node_modules/@typescript/typescript-linux-x64/lib/tsc",
  version: "7.1.0-beta",
  normalSha256: "bc6d3fa4d1fa4488fe996a330248c52d51451ca6b9e94a88656b73ca1398996b",
  source: "d16307e5eebffa43277a4fd9e5a471b5312256cc",
  dataSha256: "fbe10d0345f26b8d38cb1c17ae4f0b575c7bc2ae1d712bb09a327fca471f50f3",
} as const;
export function validateNativeCheckerCapture(
  original: {
    identity: { backend: string; backendSha256: string; source: string };
    binarySha256: string;
    command: { args: string[] };
  },
  actualSdkSha256: string,
) {
  assert.equal(actualSdkSha256, nativeCheckerImage.sha256, "unrelated SDK image rejected");
  assert.equal(original.identity.backendSha256, nativeCheckerImage.sha256);
  assert.equal(original.identity.backend, nativeCheckerImage.path);
  assert.equal(original.identity.source, nativeCheckerImage.source);
  assert.equal(original.binarySha256, nativeCheckerImage.normalSha256);
  assert.equal(original.command.args.filter((arg) => arg === "--corsa-path").length, 1);
  const at = original.command.args.indexOf("--corsa-path");
  assert.ok(at >= 0);
  assert.equal(original.command.args[at + 1], nativeCheckerImage.path);
}
export function inspectNativeChecker(options: {
  root: string;
  out: string;
  provider: string;
  perf: string;
  go: string;
}) {
  assert.ok(!existsSync(options.out), "fresh metadata evidence required");
  const { original, build, binary } = retainedNormalIdentity(options.root);
  const sdkPackage = join(options.provider, "node_modules/@typescript/typescript-linux-x64");
  const sdk = join(sdkPackage, "lib/tsc");
  validateNativeCheckerCapture(original, digest(sdk));
  const pkg = JSON.parse(readFileSync(join(sdkPackage, "package.json"), "utf8"));
  assert.equal(pkg.name, "@typescript/typescript-linux-x64");
  assert.equal(pkg.version, nativeCheckerImage.version);
  const data = join(options.root, "record-head/direct-process/perf.data");
  assert.equal(digest(data), nativeCheckerImage.dataSha256);
  assert.ok(!existsSync(nativeCheckerImage.path), "do not overwrite an existing checker");
  mkdirSync(options.out, { recursive: true });
  mkdirSync(dirname(nativeCheckerImage.path), { recursive: true });
  copyFileSync(sdk, nativeCheckerImage.path);
  assert.equal(digest(nativeCheckerImage.path), nativeCheckerImage.sha256);
  writeFileSync(
    join(options.out, "provider-lock.json"),
    readFileSync(join(options.provider, "package-lock.json")),
  );
  writeFileSync(
    join(options.out, "sdk-package.json"),
    readFileSync(join(sdkPackage, "package.json")),
  );
  const env = { GOTOOLCHAIN: "local", GOTELEMETRY: "off", GOWORK: "off" };
  const tools = {
    go: boundedCommand(join(options.out, "go-version"), options.go, ["version"], 3000, undefined, {
      env,
    }),
    module: boundedCommand(
      join(options.out, "go-module"),
      options.go,
      ["version", "-m", nativeCheckerImage.path],
      10_000,
      undefined,
      { env },
    ),
    buildId: boundedCommand(
      join(options.out, "go-build-id"),
      options.go,
      ["tool", "buildid", nativeCheckerImage.path],
      10_000,
      undefined,
      { env },
    ),
    elf: boundedCommand(
      join(options.out, "sdk-elf"),
      "readelf",
      [
        "--file-header",
        "--program-headers",
        "--section-headers",
        "--notes",
        "--wide",
        nativeCheckerImage.path,
      ],
      10_000,
    ),
    trace: boundedCommand(
      join(options.out, "runtime-addresses"),
      options.perf,
      [
        "script",
        "--no-demangle",
        "--no-inline",
        "--hide-call-graph",
        "--show-mmap-events",
        "--show-task-events",
        "--ns",
        "--fields",
        "pid,tid,time,period,ip,dso,dsoff",
        "--input",
        data,
      ],
      30_000,
    ),
  };
  for (const [name, tool] of Object.entries(tools))
    assert.equal(tool.status, 0, name + ": " + tool.stderr);
  assert.match(tools.go.stdout, /\bgo1\.27\.2\b/u);
  assert.ok(tools.buildId.stdout.trim().length > 0);
  assert.match(tools.elf.stdout, /Machine:\s+Advanced Micro Devices X86-64/u);
  const symbols = renderSymbolCpu({
    binary,
    data,
    perf: options.perf,
    out: join(options.out, "restored-symbols"),
    totalSamples: original.sampleCount,
    identity: {
      compiledSource: build.sourceSha,
      build,
      originalCapture: original,
      sdkSha256: nativeCheckerImage.sha256,
    },
  });
  assert.ok(symbols.qualified);
  assert.equal(symbols.census.allDsos.find((entry) => entry.dso === "tsc")?.samples, 1380);
  const unchanged =
    digest(binary) === original.binarySha256 &&
    digest(sdk) === nativeCheckerImage.sha256 &&
    digest(nativeCheckerImage.path) === nativeCheckerImage.sha256 &&
    digest(data) === nativeCheckerImage.dataSha256;
  const result = {
    schemaVersion: 1,
    stage: "metadata-only",
    provenanceQualified: unchanged,
    observerSource: process.env.GITHUB_SHA ?? null,
    compiledSource: build.sourceSha,
    normalBinarySha256: original.binarySha256,
    dataSha256: nativeCheckerImage.dataSha256,
    sdk: {
      sha256: nativeCheckerImage.sha256,
      bytes: statSync(sdk).size,
      restoredPath: nativeCheckerImage.path,
      package: pkg.name,
      version: pkg.version,
      goBuildId: tools.buildId.stdout.trim(),
      moduleMetadata: tools.module.stdout,
      hasPclntab: /\.gopclntab|\.data\.rel\.ro\.gopclntab/u.test(tools.elf.stdout),
    },
    tools: Object.fromEntries(
      Object.entries(tools).map(([name, tool]) => [
        name,
        {
          executable: tool.executable,
          args: tool.args,
          status: tool.status,
          wallMs: tool.wallMs,
        },
      ]),
    ),
    capturedSampleCensus: symbols.census,
    goFunctionsQualified: false,
    sourceLineQualified: false,
    hardwareInstructions: null,
    sourceTreeOwnershipVerified: false,
    scope:
      "same public SDK bytes and recorded addresses only; no replacement debug image, new capture, program run, or instruction count",
  };
  writeFileSync(join(options.out, "metadata.json"), JSON.stringify(result, null, 2) + "\n");
  assert.ok(result.provenanceQualified);
  return result;
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      root: { type: "string" },
      out: { type: "string" },
      provider: { type: "string" },
      perf: { type: "string" },
      go: { type: "string", default: "go" },
    },
  });
  assert.ok(values.root && values.out && values.provider && values.perf);
  const metadata = inspectNativeChecker({
    root: resolve(values.root),
    out: resolve(values.out),
    provider: resolve(values.provider),
    perf: values.perf,
    go: values.go!,
  });
  const decoded = decodeNativeCheckerCpu({
    sdk: nativeCheckerImage.path,
    sdkSha256: nativeCheckerImage.sha256,
    data: join(resolve(values.root), "record-head/direct-process/perf.data"),
    dataSha256: nativeCheckerImage.dataSha256,
    trace: join(resolve(values.out), "runtime-addresses/stdout.txt"),
    elf: join(resolve(values.out), "sdk-elf/stdout.txt"),
    out: join(resolve(values.out), "go-attribution"),
    go: values.go!,
    identity: {
      compiledSource: metadata.compiledSource,
      observerSource: metadata.observerSource,
      normalBinarySha256: metadata.normalBinarySha256,
      sdk: metadata.sdk,
    },
  });
  collectRustCallchains({
    binary: join(resolve(values.root), "binaries/normal-head/vize"),
    binarySha256: nativeCheckerImage.normalSha256,
    data: join(resolve(values.root), "record-head/direct-process/perf.data"),
    dataSha256: nativeCheckerImage.dataSha256,
    trace: join(resolve(values.out), "runtime-addresses/stdout.txt"),
    perf: values.perf,
    out: join(resolve(values.out), "rust-callers"),
    source: nativeCheckerImage.source,
  });
  console.log(
    JSON.stringify({
      qualified: decoded.qualified,
      checkerSampleCount: decoded.checkerSampleCount,
      resolvedFunctionSamples: decoded.resolvedFunctionSamples,
      unresolvedFunctionSamples: decoded.unresolvedFunctionSamples,
      hardwareInstructions: null,
    }),
  );
}
