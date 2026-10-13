import { spawn } from "node:child_process";

function stop(child, signal) {
  try {
    if (process.platform === "win32") child.kill(signal);
    else process.kill(-child.pid, signal);
  } catch (error) {
    if (error.code !== "ESRCH") throw error;
  }
}

/** Reap each real acquisition before another attempt; inherit both raw streams. */
export async function downloadAttempts(
  command,
  args,
  {
    attempts = 2,
    attemptTimeoutMs = 140_000,
    totalTimeoutMs = 285_000,
    teardownMs = 1_000,
    environment = process.env,
  } = {},
) {
  const deadline = performance.now() + totalTimeoutMs;
  let result = 1;
  for (let attempt = 1; attempt <= attempts; attempt++) {
    const remaining = deadline - performance.now() - teardownMs;
    if (remaining <= 0) return 124;
    console.error(`VS Code acquisition attempt ${attempt}/${attempts}`);
    result = await new Promise((resolve, reject) => {
      const child = spawn(command, args, {
        env: environment,
        stdio: "inherit",
        detached: process.platform !== "win32",
      });
      let expired = false;
      let escalation;
      const interrupt = (signal) => {
        stop(child, "SIGKILL");
        process.exit(signal === "SIGINT" ? 130 : 143);
      };
      const onInterrupt = () => interrupt("SIGINT");
      const onTerminate = () => interrupt("SIGTERM");
      process.once("SIGINT", onInterrupt);
      process.once("SIGTERM", onTerminate);
      const timeout = setTimeout(
        () => {
          expired = true;
          console.error(`VS Code acquisition attempt ${attempt} exceeded its time budget`);
          stop(child, "SIGTERM");
          escalation = setTimeout(() => stop(child, "SIGKILL"), teardownMs);
        },
        Math.min(attemptTimeoutMs, remaining),
      );
      const cleanup = () => {
        clearTimeout(timeout);
        clearTimeout(escalation);
        process.removeListener("SIGINT", onInterrupt);
        process.removeListener("SIGTERM", onTerminate);
        if (child.pid) stop(child, "SIGKILL");
      };
      child.once("error", (error) => {
        cleanup();
        reject(error);
      });
      child.once("close", (code, signal) => {
        cleanup();
        resolve(expired ? 124 : (code ?? (signal === "SIGINT" ? 130 : 1)));
      });
    });
    if (result === 0) return 0;
    console.error(`VS Code acquisition attempt ${attempt} failed with exit ${result}`);
  }
  return result;
}
