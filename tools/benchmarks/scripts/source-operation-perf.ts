import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import {
  cacheEvents,
  coreEvents,
  counterSanity,
  fatalPerfCapability,
  hardwareCalibration,
  optionalText,
  parsePerfEventStats,
  parsePerfStat,
  softwareEvents,
  sourcePerfEnvironment,
  symbolizationErrors,
} from "./source-operation-perf-evidence.ts";
export {
  parsePerfEventStats,
  parsePerfStat,
  sourcePerfEnvironment,
} from "./source-operation-perf-evidence.ts";

export interface SourceOperationCommand {
  executable: string;
  args: string[];
  cwd: string;
  /** Explicit non-secret variables. The inherited environment is not exported. */
  env: Record<string, string>;
}

export interface SourceOperationPerfRequest {
  mode: "stat" | "record";
  perfExecutable: string;
  outputDir: string;
  command: SourceOperationCommand;
  identity: Record<string, unknown>;
  addr2lineExecutable?: string;
  eventGroup?: "core" | "cache";
  calibrationPath?: string;
  timeoutMs?: number;
  reportTimeoutMs?: number;
}

/** A timeout supervisor is outside perf's measured child and kills its process group. */
function runBounded(
  executable: string,
  args: string[],
  timeoutMs: number,
  command: SourceOperationCommand,
) {
  const supervisor = process.platform === "linux" && existsSync("/usr/bin/timeout");
  return spawnSync(
    supervisor ? "/usr/bin/timeout" : executable,
    supervisor
      ? ["--signal=TERM", "--kill-after=5s", `${timeoutMs / 1000}s`, executable, ...args]
      : args,
    {
      cwd: command.cwd,
      env: {
        ...process.env,
        ...command.env,
        LC_ALL: "C",
        PERF_CONFIG: "/dev/null",
        DEBUGINFOD_URLS: "",
      },
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
      ...(supervisor ? {} : { timeout: timeoutMs }),
    },
  );
}

