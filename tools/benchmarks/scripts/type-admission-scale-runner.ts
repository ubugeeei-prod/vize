import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";
import {
  prepareAdmissionScale,
  scaleDefaults,
  type ScaleDiagnostic,
} from "./type-admission-scale.ts";
import type { ScaleOptions, ScaleShape, ScaleStage } from "./type-admission-scale-types.ts";
import { scaleCliIdentity, scaleNativeIdentity } from "./type-admission-scale-identity.ts";

type Tool = "cli" | "mapper" | "vue-tsc";
type Capture = {
  provider: string;
  out: string;
  tool: Tool;
  scale: ScaleOptions;
  binary?: string;
  source?: string;
  mapperSource?: string;
  timeoutMs?: number;
};
const sha = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const identity = (entry: ScaleDiagnostic) => JSON.stringify(entry);

export function scaleDiagnostics(stdout: string, tool: Tool, project: string) {
  const normalize = (file: string) => {
    if (path.isAbsolute(file)) file = path.relative(project, file);
    return file.replaceAll("\\", "/").replace(/^\.\//, "");
  };
  if (tool !== "cli") {
    const diagnostics = [...stdout.matchAll(/^(.+?)\((\d+),(\d+)\): error TS(\d+): (.*)$/gm)].map(
      (m) => ({ file: normalize(m[1]), line: +m[2], column: +m[3], code: +m[4], message: m[5] }),
    );
    assert.equal(
      (stdout.match(/error TS\d+:/g) ?? []).length,
      diagnostics.length,
      "every native/compiler diagnostic needs a complete authored identity",
    );
    return diagnostics;
  }
  const report = JSON.parse(stdout) as {
    files: { file: string; diagnostics: string[] }[];
    errorCount: number;
    warningCount: number;
  };
  const diagnostics = report.files.flatMap((file) =>
    file.diagnostics.map((text) => {
      const m = /^(error|warning):(\d+):(\d+) \[TS(\d+)\] ([\s\S]+)$/.exec(text);
      assert.ok(m, "unclassified authored diagnostic: " + text);
      assert.equal(m[1], "error", "unexpected warnings must not disappear from admission");
      return { file: normalize(file.file), line: +m[2], column: +m[3], code: +m[4], message: m[5] };
    }),
  );
  assert.equal(report.errorCount, diagnostics.length);
  assert.equal(report.warningCount, 0);
  return diagnostics;
}

export function captureAdmissionScale(options: Capture) {
  const { provider, tool } = options;
  const sourceBuild = scaleCliIdentity(options.binary, options.source);
  fs.mkdirSync(options.out, { recursive: true });
  const project = path.join(options.out, "project");
  const corpus = prepareAdmissionScale(project, provider, options.scale);
  const require = createRequire(path.resolve(provider, "package.json"));
  const nativeRequire = createRequire(require.resolve("typescript-native/package.json"));
  const platform = `@typescript/typescript-${process.platform}-${process.arch}`;
  const checker = path.join(
    path.dirname(nativeRequire.resolve(`${platform}/package.json`)),
    "lib",
    process.platform === "win32" ? "tsc.exe" : "tsc",
  );
  const cli = options.binary ?? path.join(provider, "node_modules/vize/bin/vize");
  const command =
    tool === "mapper" ? checker : tool === "vue-tsc" || !options.binary ? process.execPath : cli;
  const argv =
    tool === "mapper"
      ? ["--runExternalCode", "--noEmit", "--pretty", "false", "-p", "tsconfig.mapper.json"]
      : tool === "vue-tsc"
        ? [
            path.join(provider, "node_modules/vue-tsc/bin/vue-tsc.js"),
            "--noEmit",
            "--pretty",
            "false",
            "-p",
            "tsconfig.json",
          ]
        : [
            ...(options.binary ? [] : [cli]),
            "check",
            "--no-config",
            "--format",
            "json",
            "--tsconfig",
            "tsconfig.json",
            "--corsa-path",
            checker,
          ];
  const resourcePath = path.join(options.out, "resource.txt");
  const time = fs.existsSync("/usr/bin/time") && ["linux", "darwin"].includes(process.platform);
  const wrapper = time ? "/usr/bin/time" : command;
  const wrapperArgv = time
    ? [...(process.platform === "linux" ? ["-v", "-o", resourcePath] : ["-l"]), command, ...argv]
    : argv;
  const environment = {
    ...process.env,
    XDG_CACHE_HOME: path.join(options.out, "xdg-cache"),
    TMPDIR: path.join(options.out, "tmp"),
  };
  fs.mkdirSync(environment.TMPDIR);
  const started = performance.now();
  const run = spawnSync(wrapper, wrapperArgv, {
    cwd: project,
    env: environment,
    encoding: "utf8",
    timeout: options.timeoutMs ?? 120_000,
    maxBuffer: 128 * 1024 * 1024,
  });
  const elapsedMs = performance.now() - started;
  const stdout = run.stdout ?? "",
    stderr = run.stderr ?? "";
  const resource = time
    ? process.platform === "linux"
      ? fs.readFileSync(resourcePath, "utf8")
      : stderr
    : "";
  const rss =
    process.platform === "linux"
      ? resource.match(/Maximum resident set size \(kbytes\):\s*(\d+)/)?.[1]
      : resource.match(/(\d+)\s+maximum resident set size/)?.[1];
  const versions = Object.fromEntries(
    ["vize", "@vizejs/native", "vue", "vue-tsc", "typescript", "typescript-native"].map((name) => [
      name,
      JSON.parse(fs.readFileSync(path.join(provider, "node_modules", name, "package.json"), "utf8"))
        .version,
    ]),
  );
  const nativeBinding = scaleNativeIdentity(provider, options.mapperSource);
  const entrypoint = tool === "vue-tsc" ? argv[0] : tool === "mapper" ? checker : cli;
  const vueRequire =
    tool === "vue-tsc" ? createRequire(require.resolve("vue-tsc/package.json")) : null;
  const vueCompiler = vueRequire?.resolve("typescript/lib/tsc.js");
  const javascriptCompiler =
    vueRequire && vueCompiler
      ? {
          version: JSON.parse(
            fs.readFileSync(vueRequire.resolve("typescript/package.json"), "utf8"),
          ).version,
          files: [
            vueCompiler,
            path.join(path.dirname(vueCompiler), "_tsc.js"),
            vueRequire.resolve("typescript"),
          ]
            .filter((file) => fs.existsSync(file))
            .map((file) => ({ path: file, sha256: sha(fs.readFileSync(file)) })),
        }
      : null;
  let accuracy: { passed: boolean; diagnostics?: unknown; error?: string };
  try {
    assert.equal(run.error, undefined);
    assert.equal(run.signal, null);
    const diagnostics = scaleDiagnostics(stdout, tool, project);
    const actual = diagnostics.map(({ file, line, column, code }) => ({
      file,
      line,
      column,
      code,
    }));
    assert.deepEqual(
      actual.map(identity).sort(),
      corpus.expected.map(identity).sort(),
      "all unique authored probes, duplicate multiplicity, and source locations are mandatory",
    );
    assert.equal(run.status, corpus.expected.length ? (tool === "cli" ? 1 : 2) : 0);
    accuracy = { passed: true, diagnostics };
  } catch (error) {
    accuracy = { passed: false, error: error instanceof Error ? error.stack : String(error) };
  }
  const receipt = {
    schema: 2,
    tool,
    options: options.scale,
    corpusSha256: sha(fs.readFileSync(path.join(project, "corpus.json"))),
    command,
    argv,
    wrapper,
    wrapperArgv,
    cwd: project,
    versions,
    source:
      tool === "cli" && options.binary
        ? options.source
        : tool === "mapper"
          ? (options.mapperSource ?? `installed-native-${versions["@vizejs/native"]}`)
          : `installed-${tool}`,
    binarySha256: sha(fs.readFileSync(command)),
    entrypoint: { path: entrypoint, sha256: sha(fs.readFileSync(entrypoint)) },
    javascriptCompiler,
    checkerSha256: sha(fs.readFileSync(checker)),
    checkerRole:
      tool === "vue-tsc" ? "comparison-provider-not-executed" : "selected-native-backend",
    sourceBuild,
    nativeBinding,
    nativeBindingRole:
      tool === "vue-tsc"
        ? "comparison-provider-observer-only"
        : tool === "cli" && options.binary
          ? "installed-provider-observer-only"
          : "selected-provider",
    status: run.status,
    signal: run.signal,
    error: run.error?.message ?? null,
    stdoutBase64: Buffer.from(stdout).toString("base64"),
    stderrBase64: Buffer.from(stderr).toString("base64"),
    resource,
    elapsedMs,
    maxProcessRssBytes: rss ? +rss * (process.platform === "linux" ? 1024 : 1) : null,
    rssMeaning: "time-reported process maximum; not simultaneous aggregate process-tree RSS",
    threads: {
      gomaxprocs: process.env.GOMAXPROCS ?? null,
      rayon: process.env.RAYON_NUM_THREADS ?? null,
    },
    machine: {
      platform: process.platform,
      arch: process.arch,
      cpus: os.cpus().length,
      model: os.cpus()[0]?.model,
    },
    accuracy,
    sfcs: corpus.sfcs.length,
    totalOriginalSourceBytes: corpus.sourceBytes,
    cold: corpus.cold,
  };
  fs.writeFileSync(path.join(options.out, "receipt.json"), JSON.stringify(receipt, null, 2));
  return receipt;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const { values } = parseArgs({
    options: {
      provider: { type: "string" },
      out: { type: "string" },
      tool: { type: "string", default: "cli" },
      binary: { type: "string" },
      source: { type: "string" },
      "mapper-source": { type: "string" },
      sfcs: { type: "string" },
      "sfc-bytes": { type: "string" },
      "graph-nodes": { type: "string" },
      shape: { type: "string" },
      "type-width": { type: "string" },
      "type-depth": { type: "string" },
      stage: { type: "string", default: "valid" },
      "allow-large": { type: "boolean" },
    },
  });
  assert.ok(values.provider && values.out, "--provider and --out are required");
  assert.ok(["cli", "mapper", "vue-tsc"].includes(values.tool!));
  const report = captureAdmissionScale({
    provider: path.resolve(values.provider),
    out: path.resolve(values.out),
    tool: values.tool as Tool,
    binary: values.binary && path.resolve(values.binary),
    source: values.source,
    mapperSource: values["mapper-source"],
    scale: {
      sfcs: +(values.sfcs ?? scaleDefaults.sfcs),
      sfcBytes: +(values["sfc-bytes"] ?? scaleDefaults.sfcBytes),
      graphNodes: +(values["graph-nodes"] ?? scaleDefaults.graphNodes),
      shape: (values.shape ?? scaleDefaults.shape) as ScaleShape,
      typeWidth: +(values["type-width"] ?? scaleDefaults.typeWidth),
      typeDepth: +(values["type-depth"] ?? scaleDefaults.typeDepth),
      stage: values.stage as ScaleStage,
      allowLarge: values["allow-large"],
    },
  });
  console.log(
    JSON.stringify({
      tool: report.tool,
      sfcs: report.sfcs,
      elapsedMs: report.elapsedMs,
      maxProcessRssBytes: report.maxProcessRssBytes,
      accuracy: report.accuracy.passed,
      receipt: path.join(values.out, "receipt.json"),
    }),
  );
  if (!report.accuracy.passed) process.exitCode = 1;
}
