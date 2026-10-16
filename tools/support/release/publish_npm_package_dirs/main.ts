// Publishes every generated npm package directory below one parent directory.
import {
  collectOutput,
  ioErrorMessage,
  isSpawnError,
  println,
  runMain,
  spawnErrorText,
  writeStderr,
  writeStdout,
} from "../command_io.ts";
import {
  cliArgs,
  isDir,
  joinPath,
  lexicalCompare,
  pathExists,
  readDir,
  resolvePath,
  toolBin,
  writeStringToFile,
} from "../command_runtime.ts";
import { JsonObject, jsonNumber, stringifyJson, type JsonValue } from "../ordered_json.ts";

/** A complete result for one package, including its registry visibility wait. */
interface PublishResult {
  packageDir: string;
  exitCode: number;
  walltimeMs: number;
}

/** Keep each launched child's complete decoded UTF-8 diagnostic streams. */
interface PublishOutput {
  result: PublishResult;
  stdout: string;
  stderr: string;
}

/**
 * Keep an explicit ceiling on registry pressure, and reject malformed input
 * instead of silently falling back to a different publish mode.
 */
function parseConcurrency(value: string): number | null {
  return /^[1-8]$/.test(value) ? Number(value) : null;
}

/**
 * Run the existing single-package publisher unchanged: it owns provenance,
 * rerun safety, retries, and exact version + dist-tag visibility checks.
 * Ordinary child failures are results, so every launched child can finish and
 * keep its diagnostics.
 */
async function publishPackage(
  nodeBin: string,
  scriptPath: string,
  currentDir: string,
  packageDir: string,
  forwardArgs: readonly string[],
): Promise<PublishOutput> {
  const started = Date.now();
  println(`Publishing ${packageDir}...`);
  let exitCode: number;
  let stdout: string;
  let stderr: string;
  try {
    const output = await collectOutput(nodeBin, [scriptPath, packageDir, ...forwardArgs], {
      cwd: currentDir,
    });
    exitCode = output.status;
    stdout = output.stdout;
    stderr = output.stderr;
  } catch (error) {
    exitCode = 1;
    stdout = "";
    stderr = `Failed to run publisher for ${packageDir}: ${isSpawnError(error) ? spawnErrorText(error) : String(error)}`;
  }
  return {
    result: { packageDir, exitCode, walltimeMs: Date.now() - started },
    stdout,
    stderr,
  };
}

function receiptObject(entries: [string, JsonValue][]): JsonObject {
  const object = new JsonObject();
  for (const [key, value] of entries) object.set(key, value);
  return object;
}

const USAGE =
  "Usage: node tools/support/release/publish_npm_package_dirs/main.ts <dir> [--concurrency 1..8] [--receipt path] [--provenance] [--ignore-failure] [--retries N] [--delay N]\n";

/**
 * Main entry point for publishing generated npm package directories.
 * Serial execution is the compatible default; release opts into bounded
 * concurrency. A failed child stops queued launches while active children are
 * reaped with complete diagnostics. Any failure prevents the wrapper publish.
 */
