#!/usr/bin/env node
import { spawn } from "node:child_process";
console.error(
  "VIZE_TASK_CACHE_SOURCE_ENV " +
    JSON.stringify({
      cwd: process.cwd(),
      xdgCacheHome: process.env.XDG_CACHE_HOME,
      tmpdir: process.env.TMPDIR,
      argv: process.argv.slice(2),
    }),
);
const child = spawn("@@SOURCE_CLI@@", process.argv.slice(2), { stdio: "inherit" });
const stop = (signal) => {
  if (!child.killed) child.kill(signal);
};
process.on("SIGTERM", () => stop("SIGTERM"));
process.on("SIGINT", () => stop("SIGINT"));
child.on("error", (error) => {
  throw error;
});
child.on("exit", (code, signal) => {
  if (signal) process.kill(process.pid, signal);
  else process.exitCode = code ?? 1;
});
