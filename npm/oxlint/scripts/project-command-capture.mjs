import assert from "node:assert/strict";
import { spawn } from "node:child_process";

// Keep the original command, stdin and complete binary/UTF-8 result, while an
// adjacent owned worker's deadline and cancellation remain responsive.
export function captureProjectCommand(command, args, options = {}) {
  const { timeout = 0, maxBuffer = 1024 * 1024, encoding, input, ...spawnOptions } = options;
  assert.ok(Number.isSafeInteger(timeout) && timeout >= 0);
  assert.ok(Number.isSafeInteger(maxBuffer) && maxBuffer > 0);
  assert.notEqual(process.platform, "win32", "source Actions owns POSIX command groups");
  return new Promise((resolve) => {
    const streams = { stdout: [], stderr: [] };
    let bytes = 0,
      error,
      timer,
      escalation,
      child;
    const kill = (signal) => {
      if (!child?.pid) return;
      try {
        process.kill(-child.pid, signal);
      } catch (failure) {
        if (failure.code !== "ESRCH") error ??= failure;
      }
    };
    const stop = (reason, code, immediate = false) => {
      error ??= Object.assign(new Error(reason), { code });
      kill(immediate ? "SIGKILL" : "SIGTERM");
      if (!immediate) escalation ??= setTimeout(() => kill("SIGKILL"), 200);
    };
    const onInterrupt = () => stop("interrupted by SIGINT", "EINTR");
    const onTerminate = () => stop("interrupted by SIGTERM", "EINTR");
    const onExit = () => kill("SIGKILL");
    const finish = (status, signal) => {
      // A closed leader can leave ignored-stdio descendants in its owned group.
      kill("SIGKILL");
      clearTimeout(timer);
      clearTimeout(escalation);
      process.removeListener("SIGINT", onInterrupt);
      process.removeListener("SIGTERM", onTerminate);
      process.removeListener("exit", onExit);
      const decode = (chunks) => {
        const buffer = Buffer.concat(chunks);
        return encoding ? buffer.toString(encoding) : buffer;
      };
      const stdout = decode(streams.stdout),
        stderr = decode(streams.stderr);
      resolve({
        pid: child?.pid,
        output: [null, stdout, stderr],
        stdout,
        stderr,
        status,
        signal,
        ...(error ? { error } : {}),
      });
    };
    process.once("SIGINT", onInterrupt);
    process.once("SIGTERM", onTerminate);
    process.once("exit", onExit);
    try {
      child = spawn(command, args, {
        ...spawnOptions,
        detached: true,
        stdio: ["pipe", "pipe", "pipe"],
      });
    } catch (failure) {
      error = failure;
      finish(null, null);
      return;
    }
    child.stdin.on("error", (failure) => {
      if (failure.code !== "EPIPE") error ??= failure;
    });
    child.stdin.end(input);
    for (const stream of ["stdout", "stderr"])
      child[stream].on("data", (chunk) => {
        streams[stream].push(Buffer.from(chunk));
        bytes += chunk.length;
        if (bytes > maxBuffer) stop("command exceeded original output bound", "ENOBUFS", true);
      });
    child.once("error", (failure) => {
      error ??= failure;
    });
    child.once("close", finish);
    if (timeout)
      timer = setTimeout(() => stop(`command timed out after ${timeout} ms`, "ETIMEDOUT"), timeout);
  });
}
