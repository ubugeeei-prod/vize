import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";

export interface SourceOperationCommand {
  executable: string;
  args: string[];
  cwd: string;
  /** Explicit, non-secret measurement variables; inherited environment is not exported. */
  env: Record<string, string>;
}

export interface SourceOperationPerfRequest {
  mode: "stat" | "record";
  perfExecutable: string;
  outputDir: string;
  command: SourceOperationCommand;
  identity: Record<string, unknown>;
  /** Prefer llvm-addr2line for Rust DWARF; its identity is retained. */
  addr2lineExecutable?: string;
  /** Timeout of the measured command, distinct from the bounded report timeout. */
  timeoutMs?: number;
}

const hardwareEvents = [
  "instructions",
  "cycles",
  "branches",
  "branch-misses",
  "cache-references",
  "cache-misses",
];
const softwareEvents = ["task-clock", "context-switches", "cpu-migrations", "page-faults"];

function optionalText(path: string) {
  try {
    return readFileSync(path, "utf8");
  } catch {
    return null;
  }
}

/** Parse counters without treating unsupported/not-counted or multiplexing as zero. */
export function parsePerfStat(text: string) {
  return text.split("\n").flatMap((line) => {
    const fields = line.split(",");
    const event = fields[2]?.trim();
    if (!event || ![...hardwareEvents, ...softwareEvents].includes(event)) return [];
    const raw = fields[0]?.trim();
    const value = raw && /^\d+(?:\.\d+)?$/u.test(raw) ? Number(raw) : null;
    return [
      {
        event,
        value,
        raw,
        unit: fields[1]?.trim(),
        runtime: fields[3]?.trim(),
        runningPercent: fields[4]?.trim(),
        available: value !== null && Number.isFinite(value),
      },
    ];
  });
}

/**
 * Profile an already prepared command. perf's default inheritance includes its
 * measured child, subsequently created threads and descendants. No shell,
 * fixture generator, installer, or Node driver is inside the measured window.
 * stat hardware counts and record software samples are separate fresh processes.
 */
export function captureSourceOperationPerf(request: SourceOperationPerfRequest) {
  const { command, mode } = request;
  const out = resolve(request.outputDir);
  if (existsSync(out)) throw new Error(`Refusing to overwrite evidence: ${out}`);
  mkdirSync(out, { recursive: true });
  const binary = resolve(command.executable);
  const binarySha256 = createHash("sha256").update(readFileSync(binary)).digest("hex");
  const invocation = { ...command, executable: binary };
  const rawPath = join(out, mode === "stat" ? "perf-stat.csv" : "perf.data");
  const perfArgs =
    mode === "stat"
      ? ["stat", "-x,", "--output", rawPath, "-e", [...hardwareEvents, ...softwareEvents].join(",")]
      : [
          "record",
          "--event",
          "cpu-clock",
          "--freq",
          "99",
          "--call-graph",
          "dwarf,4096",
          "--output",
          rawPath,
        ];
  // No wrapper changes the measured descendant scope. The receipt keeps perf
  // status distinct from an unknown child status when collection is unavailable.
  const startedAt = new Date().toISOString();
  const result = spawnSync(request.perfExecutable, [...perfArgs, "--", binary, ...command.args], {
    cwd: command.cwd,
    env: { ...process.env, ...command.env, LC_ALL: "C" },
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    timeout: request.timeoutMs ?? 300_000,
  });
  writeFileSync(join(out, "stdout.txt"), result.stdout ?? "");
  writeFileSync(join(out, "stderr.txt"), result.stderr ?? "");
  const raw = mode === "stat" ? (optionalText(rawPath) ?? "") : "";
  const counters = parsePerfStat(raw);
  const missingEvents =
    mode === "stat"
      ? [...hardwareEvents, ...softwareEvents].filter(
          (name) => !counters.some((row) => row.event === name && row.available),
        )
      : [];
  const stderr = result.stderr ?? "";
  const capabilityFailure =
    /permission|not permitted|not supported|no permission|access to performance|unknown tracepoint|failed to open|No such file/iu.test(
      `${stderr}\n${raw}`,
    );
  const collection =
    result.error || result.signal
      ? "failed"
      : capabilityFailure || (mode === "stat" && missingEvents.length > 0)
        ? "unavailable"
        : mode === "record" && !existsSync(rawPath)
          ? "failed"
          : "collected";
  let report: null | { status: number | null; signal: string | null; error: string | null } = null;
  if (mode === "record" && collection === "collected") {
    const rendered = spawnSync(
      request.perfExecutable,
      [
        "report",
        "--stdio",
        "--no-children",
        "--sort",
        "dso,symbol,srcline",
        "--percent-limit",
        "0.01",
        ...(request.addr2lineExecutable ? ["--addr2line", request.addr2lineExecutable] : []),
        "--input",
        rawPath,
      ],
      {
        encoding: "utf8",
        timeout: 120_000,
        maxBuffer: 32 * 1024 * 1024,
        env: { ...process.env, ...command.env, LC_ALL: "C" },
        cwd: command.cwd,
      },
    );
    writeFileSync(join(out, "source-report.txt"), rendered.stdout ?? "");
    writeFileSync(join(out, "source-report.stderr.txt"), rendered.stderr ?? "");
    report = {
      status: rendered.status,
      signal: rendered.signal,
      error: rendered.error?.message ?? null,
    };
  }
  const receipt = {
    schemaVersion: 1,
    mode,
    metric:
      mode === "stat" ? "hardware-and-software-performance-counters" : "software-cpu-clock-samples",
    scope:
      "perf child process, its threads and inherited descendants; setup and report rendering excluded",
    simulation: false,
    samplingHz: mode === "record" ? 99 : null,
    addr2lineExecutable: request.addr2lineExecutable ?? null,
    dwarfStackBytes: mode === "record" ? 4096 : null,
    collection,
    counters,
    missingEvents,
    startedAt,
    finishedAt: new Date().toISOString(),
    identity: request.identity,
    command: invocation,
    binarySha256,
    perf: {
      executable: request.perfExecutable,
      locale: "C",
      args: [...perfArgs, "--", binary, ...command.args],
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
    },
    // perf can propagate a child status or fail before exec; those cases cannot
    // be independently distinguished without changing the measured command.
    commandExit: {
      status: collection === "collected" ? result.status : null,
      evidence:
        collection === "collected"
          ? "perf propagated status; stdout oracle must also pass"
          : "unknown; perf may not have executed command",
    },
    capabilities: {
      platform: process.platform,
      architecture: process.arch,
      paranoid: optionalText("/proc/sys/kernel/perf_event_paranoid"),
      kptrRestrict: optionalText("/proc/sys/kernel/kptr_restrict"),
      cpuPmuType: optionalText("/sys/bus/event_source/devices/cpu/type"),
      effectiveUid: process.geteuid?.() ?? null,
    },
    report,
  };
  writeFileSync(join(out, "receipt.json"), `${JSON.stringify(receipt, null, 2)}\n`);
  return receipt;
}
