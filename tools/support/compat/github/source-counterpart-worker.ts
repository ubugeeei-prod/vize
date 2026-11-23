import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const program = fileURLToPath(import.meta.url);

export function observePrime(event: Record<string, unknown>) {
  const directory = process.env.VIZE_TOOLING_LISTING_OBSERVATION_DIR;
  if (!directory) return;
  fs.mkdirSync(directory, { recursive: true });
  fs.appendFileSync(
    path.join(directory, "history-prime-" + process.pid + ".jsonl"),
    JSON.stringify({ time: new Date().toISOString(), pid: process.pid, ...event }) + "\n",
  );
}

export function processAlive(pid: number) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ESRCH") return false;
    throw error;
  }
}

function signalGroup(group: number, signal: NodeJS.Signals) {
  try {
    process.kill(process.platform === "win32" ? group : -group, signal);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "ESRCH") throw error;
  }
}

function monitor(owner: number, group: number, timeout: number) {
  const end = Date.now() + timeout;
  observePrime({ event: "monitor-start", owner, group, end });
  // The monitor stays in the worker's group until it can escalate the termination.
  process.on("SIGTERM", () => {});
  const timer = setInterval(() => {
    const ownerAlive = processAlive(owner);
    const groupAlive = processAlive(group);
    const expired = Date.now() >= end;
    if (ownerAlive && groupAlive && !expired) return;
    clearInterval(timer);
    observePrime({
      event: "monitor-signal",
      owner,
      group,
      ownerAlive,
      groupAlive,
      expired,
      signal: "SIGTERM",
    });
    signalGroup(group, "SIGTERM");
    setTimeout(() => {
      observePrime({ event: "monitor-signal", owner, group, signal: "SIGKILL" });
      signalGroup(group, "SIGKILL");
    }, 200);
  }, 50);
}

async function prime(
  owner: number,
  timeout: number,
  lock: string,
  token: string,
  workerFile: string,
  sources: string[],
) {
  const { ensureHostedLiteralSource } =
    await import("../../../../tests/performance/support/warm-type-backed-git-reader-literal-source.ts");
  const group = process.pid;
  observePrime({ event: "worker-start", owner, group });
  assert.equal(fs.readFileSync(lock, "utf8"), token, "same owned lock required");
  fs.writeFileSync(workerFile, token + ":" + group, { flag: "wx", mode: 0o600 });
  assert.equal(fs.readFileSync(lock, "utf8"), token, "same live owner lock required");
  assert.ok(processAlive(owner), "live owner required");
  const watcher = spawn(
    process.execPath,
    [program, "monitor", String(owner), String(timeout), String(group)],
    { stdio: "ignore" },
  );
  observePrime({ event: "monitor-launched", monitor: watcher.pid, owner, group });
  watcher.unref();
  for (const source of sources) ensureHostedLiteralSource(process.cwd(), source);
  observePrime({ event: "worker-complete", owner, group });
}

if (process.argv[1] === program) {
  const [mode, ownerText, timeoutText, ...args] = process.argv.slice(2);
  const owner = Number(ownerText);
  const timeout = Number(timeoutText);
  assert.ok(Number.isSafeInteger(owner) && owner > 0);
  assert.ok(Number.isSafeInteger(timeout) && timeout > 0 && timeout <= 240_000);
  if (mode === "monitor") {
    const group = Number(args[0]);
    assert.ok(args.length === 1 && Number.isSafeInteger(group) && group > 0);
    monitor(owner, group, timeout);
  } else {
    assert.equal(mode, "prime");
    const [lock, token, workerFile, ...sources] = args;
    await prime(owner, timeout, lock, token, workerFile, sources);
  }
}
