import { spawn } from "node:child_process";

const defaultForceKillDelayMs = 5_000;
const defaultForceSettleDelayMs = 10_000;
const defaultMaxBuffer = 1024 * 1024 * 1024;

export function runTypecheckCommand(command, args, options) {
  if (!Number.isSafeInteger(options.timeoutMs) || options.timeoutMs <= 0) {
    throw new Error("timeoutMs must be a positive safe integer");
  }
  return new Promise((settle) => {
    let stdout = "";
    let stderr = "";
    let stdoutBytes = 0;
    let stderrBytes = 0;
    let spawnError = null;
    let timeoutError = null;
    let maxBufferError = null;
    let timeoutTimer = null;
    let forceKillTimer = null;
    let forceSettleTimer = null;
    let terminating = false;
    let settled = false;
    const maxBuffer = options.maxBuffer ?? defaultMaxBuffer;
    const forceKillDelayMs =
      options.forceKillDelayMs ?? Math.min(defaultForceKillDelayMs, options.timeoutMs);
    const forceSettleDelayMs =
      options.forceSettleDelayMs ??
      forceKillDelayMs + Math.min(defaultForceSettleDelayMs, options.timeoutMs);

    const child = spawn(command, args, {
      cwd: options.cwd,
      detached: process.platform !== "win32",
      env: options.env,
      stdio: ["ignore", "pipe", "pipe"],
    });

    function stopTimers() {
      clearTimeout(timeoutTimer);
      clearTimeout(forceKillTimer);
      clearTimeout(forceSettleTimer);
    }

    function settleResult(status, signal) {
      if (settled) return;
      settled = true;
      stopTimers();
      settle({
        error: spawnError ?? timeoutError ?? maxBufferError,
        signal,
        status: spawnError == null ? status : null,
        stderr,
        stdout,
      });
    }

    function signalChildTree(signal) {
      if (child.pid == null) {
        child.kill(signal);
        return;
      }
      if (process.platform !== "win32") {
        try {
          process.kill(-child.pid, signal);
          return;
        } catch (error) {
          if (error?.code === "ESRCH") return;
        }
      }
      child.kill(signal);
    }

    function beginTermination() {
      if (terminating) return;
      terminating = true;
      signalChildTree("SIGTERM");
      forceKillTimer = setTimeout(() => signalChildTree("SIGKILL"), forceKillDelayMs);
      forceKillTimer.unref?.();
      forceSettleTimer = setTimeout(() => settleResult(null, "SIGKILL"), forceSettleDelayMs);
      forceSettleTimer.unref?.();
    }

    function killForMaxBuffer(streamName) {
      if (maxBufferError != null) return;
      maxBufferError = new Error(`${streamName} maxBuffer exceeded`);
      maxBufferError.code = "ENOBUFS";
      beginTermination();
    }

    child.stdout?.setEncoding("utf8");
    child.stderr?.setEncoding("utf8");
    child.stdout?.on("data", (chunk) => {
      stdoutBytes += Buffer.byteLength(chunk);
      if (stdoutBytes > maxBuffer) killForMaxBuffer("stdout");
      else stdout += chunk;
    });
    child.stderr?.on("data", (chunk) => {
      stderrBytes += Buffer.byteLength(chunk);
      if (stderrBytes > maxBuffer) killForMaxBuffer("stderr");
      else stderr += chunk;
    });
    child.once("error", (error) => {
      spawnError = error;
      settleResult(null, null);
    });
    child.once("close", (status, signal) => {
      settleResult(status, signal);
    });

    timeoutTimer = setTimeout(() => {
      timeoutError = new Error(`spawn timed out after ${options.timeoutMs}ms`);
      timeoutError.code = "ETIMEDOUT";
      beginTermination();
    }, options.timeoutMs);
    timeoutTimer.unref?.();
  });
}
