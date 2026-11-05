import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { closeSync, mkdirSync, openSync, readFileSync, writeFileSync, writeSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

// Keep the executable outside node_modules so Node can strip its TypeScript.
export const mapperCaptureAsset = fileURLToPath(import.meta.url);

export function installMapperCapture(
  packageRoot: string,
  captureDirectory: string,
  command: readonly string[],
): string[] {
  assert.ok(command.length > 0 && command.every((part) => typeof part === "string"));
  mkdirSync(captureDirectory);
  const configuration = join(captureDirectory, "command.json");
  writeFileSync(configuration, JSON.stringify({ command, packageRoot }, null, 2) + "\n", {
    flag: "wx",
  });
  return [process.execPath, mapperCaptureAsset, "--capture", captureDirectory, configuration];
}

async function capture(directory: string, configuration: string) {
  const { command, packageRoot } = JSON.parse(readFileSync(configuration, "utf8")) as {
    command: string[];
    packageRoot: string;
  };
  assert.ok(command.length > 0 && command.every((part) => typeof part === "string"));
  const input = openSync(join(directory, `in-${process.pid}.bin`), "wx");
  const output = openSync(join(directory, `out-${process.pid}.bin`), "wx");
  const errors = openSync(join(directory, `err-${process.pid}.bin`), "wx");
  const child = spawn(command[0], command.slice(1), { stdio: ["pipe", "pipe", "pipe"] });
  let error: string | null = null;
  const recordInput = (bytes: Buffer) => writeSync(input, bytes);
  const fail = (cause: Error) => {
    error ??= cause.message;
    child.kill("SIGTERM");
  };
  child.once("error", fail);
  child.stdin.on("error", fail);
  process.stdin.on("error", fail);
  child.stdout.on("error", fail);
  child.stderr.on("error", fail);
  process.stdout.on("error", fail);
  process.stderr.on("error", fail);
  process.stdin.on("data", recordInput);
  child.stdout.on("data", (bytes: Buffer) => writeSync(output, bytes));
  child.stderr.on("data", (bytes: Buffer) => writeSync(errors, bytes));
  process.stdin.pipe(child.stdin);
  child.stdout.pipe(process.stdout);
  child.stderr.pipe(process.stderr);
  const [status, signal] = await new Promise<[number | null, NodeJS.Signals | null]>((done) =>
    child.once("close", (code, reason) => done([code, reason])),
  );
  process.stdin.unpipe(child.stdin);
  process.stdin.removeListener("data", recordInput);
  process.stdin.pause();
  for (const fd of [input, output, errors]) closeSync(fd);
  writeFileSync(
    join(directory, `process-${process.pid}.json`),
    JSON.stringify(
      {
        command,
        cwd: process.cwd(),
        packageRoot,
        pid: child.pid ?? null,
        status,
        signal,
        error,
      },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
  process.exitCode = error || signal ? 1 : (status ?? 1);
}

if (process.argv[1] && resolve(process.argv[1]) === mapperCaptureAsset) {
  assert.equal(process.argv[2], "--capture");
  const [directory, configuration] = process.argv.slice(3);
  assert.ok(directory && configuration);
  capture(directory, configuration).catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });
}
