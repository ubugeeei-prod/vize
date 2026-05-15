import { spawn } from "node:child_process";
import type { Writable } from "node:stream";

import { getLintTargets } from "./cli/args.js";
import { collectVueFilesFromTargets } from "./cli/files.js";
import { resolveOxlintCliEntrypoint } from "./cli/oxlint.js";
import { rewriteReportedPaths } from "./cli/output.js";
import { prepareScriptlessWorkaroundFiles } from "./cli/workaround-files.js";

async function main(): Promise<void> {
  const cwd = process.cwd();
  const forwardedArgs = process.argv.slice(2);
  const targets = getLintTargets(forwardedArgs);
  const vueFiles = collectVueFilesFromTargets(cwd, targets);
  const prepared = prepareScriptlessWorkaroundFiles(cwd, vueFiles);
  const oxlintEntrypoint = resolveOxlintCliEntrypoint(cwd);
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

    if (prepared.usedScriptlessWorkaround && forwardedArgs.includes("--fix")) {
      await writeStream(
        process.stderr,
        "\n[oxlint-plugin-vize] Scriptless SFC workaround is active; fixes are not applied back to original .vue files yet.\n",
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
