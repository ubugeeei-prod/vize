import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { cpus, release } from "node:os";

export const coreEvents = ["instructions", "cycles", "branches", "branch-misses"];
export const cacheEvents = ["cache-references", "cache-misses"];
export const softwareEvents = ["task-clock", "context-switches", "cpu-migrations", "page-faults"];

export function optionalText(path: string) {
  try {
    return readFileSync(path, "utf8");
  } catch {
    return null;
  }
}

/** Initial inherited affinity, not a claim that a child never changes affinity. */
export function sourcePerfEnvironment() {
  const cpuInfo = optionalText("/proc/cpuinfo");
  const environment = {
    platform: process.platform,
    architecture: process.arch,
    cpuModel: cpus()[0]?.model ?? null,
    microcode: /^microcode\s*:\s*(.+)$/m.exec(cpuInfo ?? "")?.[1] ?? null,
    kernel: release(),
    initialAffinity:
      /^Cpus_allowed_list:\s*(.+)$/m.exec(optionalText("/proc/self/status") ?? "")?.[1] ?? null,
    cpuPmuType: optionalText("/sys/bus/event_source/devices/cpu/type")?.trim() ?? null,
  };
  return {
    ...environment,
    fingerprint: createHash("sha256").update(JSON.stringify(environment)).digest("hex"),
  };
}

/** perf's CSV values are already scaled; do not multiply by running percentage. */
export function parsePerfStat(text: string) {
  return text.split("\n").flatMap((line) => {
    const fields = line.split(",");
    const event = fields[2]?.trim();
    if (!event || ![...coreEvents, ...cacheEvents, ...softwareEvents].includes(event)) return [];
    const raw = fields[0]?.trim();
    const value = raw && /^\d+(?:\.\d+)?$/u.test(raw) ? Number(raw) : null;
    const available = value !== null && Number.isFinite(value);
    const running = Number.parseFloat(fields[4] ?? "");
    const runningPercentage = Number.isFinite(running) ? running : null;
    return [
      {
        event,
        value,
        raw,
        unit: fields[1]?.trim(),
        runtime: fields[3]?.trim(),
        runningPercent: fields[4]?.trim(),
        runningPercentage,
        available,
        interpretation: !available
          ? "unavailable"
          : runningPercentage === null
            ? "scheduling unknown"
            : runningPercentage < 99.9
              ? "scaled estimate from multiplexed event"
              : "counter at full scheduling",
      },
    ];
  });
}

export function counterSanity(counters: ReturnType<typeof parsePerfStat>, group: "core" | "cache") {
  const value = (event: string) => counters.find((row) => row.event === event)?.value;
  const reasons: string[] = [];
  if (group === "core") {
    if (value("instructions") === 0)
      reasons.push("zero retired instructions for an executed process");
    if ((value("instructions") ?? 0) > 0 && value("cycles") === 0)
      reasons.push("positive instructions with zero cycles");
    if ((value("instructions") ?? 0) > 1_000_000 && value("branches") === 0)
      reasons.push("large instruction count with zero branches");
  }
  return { plausible: reasons.length === 0, reasons };
}

export function hardwareCalibration(
  path: string | undefined,
  environmentFingerprint: string,
  group: "core" | "cache",
) {
  const reasons: string[] = [];
  let sha256: string | null = null;
  if (!path) reasons.push("no PMU calibration supplied");
  else {
    try {
      const bytes = readFileSync(path);
      sha256 = createHash("sha256").update(bytes).digest("hex");
      const calibration = JSON.parse(bytes.toString("utf8"));
      if (calibration.schemaVersion !== 1) reasons.push("unsupported calibration schema");
      if (calibration.environmentFingerprint !== environmentFingerprint)
        reasons.push("calibration CPU/kernel/affinity identity mismatch");
      if (calibration.groups?.[group]?.qualified !== true)
        reasons.push("PMU calibration group did not qualify");
    } catch (error) {
      reasons.push("invalid PMU calibration: " + String(error));
    }
  }
  return { qualified: reasons.length === 0, reasons, path: path ?? null, sha256 };
}

/** Kernel-symbol warning prose mentioning permission is not an event-open failure. */
export function fatalPerfCapability(stderr: string) {
  return (
    /^(?:Error:\s*(?:No permission|Access to|Permission)|No permission to enable|perf_event_open.*(?:failed|denied)|Access to performance monitoring|failed to (?:open|mmap)|Failed to (?:open|mmap)|Permission error)/mu.test(
      stderr,
    ) ||
    /(?:not permitted|not supported|unknown tracepoint|No such file or directory)/iu.test(stderr)
  );
}

export function symbolizationErrors(stderr: string) {
  return /addr2line.*(?:could not read|unexpected record)|DWARF.*(?:error|failed)|failed to find (?:a )?DIE/iu.test(
    stderr,
  );
}

/** --no-buildid deliberately omits record's post-pass sample-count message. */
export function parsePerfEventStats(text: string) {
  const raw = /^\s*(?:PERF_RECORD_)?SAMPLE(?:\s+events)?\s*:\s*(\d+)(?:\s|$)/m.exec(text)?.[1];
  const value = raw === undefined ? null : Number(raw);
  return value !== null && Number.isSafeInteger(value) ? value : null;
}
