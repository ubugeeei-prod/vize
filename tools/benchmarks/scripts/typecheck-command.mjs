import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, realpathSync } from "node:fs";
import { relative, resolve, sep } from "node:path";
import { performance } from "node:perf_hooks";
import { stripVTControlCharacters } from "node:util";

export function runTypecheckCommand(binary, args, { cwd, env = {} }) {
  const start = performance.now();
  const result = spawnSync(binary, args, {
    cwd,
    env: { ...process.env, NO_COLOR: "1", VIZE_BENCH: "1", ...env },
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    timeout: 300_000,
  });
  const ms = performance.now() - start;
  if (result.error) throw result.error;
  return { ms, status: result.status, stdout: result.stdout ?? "", stderr: result.stderr ?? "" };
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
  if (
    /^(?:\w*Error \[ERR_[A-Z_]+\]|panic:|fatal error:|Segmentation fault|Trace\/BPT trap)/mu.test(
      output,
    )
  ) {
    throw new Error(`type-check process failed\n${output}`);
  }
  const diagnostics = [];
  if (format === "json") {
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
    for (const line of output.split(/\r?\n/u)) {
      const match = /^(.+?)\((\d+),(\d+)\): (error|warning) (TS\d+): (.*)$/u.exec(line);
      if (match) {
        const [, file, line, column, kind, code, message] = match;
        diagnostics.push([sourcePath(file, cwd), kind, line, column, code, message]);
      } else if (/^[ \t]+\S/u.test(line) && diagnostics.length > 0) {
        diagnostics.at(-1)[5] += `\n${line.trimEnd()}`;
      }
    }
  }
  const errors = diagnostics.filter((d) => d[1] === "error").length;
  if ((result.status !== 0 && errors === 0) || (result.status === 0 && errors > 0)) {
    throw new Error(`type-check exit status disagrees with diagnostics\n${output}`);
  }
  diagnostics.sort((a, b) => JSON.stringify(a).localeCompare(JSON.stringify(b), "en"));
  return { status: result.status, diagnostics };
}

export function diagnosticFingerprint(report) {
  return createHash("sha256").update(JSON.stringify(report)).digest("hex");
}
