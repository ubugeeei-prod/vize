import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
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

export async function check(directory: string, args: string[] = []): Promise<Diagnostic[]> {
  const launch = resolveVizeLaunchCommand();
  const child = spawn(launch[0], [...launch.slice(1, -1), "check", ...args, "--format", "json"], {
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
    const status = await new Promise<number | null>((resolve, reject) => {
      child.on("error", reject);
      child.on("close", resolve);
    });
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
