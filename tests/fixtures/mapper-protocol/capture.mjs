// Consumer runtime asset, copied verbatim into node_modules by the typed driver.
import { spawn } from "node:child_process";
import { closeSync, openSync, readFileSync, writeSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const directory = dirname(fileURLToPath(import.meta.url));
const command = JSON.parse(readFileSync(join(directory, "capture-command.json"), "utf8"));
const input = openSync(join(process.env.TEMPLATE_ALIAS_CAPTURE, `in-${process.pid}.bin`), "wx");
const output = openSync(join(process.env.TEMPLATE_ALIAS_CAPTURE, `out-${process.pid}.bin`), "wx");
const child = spawn(command[0], [...command.slice(1), ...process.argv.slice(2)], {
  cwd: directory,
  stdio: ["pipe", "pipe", "inherit"],
});
process.stdin.on("data", (bytes) => writeSync(input, bytes));
child.stdout.on("data", (bytes) => writeSync(output, bytes));
process.stdin.pipe(child.stdin);
child.stdout.pipe(process.stdout);
child.stdin.on("error", (error) => {
  if (error.code !== "EPIPE") throw error;
});
child.on("error", (error) => {
  console.error(error);
  process.exitCode = 1;
});
child.on("close", (status, signal) => {
  closeSync(input);
  closeSync(output);
  if (signal) process.kill(process.pid, signal);
  else process.exitCode = status ?? 1;
});
