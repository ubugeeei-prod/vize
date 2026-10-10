import type { Writable } from "node:stream";

import { expectsLintReport, getLintTargets } from "./cli/args.js";
import { collectVueLikeFilesFromTargets } from "./cli/files.js";
import { resolveOxlintCliEntrypoint, verifyOxlintCliEntrypoint } from "./cli/oxlint.js";
import { rewriteReportedPaths } from "./cli/output.js";
import { rewriteReportedLocations } from "./cli/locations.js";
import { prepareScriptlessWorkaroundFiles } from "./cli/workaround-files.js";
import { prepareScopedSelection } from "./cli/scoped-selection.js";
import { mergeProjectOutput } from "./cli/project-output.js";
import { isStandaloneHtmlFile } from "./file-kinds.ts";
import { prepareHtmlContext, executeHtml } from "./cli/html-operation.ts";
import { mergeHtmlOutput, unavailableHtml, qualifyHtmlHost } from "./cli/html-project-output.ts";
import { runOxlint } from "./cli/process.ts";

async function main(): Promise<void> {
  const cwd = process.cwd();
  const forwardedArgs = process.argv.slice(2);
  const targets = getLintTargets(forwardedArgs);
  const candidates = new Set<string>();
  const lintFiles = collectVueLikeFilesFromTargets(cwd, targets, (file) => candidates.add(file));
  const oxlintEntrypoint = resolveOxlintCliEntrypoint(cwd);
  const version = verifyOxlintCliEntrypoint(process.execPath, oxlintEntrypoint);
  const html = lintFiles.some(isStandaloneHtmlFile)
    ? prepareHtmlContext(cwd, forwardedArgs, oxlintEntrypoint, version, candidates)
    : undefined;
  const vueFiles = html ? lintFiles.filter((file) => !isStandaloneHtmlFile(file)) : lintFiles;
  let scoped: Awaited<ReturnType<typeof prepareScopedSelection>>;
  try {
    scoped = await prepareScopedSelection(
      cwd,
      forwardedArgs,
      vueFiles,
      (args) => runOxlint(process.execPath, [oxlintEntrypoint, ...args], cwd),
      candidates,
    );
  } catch (error) {
    if (!html) throw error;
    // No normal source packet has been returned yet. Preserve one genuine
    // unsplit normal run and make preparation failure visible without a carrier.
    html.failure = error instanceof Error ? error.message : String(error);
    scoped = {
      result: await runOxlint(process.execPath, [oxlintEntrypoint, ...forwardedArgs], cwd),
    };
  }
  const sourceResult = scoped && "result" in scoped ? scoped.result : undefined;
  const transport = scoped && "prepared" in scoped ? scoped : undefined;
  const prepared = sourceResult
    ? prepareScriptlessWorkaroundFiles(cwd, [])
    : (transport?.prepared ?? prepareScriptlessWorkaroundFiles(cwd, vueFiles));
  const args = [
    oxlintEntrypoint,
    ...(transport?.args ?? [...forwardedArgs, ...prepared.appendedArgs]),
  ];

  try {
    let result: Awaited<ReturnType<typeof runOxlint>> | undefined;
    let merged: Awaited<ReturnType<typeof runOxlint>>;
    try {
      result = sourceResult ?? (await runOxlint(process.execPath, args, cwd));
      const mappedStdout = rewriteReportedPaths(
        rewriteReportedLocations(result.stdout, prepared.locations),
        prepared.pathReplacements,
      );
      const mappedStderr = rewriteReportedPaths(
        rewriteReportedLocations(result.stderr, prepared.locations),
        prepared.pathReplacements,
      );
      merged = {
        ...result,
        stdout: mappedStdout,
        stderr: mappedStderr,
        rawStdout: mappedStdout === result.stdout ? result.rawStdout : undefined,
        rawStderr: mappedStderr === result.stderr ? result.rawStderr : undefined,
      };
      if (transport?.originalResult)
        merged = mergeProjectOutput(transport.originalResult, merged, forwardedArgs);
      if (html) {
        try {
          qualifyHtmlHost(html, merged);
          const outcome = executeHtml(html);
          if (!outcome.completed) throw new Error("original HTML operation is incomplete");
          merged = mergeHtmlOutput(merged, outcome.completed, forwardedArgs);
        } catch (error) {
          merged = unavailableHtml(merged, error);
        }
      }
    } catch (error) {
      // Preparation, spawn, mapping and JSON failures retain authored packets.
      if (transport?.originalResult) {
        process.exitCode = Math.max(transport.originalResult.status ?? 1, result?.status ?? 1, 1);
        await writeStream(
          process.stdout,
          transport.originalResult.rawStdout ?? transport.originalResult.stdout,
        );
        await writeStream(
          process.stderr,
          Buffer.concat([
            transport.originalResult.rawStderr ?? Buffer.from(transport.originalResult.stderr),
            result?.rawStderr ?? Buffer.from(result?.stderr ?? ""),
            result?.rawStdout ?? Buffer.from(result?.stdout ?? ""),
          ]),
        );
      }
      throw error;
    }
    const { stdout, stderr } = merged;

    if (stdout) {
      await writeStream(process.stdout, merged.rawStdout ?? stdout);
    }

    if (stderr) {
      await writeStream(process.stderr, merged.rawStderr ?? stderr);
    }

    if (merged.status === 0 && stdout === "" && stderr === "" && expectsLintReport(forwardedArgs)) {
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

    process.exitCode = merged.status ?? 1;
  } finally {
    prepared.cleanup();
  }
}

main().catch((error: unknown) => {
  process.stderr.write(
    `${error instanceof Error ? (error.stack ?? error.message) : String(error)}\n`,
  );
  process.exitCode = Math.max(Number(process.exitCode) || 1, 1);
});

function writeStream(stream: Writable, text: string | Uint8Array): Promise<void> {
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
