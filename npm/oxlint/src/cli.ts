import { spawn } from "node:child_process";
import type { Writable } from "node:stream";

import { expectsLintReport, getLintTargets } from "./cli/args.js";
import { collectVueLikeFilesFromTargets } from "./cli/files.js";
import { resolveOxlintCliEntrypoint, verifyOxlintCliEntrypoint } from "./cli/oxlint.js";
import { rewriteReportedPaths } from "./cli/output.js";
import { prepareScriptlessWorkaroundFiles } from "./cli/workaround-files.js";

async function main(): Promise<void> {
  const cwd = process.cwd();
  const forwardedArgs = process.argv.slice(2);
  const targets = getLintTargets(forwardedArgs);
  const lintFiles = collectVueLikeFilesFromTargets(cwd, targets);
  const oxlintEntrypoint = resolveOxlintCliEntrypoint(cwd);
  verifyOxlintCliEntrypoint(process.execPath, oxlintEntrypoint);
  const prepared = prepareScriptlessWorkaroundFiles(cwd, lintFiles);
  const args = [oxlintEntrypoint, ...forwardedArgs, ...prepared.appendedArgs];

  try {
    const result = await runOxlint(process.execPath, args, cwd);
    const stdout = rewriteReportedPaths(result.stdout, prepared.pathReplacements);
    const stderr = rewriteReportedPaths(result.stderr, prepared.pathReplacements);

    if (stdout) {
      await writeStream(process.stdout, stdout);
    }

    if (stderr) {
      await writeStream(process.stderr, stderr);
    }

    if (result.status === 0 && stdout === "" && stderr === "" && expectsLintReport(forwardedArgs)) {
      await writeStream(
        process.stderr,
        `The oxlint run at ${oxlintEntrypoint} exited 0 but produced no report, ` +
          "although the requested format always emits one. " +
          "Refusing to treat the silent run as a clean lint result.\n",
      );
      process.exitCode = 1;
      return;
    }

    if (
      prepared.usedScriptlessWorkaround &&
      (forwardedArgs.includes("--fix") || forwardedArgs.includes("--fix-suggestions"))
    ) {
      await writeStream(
        process.stderr,
        "\n[oxlint-plugin-vize] Temporary Vue workaround is active; fixes are not applied back to original files yet.\n",
      );
    }

    process.exitCode = result.status ?? 1;
  } finally {
    prepared.cleanup();
  }
}

main().catch((error: unknown) => {
  process.stderr.write(
    `${error instanceof Error ? (error.stack ?? error.message) : String(error)}\n`,
  );
  process.exitCode = 1;
});

function runOxlint(
  executable: string,
  args: readonly string[],
  cwd: string,
): Promise<{ status: number | null; stderr: string; stdout: string }> {
  return new Promise((resolve, reject) => {
    const child = spawn(executable, args, {
      cwd,
      stdio: ["ignore", "pipe", "pipe"],
    });
    const stdoutChunks: Buffer[] = [];
    const stderrChunks: Buffer[] = [];

    child.stdout.on("data", (chunk: Buffer | string) => {
      stdoutChunks.push(asBuffer(chunk));
    });
    child.stderr.on("data", (chunk: Buffer | string) => {
      stderrChunks.push(asBuffer(chunk));
    });
    child.on("error", reject);
    child.on("close", (status) => {
      resolve({
        status,
        stderr: Buffer.concat(stderrChunks).toString("utf8"),
        stdout: Buffer.concat(stdoutChunks).toString("utf8"),
      });
    });
  });
}

function asBuffer(chunk: Buffer | string): Buffer {
  return typeof chunk === "string" ? Buffer.from(chunk) : chunk;
}

function writeStream(stream: Writable, text: string): Promise<void> {
  return new Promise((resolve, reject) => {
    stream.write(text, (error) => {
      if (error) {
        reject(error);
        return;
      }

      resolve();
    });
  });
}
