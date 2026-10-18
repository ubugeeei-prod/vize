import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { inspect } from "node:util";
import { root, testOutputRoot } from "../lsp/paths.ts";
import { resolveVizeLaunchCommand } from "../lsp/launch.ts";

export type Diagnostic = {
  file: string;
  line: number;
  column: number;
  code?: number;
  severity: "error" | "warning";
  message: string;
};
export const fixtureRoot = path.join(root, "tests/_fixtures/vue-language-tools");
export const patternRoot = path.join(root, "tests/_fixtures/pattern-reference");

export function workspace(prefix: string): string {
  fs.mkdirSync(testOutputRoot, { recursive: true });
  const directory = fs.mkdtempSync(path.join(testOutputRoot, prefix));
  fs.symlinkSync(
    path.join(root, "tests/node_modules"),
    path.join(directory, "node_modules"),
    process.platform === "win32" ? "junction" : "dir",
  );
  return directory;
}

export async function check(
  directory: string,
  args: string[] = [],
  capture?: { name: string; sources: string[] },
): Promise<Diagnostic[]> {
  const launch = resolveVizeLaunchCommand();
  const argv = [...launch.slice(1, -1), "check", ...args, "--format", "json"];
  const child = spawn(launch[0], argv, {
    cwd: directory,
    env: process.env,
    stdio: ["ignore", "pipe", "pipe"],
  });
  let stdout = "";
  let stderr = "";
  child.stdout.setEncoding("utf8").on("data", (chunk) => (stdout += chunk));
  child.stderr.setEncoding("utf8").on("data", (chunk) => (stderr += chunk));
  const timeout = setTimeout(() => child.kill("SIGKILL"), 60_000);
  try {
    let status: number | null = null;
    let signal: NodeJS.Signals | null = null;
    let closeObserved = false;
    let error: Error | undefined;
    try {
      status = await new Promise<number | null>((resolve, reject) => {
        child.on("error", (original) => {
          error = original;
          reject(original);
        });
        child.on("close", (code, receivedSignal) => {
          closeObserved = true;
          signal = receivedSignal;
          resolve(code);
        });
      });
    } finally {
      if (capture) {
        const output = path.join(
          root,
          "target/differential/fallthrough-listener-inputs",
          capture.name,
        );
        fs.mkdirSync(output, { recursive: true });
        fs.writeFileSync(path.join(output, "stdout.json"), stdout);
        fs.writeFileSync(path.join(output, "stderr.txt"), stderr);
        fs.writeFileSync(
          path.join(output, "process.json"),
          JSON.stringify({
            executable: launch[0],
            argv,
            cwd: directory,
            pid: child.pid ?? null,
            status,
            signal,
            closeObserved,
            sourceRevision: process.env.GITHUB_SHA,
            error: error
              ? {
                  name: error.name,
                  message: error.message,
                  stack: error.stack,
                  ownProperties: Object.getOwnPropertyNames(error),
                  raw: inspect(error, { showHidden: true, depth: null, customInspect: false }),
                }
              : null,
          }),
        );
        for (const file of capture.sources)
          fs.copyFileSync(path.join(directory, file), path.join(output, file));
      }
    }
    assert.ok(status === 0 || status === 1, `checker failed (${status}): ${stderr}\n${stdout}`);
    assert.ok(stdout.trim().startsWith("{"), `checker did not return JSON: ${stderr}\n${stdout}`);
    const result = JSON.parse(stdout) as {
      files: Array<{ file: string; diagnostics: string[] }>;
      errorCount: number;
    };
    const diagnostics = result.files.flatMap(({ file, diagnostics }) =>
      diagnostics.map((text) => {
        const match = /^(error|warning):(\d+):(\d+) (?:\[TS(\d+)\] )?([\s\S]+)$/.exec(text);
        assert.ok(match, `unrecognized diagnostic: ${text}`);
        return {
          file: file.replaceAll("\\", "/"),
          severity: match[1] as "error" | "warning",
          line: Number(match[2]),
          column: Number(match[3]),
          ...(match[4] ? { code: Number(match[4]) } : {}),
          message: match[5],
        };
      }),
    );
    assert.equal(result.errorCount, diagnostics.filter((d) => d.severity === "error").length);
    assert.equal(
      status === 0,
      result.errorCount === 0,
      "exit code and diagnostic verdict disagree",
    );
    return diagnostics;
  } finally {
    clearTimeout(timeout);
  }
}

export function diagnosticIdentity(diagnostic: Diagnostic): object {
  const { file, line, column, code } = diagnostic;
  return { file, line, column, code };
}

export function compareIdentity(left: object, right: object): number {
  return JSON.stringify(left).localeCompare(JSON.stringify(right));
}