/** Profile only the prepared executable, its threads and inherited descendants. */
export function captureSourceOperationPerf(request: SourceOperationPerfRequest) {
  const { command, mode } = request;
  const group = request.eventGroup ?? "core";
  const events = [...(group === "core" ? coreEvents : cacheEvents), ...softwareEvents];
  const out = resolve(request.outputDir);
  if (existsSync(out)) throw new Error(`Refusing to overwrite evidence: ${out}`);
  mkdirSync(out, { recursive: true });
  const binary = resolve(command.executable);
  const digest = () => createHash("sha256").update(readFileSync(binary)).digest("hex");
  const binarySha256 = digest();
  const environment = sourcePerfEnvironment();
  const invocation = { ...command, executable: binary };
  const rawPath = join(out, mode === "stat" ? "perf-stat.csv" : "perf.data");
  // --no-buildid skips record's expensive symbolizing post-pass. The exact
  // executable must remain unchanged until the immediate source report ends.
  const perfArgs =
    mode === "stat"
      ? ["stat", "-x,", "--output", rawPath, "-e", events.join(",")]
      : [
          "record",
          "--no-buildid",
          "--no-buildid-cache",
          "--sample-cpu",
          "--event",
          "cpu-clock:u",
          "--freq",
          "99",
          "--call-graph",
          "dwarf,4096",
          "--output",
          rawPath,
        ];
  const version = runBounded(request.perfExecutable, ["--version"], 5000, command);
  const startedAt = new Date().toISOString();
  const result = runBounded(
    request.perfExecutable,
    [...perfArgs, "--", binary, ...command.args],
    request.timeoutMs ?? 300_000,
    command,
  );
  const captureFinishedAt = new Date().toISOString();
  writeFileSync(join(out, "stdout.txt"), result.stdout ?? "");
  writeFileSync(join(out, "stderr.txt"), result.stderr ?? "");
  const counters = parsePerfStat(mode === "stat" ? (optionalText(rawPath) ?? "") : "");
  const missingEvents =
    mode === "stat"
      ? events.filter((event) => !counters.some((row) => row.event === event && row.available))
      : [];
  const sanity = counterSanity(counters, group);
  const stderr = result.stderr ?? "";
  const recordSampleText = /\((\d+) samples\)/u.exec(stderr)?.[1];
  const recordSampleCount = recordSampleText === undefined ? null : Number(recordSampleText);
  const dataBytes = existsSync(rawPath) ? statSync(rawPath).size : 0;
  let sampleCount: number | null = null;
  let eventCensus: null | { status: number | null; signal: string | null; error: string | null } =
    null;
  if (
    mode === "record" &&
    dataBytes > 0 &&
    !result.error &&
    !result.signal &&
    result.status === 0
  ) {
    // The count-only handlers of report --stats do not resolve symbols or unwind DWARF.
    const counted = runBounded(
      request.perfExecutable,
      ["report", "--stdio", "--stats", "--input", rawPath],
      30_000,
      command,
    );
    writeFileSync(join(out, "event-stats.txt"), counted.stdout ?? "");
    writeFileSync(join(out, "event-stats.stderr.txt"), counted.stderr ?? "");
    if (counted.status === 0) sampleCount = parsePerfEventStats(counted.stdout ?? "");
    eventCensus = {
      status: counted.status,
      signal: counted.signal,
      error: counted.error?.message ?? null,
    };
  }
  const evidencePresent =
    mode === "stat" ? missingEvents.length === 0 : dataBytes > 0 && result.status === 0;
  const capabilityFailure = fatalPerfCapability(stderr) && !evidencePresent;
  const collection =
    result.error ||
    result.signal ||
    [124, 125, 126, 127, 137].includes(result.status ?? -1) ||
    (!evidencePresent && result.status !== 0 && !capabilityFailure)
      ? "failed"
      : capabilityFailure || (mode === "stat" && (missingEvents.length > 0 || !sanity.plausible))
        ? "unavailable"
        : mode === "record" && !dataBytes
          ? "unavailable"
          : "collected";
  let report: null | {
    status: number | null;
    signal: string | null;
    error: string | null;
    sourceRows: number;
    qualified: boolean;
  } = null;
  if (mode === "record" && collection === "collected" && (sampleCount ?? 0) > 0) {
    const rendered = runBounded(
      request.perfExecutable,
      [
        "report",
        "--stdio",
        "--no-children",
        "--no-inline",
        "--sort",
        "dso,symbol,srcline",
        "--percent-limit",
        "0.1",
        ...(request.addr2lineExecutable ? ["--addr2line", request.addr2lineExecutable] : []),
        "--input",
        rawPath,
      ],
      request.reportTimeoutMs ?? 120_000,
      command,
    );
    writeFileSync(join(out, "source-report.txt"), rendered.stdout ?? "");
    writeFileSync(join(out, "source-report.stderr.txt"), rendered.stderr ?? "");
    const sourceRows = [...(rendered.stdout ?? "").matchAll(/\.(?:rs|go|c|cpp|cc|h):[1-9]\d*/gu)]
      .length;
    report = {
      status: rendered.status,
      signal: rendered.signal,
      error: rendered.error?.message ?? null,
      sourceRows,
      qualified:
        rendered.status === 0 &&
        !rendered.error &&
        !symbolizationErrors(rendered.stderr ?? "") &&
        sourceRows > 0,
    };
  }
  const binarySha256After = digest();
  const calibration = hardwareCalibration(request.calibrationPath, environment.fingerprint, group);
  const schedulingQualified = counters
    .filter((row) => (group === "core" ? coreEvents : cacheEvents).includes(row.event))
    .every((row) => row.runningPercentage !== null && row.runningPercentage >= 99.9);
  const qualified =
    collection === "collected" &&
    binarySha256After === binarySha256 &&
    (mode === "stat"
      ? calibration.qualified && schedulingQualified
      : report?.qualified === true && !symbolizationErrors(stderr));
  const receipt = {
    schemaVersion: 2,
    mode,
    metric:
      mode === "stat"
        ? "hardware-and-software-performance-counters"
        : "software-user-cpu-clock-samples",
    scope:
      "perf child process, its threads and inherited descendants; setup, supervisor and report rendering excluded",
    simulation: false,
    eventGroup: mode === "stat" ? group : null,
    qualified,
    samplingHz: mode === "record" ? 99 : null,
    dwarfStackBytes: mode === "record" ? 4096 : null,
    addr2lineExecutable: request.addr2lineExecutable ?? null,
    buildIdPolicy:
      mode === "record"
        ? "post-pass disabled; exact executable paths must be retained for later replay"
        : null,
    collection,
    counters,
    missingEvents,
    counterSanity: sanity,
    schedulingQualified,
    calibration,
    sampleCount,
    recordSampleCount,
    eventCensus,
    dataBytes,
    startedAt,
    captureFinishedAt,
    finishedAt: new Date().toISOString(),
    identity: request.identity,
    command: invocation,
    binarySha256,
    binarySha256After,
    perf: {
      executable: request.perfExecutable,
      version: { status: version.status, stdout: version.stdout, stderr: version.stderr },
      locale: "C",
      config: "/dev/null",
      args: [...perfArgs, "--", binary, ...command.args],
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      timeoutMs: request.timeoutMs ?? 300_000,
      reportTimeoutMs: request.reportTimeoutMs ?? 120_000,
    },
    commandExit: {
      status: collection === "collected" ? result.status : null,
      evidence:
        collection === "collected"
          ? "perf propagated status; stdout oracle must also pass"
          : "unknown; collection did not qualify",
    },
    capabilities: {
      ...environment,
      paranoid: optionalText("/proc/sys/kernel/perf_event_paranoid"),
      kptrRestrict: optionalText("/proc/sys/kernel/kptr_restrict"),
      effectiveUid: process.geteuid?.() ?? null,
    },
    report,
  };
  writeFileSync(join(out, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
  return receipt;
}