export async function publishNpmPackageDirs(args: readonly string[]): Promise<number> {
  if (args.length === 0) {
    writeStderr(USAGE);
    return 1;
  }

  const baseDirectory = resolvePath(args[0]);
  const forwardArgs: string[] = [];
  let concurrency = 1;
  let receiptPath: string | null = null;
  let index = 1;
  while (index < args.length) {
    const arg = args[index];
    if (arg === "--concurrency") {
      const parsed = parseConcurrency(index + 1 < args.length ? args[index + 1] : "");
      if (parsed === null) {
        writeStderr("--concurrency requires an integer from 1 to 8\n");
        return 1;
      }
      concurrency = parsed;
      index += 2;
      continue;
    }
    if (arg === "--receipt") {
      if (index + 1 >= args.length || args[index + 1] === "" || args[index + 1].startsWith("--")) {
        writeStderr("--receipt requires a file path\n");
        return 1;
      }
      receiptPath = resolvePath(args[index + 1]);
      index += 2;
      continue;
    }
    forwardArgs.push(arg);
    index += 1;
  }

  const currentDir = resolvePath(process.cwd());
  const nodeBin = toolBin("NODE_BIN", process.execPath);
  const scriptPath = joinPath(currentDir, "tools/support/release/publish_npm_package/main.ts");
  const packageDirectories: string[] = [];
  for (const entryName of readDir(baseDirectory)) {
    const packageDir = joinPath(baseDirectory, entryName);
    if (isDir(packageDir) && pathExists(joinPath(packageDir, "package.json"))) {
      packageDirectories.push(packageDir);
    }
  }
  packageDirectories.sort(lexicalCompare);
  if (packageDirectories.length === 0) {
    writeStderr(`No publishable package directories were found in ${baseDirectory}\n`);
    return 1;
  }

  const started = Date.now();
  let stopLaunching = false;
  const outputs: (PublishOutput | null)[] = Array.from(
    { length: packageDirectories.length },
    () => null,
  );
  const runTask = async (taskIndex: number): Promise<void> => {
    // A slot admits this body only when it is free. No await separates the
    // stop check from launch or the failure from its stop flag.
    if (stopLaunching) return;
    const packageDir = packageDirectories[taskIndex];
    const output = await publishPackage(nodeBin, scriptPath, currentDir, packageDir, forwardArgs);
    if (output.result.exitCode !== 0 && !stopLaunching) {
      stopLaunching = true;
      // Only the first failure writes this progress marker. Child streams
      // remain buffered until collection.
      writeStderr(`Stopping queued package launches after publisher failure: ${packageDir}\n`);
    }
    outputs[taskIndex] = output;
  };
  // Launch in package order with at most `concurrency` active publishers, and
  // reap every active publisher before collecting results in package order.
  let nextTask = 0;
  const worker = async (): Promise<void> => {
    while (nextTask < packageDirectories.length) {
      const taskIndex = nextTask;
      nextTask += 1;
      await runTask(taskIndex);
    }
  };
  await Promise.all(Array.from({ length: concurrency }, () => worker()));
  const walltimeMs = Date.now() - started;

  const results: PublishResult[] = [];
  const skipped: string[] = [];
  const diagnostics: { packageDir: string; stdout: string; stderr: string }[] = [];
  let exitCode = 0;
  outputs.forEach((output, outputIndex) => {
    const packageDir = packageDirectories[outputIndex];
    if (output === null) {
      skipped.push(packageDir);
      println(`Skipped queued package after publisher failure: ${packageDir}`);
      return;
    }
    const result = output.result;
    results.push(result);
    diagnostics.push({ packageDir, stdout: output.stdout, stderr: output.stderr });
    println(`Publish result for ${result.packageDir}: exit ${result.exitCode}`);
    if (output.stdout !== "") {
      writeStdout(output.stdout);
    }
    if (output.stderr !== "") {
      writeStderr(`Publish diagnostics for ${result.packageDir}:\n`);
      writeStderr(output.stderr);
    }
    if (exitCode === 0 && result.exitCode !== 0) {
      exitCode = result.exitCode > 0 ? result.exitCode : 1;
    }
  });
  println(
    `Platform package publish summary: ${packageDirectories.length} packages, concurrency ${concurrency}, ${walltimeMs} ms, exit ${exitCode}`,
  );
  if (receiptPath !== null) {
    const receipt = receiptObject([
      ["schema_version", jsonNumber(1)],
      ["concurrency", jsonNumber(concurrency)],
      ["package_count", jsonNumber(packageDirectories.length)],
      ["walltime_ms", jsonNumber(walltimeMs)],
      ["exit_code", jsonNumber(exitCode)],
      [
        "results",
        results.map((result) =>
          receiptObject([
            ["package_dir", result.packageDir],
            ["exit_code", jsonNumber(result.exitCode)],
            ["walltime_ms", jsonNumber(result.walltimeMs)],
          ]),
        ),
      ],
      ["skipped", skipped],
      [
        "diagnostics",
        diagnostics.map((diagnostic) =>
          receiptObject([
            ["package_dir", diagnostic.packageDir],
            ["stdout", diagnostic.stdout],
            ["stderr", diagnostic.stderr],
          ]),
        ),
      ],
    ]);
    try {
      writeStringToFile(receiptPath, `${stringifyJson(receipt)}\n`);
    } catch (error) {
      writeStderr(`Failed to write publish receipt to ${receiptPath}: ${ioErrorMessage(error)}\n`);
      return exitCode !== 0 ? exitCode : 1;
    }
  }
  return exitCode;
}

if (import.meta.main) {
  runMain(() => publishNpmPackageDirs(cliArgs()));
}
