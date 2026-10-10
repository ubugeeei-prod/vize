import { spawn } from "node:child_process";
import type { OxlintProcessResult } from "./scoped-selection.ts";
import { presentationContext } from "./presentation-context.ts";

/** Retain the actual child packet, including buffered bytes on stream/spawn failure. */
export function runOxlint(
  executable: string,
  args: readonly string[],
  cwd: string,
): Promise<OxlintProcessResult> {
  return new Promise((resolve) => {
    const child = spawn(executable, args, { cwd, stdio: ["ignore", "pipe", "pipe"] });
    const stdout: Buffer[] = [],
      stderr: Buffer[] = [];
    let error: string | null = null;
    const failure = (reason: Error) => {
      error ??= reason.message;
    };
    child.stdout.on("data", (chunk: Buffer | string) =>
      stdout.push(typeof chunk === "string" ? Buffer.from(chunk) : chunk),
    );
    child.stderr.on("data", (chunk: Buffer | string) =>
      stderr.push(typeof chunk === "string" ? Buffer.from(chunk) : chunk),
    );
    child.stdout.on("error", failure);
    child.stderr.on("error", failure);
    child.on("error", failure);
    child.on("close", (status, signal) => {
      const stdoutBytes = Buffer.concat(stdout),
        stderrBytes = Buffer.concat(stderr);
      const failureBytes = error
        ? Buffer.from(`\nOxlint child failure: ${error}\n`)
        : Buffer.alloc(0);
      const reportedStderr = Buffer.concat([stderrBytes, failureBytes]);
      resolve({
        status,
        stdout: stdoutBytes.toString("utf8"),
        rawStdout: stdoutBytes,
        rawStderr: reportedStderr,
        stderr: reportedStderr.toString("utf8"),
        observation: {
          executable,
          args: [...args],
          cwd,
          status,
          signal,
          error,
          stdoutBytes,
          stderrBytes,
          environment: presentationContext(),
        },
      });
    });
  });
}
