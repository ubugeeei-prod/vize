import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, realpathSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, relative, resolve, sep } from "node:path";
import { performance } from "node:perf_hooks";
import { stripVTControlCharacters } from "node:util";

let clockTicksPerSecond;

/**
 * User+sys CPU of every reaped descendant (cutime + cstime of this process),
 * in ms. spawnSync reaps only its own child, and a child's total includes the
 * mapper or checker processes it waited for, so the delta around one
 * invocation is that invocation's whole process tree. Linux only; elsewhere
 * no CPU figure is recorded.
 */
function reapedChildCpuMs() {
  if (process.platform !== "linux") return null;
  clockTicksPerSecond ??= Number(execFileSync("getconf", ["CLK_TCK"], { encoding: "utf8" }));
  const stat = readFileSync("/proc/self/stat", "utf8");
  const fields = stat.slice(stat.lastIndexOf(")") + 2).split(" ");
  const ticks = Number(fields[13]) + Number(fields[14]);
  if (!Number.isFinite(ticks) || !(clockTicksPerSecond > 0)) {
    throw new Error("type-check runner: cannot read reaped child CPU time from /proc/self/stat");
  }
  return (ticks * 1000) / clockTicksPerSecond;
}

const GNU_TIME = "/usr/bin/time";
let rssSamples = 0;

/**
 * On Linux every invocation runs under GNU time for its peak RSS: `%M` is the
 * largest single process of the tree (KiB), not the sum of the tree.
 */
function withPeakRss(binary, args) {
  if (process.platform !== "linux") return { command: binary, argv: args, rssFile: null };
  if (!existsSync(GNU_TIME)) {
    throw new Error(`type-check runner: ${GNU_TIME} (GNU time) is required for peak RSS`);
  }
  if (!existsSync(binary)) {
    throw Object.assign(new Error(`spawnSync ${binary} ENOENT`), { code: "ENOENT" });
  }
  const rssFile = join(tmpdir(), `vize-typecheck-rss-${process.pid}-${rssSamples++}`);
  return { command: GNU_TIME, argv: ["-f", "%M", "-o", rssFile, binary, ...args], rssFile };
}

function readPeakRssKb(rssFile) {
  if (rssFile == null) return null;
  const lines = readFileSync(rssFile, "utf8").trim().split("\n");
  rmSync(rssFile, { force: true });
  const kb = Number(lines.at(-1));
  if (!Number.isInteger(kb) || kb <= 0) {
    throw new Error(`type-check runner: GNU time reported no peak RSS: ${lines.join(" / ")}`);
  }
  return kb;
}

export function runTypecheckCommand(binary, args, { cwd, env = {} }) {
  const { command, argv, rssFile } = withPeakRss(binary, args);
  const cpuBefore = reapedChildCpuMs();
  const start = performance.now();
  const result = spawnSync(command, argv, {
    cwd,
    env: { ...process.env, NO_COLOR: "1", VIZE_BENCH: "1", ...env },
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    timeout: 300_000,
  });
  const ms = performance.now() - start;
  const cpuAfter = reapedChildCpuMs();
  if (result.error) throw result.error;
  return {
    ms,
    cpuMs: cpuBefore == null ? null : cpuAfter - cpuBefore,
    rssKb: readPeakRssKb(rssFile),
    status: result.status,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}

function sourcePath(file, cwd) {
  const base = realpathSync(cwd);
  const absolute = resolve(base, file);
  return relative(base, existsSync(absolute) ? realpathSync(absolute) : absolute)
    .split(sep)
    .join("/");
}

export function normalizeTypecheckResult(result, cwd, format) {
  const output = stripVTControlCharacters(`${result.stdout}\n${result.stderr}`);
  // tsc can report DiagnosticsPresent_OutputsGenerated (2), including under --noEmit.
  if (!(format === "json" ? [0, 1] : [0, 1, 2]).includes(result.status)) {
    throw new Error(`type-check process exited with ${result.status}\n${output}`);
  }
  const diagnostics = [];
  let summaryErrorCount;
  if (format === "json") {
    if (result.stderr.trim()) throw new Error(`unexpected type-check stderr\n${result.stderr}`);
    const report = JSON.parse(result.stdout);
    if (!Array.isArray(report.files)) throw new Error("type-check report has no files");
    for (const entry of report.files) {
      for (const diagnostic of entry.diagnostics) {
        const match = /^(error|warning):(\d+):(\d+) \[([^\]]+)\] (.*)$/su.exec(diagnostic);
        if (!match) throw new Error(`invalid type-check diagnostic: ${diagnostic}`);
        diagnostics.push([sourcePath(entry.file, cwd), ...match.slice(1)]);
      }
    }
    for (const kind of ["error", "warning"]) {
      if (report[`${kind}Count`] !== diagnostics.filter((d) => d[1] === kind).length) {
        throw new Error(`type-check ${kind} count does not match its diagnostics`);
      }
    }
  } else {
    let continuation = false;
    for (const line of output.split(/\r?\n/u)) {
      const match = /^(.+?)\((\d+),(\d+)\): (error|warning) (TS\d+): (.*)$/u.exec(line);
      if (match) {
        const [, file, line, column, kind, code, message] = match;
        diagnostics.push([sourcePath(file, cwd), kind, line, column, code, message]);
        continuation = true;
      } else if (
        continuation &&
        /^[ \t]+\S/u.test(line) &&
        !/^\s+(?:\w*Error(?:\s*\[[^\]]+\])?:|at\s|panic:|fatal error:)/u.test(line)
      ) {
        diagnostics.at(-1)[5] += `\n${line.trimEnd()}`;
      } else if (/^Found \d+ error\(s\) in \d+ file\(s\)\.$/u.test(line)) {
        summaryErrorCount = Number(/^Found (\d+)/u.exec(line)[1]);
        continuation = false;
      } else if (
        !line.trim() ||
        line === "Using config from ./golar.config.ts..." ||
        /^verter-tsc: checking \d+ \.vue file\(s\)\.\.\.$/u.test(line)
      ) {
        continuation = false;
      } else {
        throw new Error(`unexpected type-check output\n${line}`);
      }
    }
  }
  const errors = diagnostics.filter((d) => d[1] === "error").length;
  if (summaryErrorCount != null && summaryErrorCount !== errors) {
    throw new Error("type-check summary count does not match its diagnostics");
  }
  if ((result.status !== 0 && errors === 0) || (result.status === 0 && errors > 0)) {
    throw new Error(`type-check exit status disagrees with diagnostics\n${output}`);
  }
  diagnostics.sort((a, b) => JSON.stringify(a).localeCompare(JSON.stringify(b), "en"));
  return { status: result.status, diagnostics };
}

export function diagnosticFingerprint(report) {
  return createHash("sha256").update(JSON.stringify(report)).digest("hex");
}
