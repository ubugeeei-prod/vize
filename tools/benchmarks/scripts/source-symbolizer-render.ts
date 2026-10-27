/** Bounded post-processing, kept outside the measured program window. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { basename, join } from "node:path";
import { performance } from "node:perf_hooks";
import { parsePerfEventStats } from "./source-operation-perf.ts";
import { sentinelAdapterScript } from "./source-symbolizer-adapter.ts";
import {
  parseFlatSourceReport,
  completeKnownFrames,
  rustSourceCensus,
  sentinelStyle,
} from "./source-symbolizer-protocol.ts";

export function digest(file: string) {
  const result = spawnSync("sha256sum", ["--binary", "--", file], {
    encoding: "utf8",
    timeout: 30_000,
  });
  assert.equal(result.status, 0, result.stderr);
  const hash = result.stdout.split(/\s/u)[0];
  assert.match(hash, /^[0-9a-f]{64}$/u);
  return hash;
}
export function boundedCommand(
  out: string,
  executable: string,
  args: string[],
  timeoutMs: number,
  input?: string,
  execution?: { cwd?: string; env?: NodeJS.ProcessEnv },
) {
  mkdirSync(out, { recursive: true });
  const startedAt = new Date().toISOString(),
    start = performance.now();
  const supervised = [
    "--signal=TERM",
    "--kill-after=5s",
    `${timeoutMs / 1000}s`,
    executable,
    ...args,
  ];
  const env = {
    ...process.env,
    ...execution?.env,
    LC_ALL: "C",
    PERF_CONFIG: execution?.env?.PERF_CONFIG ?? "/dev/null",
    DEBUGINFOD_URLS: "",
    LLVM_ADDR2LINE_OPTS: "",
    LLVM_SYMBOLIZER_OPTS: "",
  };
  const result = spawnSync("/usr/bin/timeout", supervised, {
    encoding: "utf8",
    cwd: execution?.cwd,
    input,
    env,
    timeout: timeoutMs + 6000,
    killSignal: "SIGKILL",
    maxBuffer: 32 * 1024 * 1024,
  });
  const receipt = {
    executable,
    args,
    cwd: execution?.cwd ?? process.cwd(),
    startedAt,
    finishedAt: new Date().toISOString(),
    wallMs: performance.now() - start,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    supervisor: ["/usr/bin/timeout", ...supervised],
    environment: {
      LC_ALL: env.LC_ALL,
      PERF_CONFIG: env.PERF_CONFIG,
      DEBUGINFOD_URLS: env.DEBUGINFOD_URLS,
      LLVM_ADDR2LINE_OPTS: env.LLVM_ADDR2LINE_OPTS,
      LLVM_SYMBOLIZER_OPTS: env.LLVM_SYMBOLIZER_OPTS,
    },
  };
  writeFileSync(join(out, "stdout.txt"), result.stdout ?? "");
  writeFileSync(join(out, "stderr.txt"), result.stderr ?? "");
  writeFileSync(join(out, "invocation.json"), JSON.stringify(receipt, null, 2) + "\n");
  return { ...receipt, stdout: result.stdout ?? "", stderr: result.stderr ?? "" };
}
export function createSentinelAdapter(out: string, decoder: string) {
  assert.ok(existsSync(decoder));
  mkdirSync(out, { recursive: true });
  const path = join(out, "llvm-sentinel-adapter");
  assert.ok(!existsSync(path));
  const script = sentinelAdapterScript(decoder);
  writeFileSync(path, script, { mode: 0o755 });
  const python = boundedCommand(join(out, "python-version"), "python3", ["--version"], 3000);
  assert.equal(python.status, 0);
  writeFileSync(
    join(out, "adapter.json"),
    JSON.stringify(
      {
        decoder,
        decoderSha256: digest(decoder),
        wrapper: path,
        wrapperSha256: digest(path),
        driver: python.stdout.trim(),
        script,
        inputChange: "standalone comma becomes reserved noncanonical 0xdeadbeefbadc0ffe",
        outputChange: "only its exact unknown 3-line synthetic frame becomes comma",
        sourceOutputChange:
          "genuine function/source/address bytes pass unchanged; malformed sentinel is fatal",
      },
      null,
      2,
    ) + "\n",
  );
  return path;
}
export function probeDecoders(
  binary: string,
  out: string,
  candidates: string[],
  witness: { symbol: string; source: string; timeoutMs?: number } = {
    symbol: "instruction_work",
    source: "pmu_calibration.rs",
  },
) {
  assert.match(witness.symbol, /^[A-Za-z_]+$/u);
  const dwarf = boundedCommand(
    join(out, "sections"),
    "readelf",
    ["--section-headers", "--wide", binary],
    10_000,
  );
  assert.equal(dwarf.status, 0);
  assert.ok(dwarf.stdout.includes(".debug_line"), "actual carrier lacks line tables");
  const quote = (value: string) => "'" + value.replaceAll("'", "'\\''") + "'";
  const nm = boundedCommand(
    join(out, "symbols"),
    "/bin/bash",
    [
      "-o",
      "pipefail",
      "-c",
      "nm --defined-only --numeric-sort " +
        quote(binary) +
        " | awk " +
        quote("/ [tT] .*" + witness.symbol + "/ { print; found=1 } END { if (!found) exit 1 }"),
    ],
    10_000,
  );
  assert.equal(nm.status, 0);
  const address = /^\s*([0-9a-f]+)\s+[tT]\s+\S+$/mu.exec(nm.stdout)?.[1];
  assert.ok(address, "known original Rust function missing: " + witness.symbol);
  const addresses = [1n, 16n, 32n].map((offset) => (BigInt("0x" + address) + offset).toString(16));
  return candidates.map((decoder, ordinal) => {
    if (!existsSync(decoder)) return { decoder, qualified: false, reason: "not installed" };
    const root = join(out, String(ordinal));
    const version = boundedCommand(join(root, "version"), decoder, ["--version"], 3000);
    const sentinel = boundedCommand(
      join(root, "sentinel"),
      decoder,
      ["-e", binary, "-a", "-i", "-f"],
      witness.timeoutMs ?? 3000,
      ",\n",
    );
    const known = boundedCommand(
      join(root, "known-source"),
      decoder,
      ["-e", binary, "-a", "-i", "-f"],
      witness.timeoutMs ?? 3000,
      ",\n" + addresses.map((value) => value + "\n,\n").join(""),
    );
    const style = sentinelStyle(sentinel.stdout);
    const framesComplete = completeKnownFrames(known.stdout, addresses, style);
    const qualified =
      framesComplete &&
      version.status === 0 &&
      sentinel.status === 0 &&
      known.status === 0 &&
      !sentinel.error &&
      !known.error &&
      style !== null &&
      known.stdout.includes(witness.source) &&
      /\.rs:[1-9]\d*/u.test(known.stdout);
    return {
      decoder,
      qualified,
      reason: qualified ? null : "sentinel or known-source protocol failed",
      style,
      version: version.stdout.trim(),
      knownAddresses: addresses,
      framesComplete,
      sentinelWallMs: sentinel.wallMs,
      knownQueryWallMs: known.wallMs,
      firstDataLatencyMs: null,
      ioScope: "completed request/EOF wall bounds; first-byte time is not sampled",
    };
  });
}
export function renderSourceCpu(options: {
  binary: string;
  data: string;
  decoder: string;
  perf: string;
  out: string;
  totalSamples: number;
  requiredSource?: string;
  reportTimeoutMs?: number;
}) {
  assert.ok(!existsSync(options.out), "refuse to overwrite source decoding evidence");
  mkdirSync(options.out, { recursive: true });
  const counted = boundedCommand(
    join(options.out, "event-census"),
    options.perf,
    ["report", "--stdio", "--stats", "--input", options.data],
    30_000,
  );
  assert.equal(counted.status, 0);
  assert.equal(
    parsePerfEventStats(counted.stdout),
    options.totalSamples,
    "exact raw sample census must match the capture",
  );
  const config = join(options.out, "perfconfig");
  writeFileSync(config, "[core]\n\taddr2line-timeout = 10000\n");
  const before = digest(options.binary),
    dataSha256 = digest(options.data);
  const reports = Object.fromEntries(
    ["self", "inclusive"].map((kind) => {
      const command = boundedCommand(
        join(options.out, kind),
        options.perf,
        [
          "report",
          "--stdio",
          kind === "self" ? "--no-children" : "--children",
          "--call-graph",
          "none",
          "--no-inline",
          "--full-source-path",
          "--show-nr-samples",
          "--show-total-period",
          "--fields",
          kind === "self"
            ? "overhead,sample,period,dso,symbol,srcline"
            : "overhead_children,overhead,sample,period,dso,symbol,srcline",
          "--field-separator",
          "\t",
          "--percent-limit",
          "0",
          "--percentage",
          "absolute",
          "--sort",
          "dso,symbol,srcline",
          "--addr2line",
          options.decoder,
          "--input",
          options.data,
        ],
        options.reportTimeoutMs ?? 120_000,
        undefined,
        { env: { PERF_CONFIG: config } },
      );
      const parsed = parseFlatSourceReport(command.stdout);
      const protocolErrors =
        /configuration failed|could not read (?:first|sentinel) record|unexpected record instead of sentinel|failed to read.*(?:DWARF|debug)|addr2line.*(?:failed|error)/iu.test(
          command.stderr,
        );
      return [
        kind,
        {
          status: command.status,
          error: command.error,
          protocolErrors,
          qualifiedFormat: parsed.qualifiedFormat,
          rows: parsed.rows,
          census:
            kind === "self"
              ? rustSourceCensus(
                  parsed.rows,
                  basename(options.binary),
                  options.totalSamples,
                  options.requiredSource,
                )
              : null,
        },
      ];
    }),
  );
  const unchanged = digest(options.binary) === before && digest(options.data) === dataSha256;
  const result = {
    schemaVersion: 1,
    binarySha256: before,
    dataSha256,
    unchanged,
    decoder: options.decoder,
    totalSamples: options.totalSamples,
    reports,
    qualified:
      unchanged &&
      reports.self.status === 0 &&
      !reports.self.error &&
      !reports.self.protocolErrors &&
      reports.self.qualifiedFormat &&
      reports.self.census?.qualified === true &&
      reports.inclusive.status === 0 &&
      !reports.inclusive.protocolErrors &&
      reports.inclusive.qualifiedFormat,
    metric: "sampled user cpu-clock source attribution; no hardware instruction counts",
    aggregation:
      "self counts are disjoint sampled events; inclusive percentages overlap ancestors and are reported separately",
    targetDso: basename(options.binary),
    hardwareInstructions: null,
    inclusiveScope:
      "per-thread call-chain ancestors; child-process CPU is not transferred into a parent function",
    displayThresholdPercent: 0,
    addr2lineReadTimeoutMs: 10000,
    reportTimeoutMs: options.reportTimeoutMs ?? 120_000,
    postProcessingOutsideMeasuredWindow: true,
  };
  writeFileSync(join(options.out, "source-cpu.json"), JSON.stringify(result, null, 2) + "\n");
  return result;
}
