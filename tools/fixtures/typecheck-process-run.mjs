import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

export function runMeasured(command, args, options) {
  const time = "/usr/bin/time";
  if (!existsSync(time) || !["darwin", "linux"].includes(process.platform)) {
    throw new Error(`Peak RSS measurement is unsupported on ${process.platform}`);
  }
  const metricsDir = mkdtempSync(join(tmpdir(), "vize-typecheck-rss-"));
  const metricsPath = join(metricsDir, "time.txt");
  const timeArgs =
    process.platform === "darwin"
      ? ["-l", "-o", metricsPath, command, ...args]
      : ["-v", "-o", metricsPath, command, ...args];
  const startedAt = Date.now();
  try {
    const result = spawnSync(time, timeArgs, options);
    const metrics = readFileSync(metricsPath, "utf8");
    return {
      ...result,
      durationMs: Date.now() - startedAt,
      peakRssBytes: parsePeakRss(metrics, process.platform),
    };
  } finally {
    rmSync(metricsDir, { recursive: true, force: true });
  }
}

export function parsePeakRss(metrics, platform) {
  if (!["darwin", "linux"].includes(platform)) {
    throw new Error(`Peak RSS measurement is unsupported on ${platform}`);
  }
  const match =
    platform === "darwin"
      ? /^\s*(\d+)\s+maximum resident set size$/mu.exec(metrics)
      : /^\s*Maximum resident set size \(kbytes\):\s*(\d+)$/mu.exec(metrics);
  if (match == null) throw new Error("Peak RSS measurement output is invalid");
  const value = Number(match[1]);
  const bytes = platform === "linux" ? value * 1024 : value;
  if (!Number.isSafeInteger(bytes) || bytes <= 0) {
    throw new Error("Peak RSS measurement is not a positive safe integer");
  }
  return bytes;
}
