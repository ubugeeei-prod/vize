import { execSync, spawn, type ChildProcess } from "node:child_process";
import { createConnection } from "node:net";
import type { AppConfig } from "./apps.ts";

const VITE_PLUS_BIN = `${process.env.HOME ?? ""}/.vite-plus/bin`;
const PROCESS_LOGS = new WeakMap<ChildProcess, string[]>();

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function waitSync(ms: number): void {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms);
}

function isPortOpen(port: number): Promise<boolean> {
  return new Promise((resolve) => {
    const socket = createConnection({ port, host: "127.0.0.1" }, () => {
      socket.destroy();
      resolve(true);
    });
    socket.on("error", () => {
      socket.destroy();
      resolve(false);
    });
  });
}

function getListeningPids(port: number): number[] {
  try {
    const output = execSync(`lsof -tiTCP:${port} -sTCP:LISTEN`, {
      encoding: "utf-8",
      timeout: 5000,
    }).trim();
    if (!output) {
      return [];
    }
    return output
      .split("\n")
      .map((pid) => Number(pid.trim()))
      .filter((pid) => Number.isInteger(pid) && pid > 0);
  } catch {
    return [];
  }
}

function killPid(pid: number, signal: NodeJS.Signals): void {
  try {
    process.kill(pid, signal);
  } catch {
    // already gone
  }
}

function recordProcessLines(
  proc: ChildProcess,
  appName: string,
  stream: "stdout" | "stderr",
  data: Buffer,
): void {
  const lines = data
    .toString()
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);

  if (lines.length === 0) {
    return;
  }

  let logs = PROCESS_LOGS.get(proc);
  if (!logs) {
    logs = [];
    PROCESS_LOGS.set(proc, logs);
  }

  for (const line of lines) {
    logs.push(line);
    console.log(`[${appName}:${stream}] ${line}`);
  }
}

export function getProcessLogs(proc: ChildProcess): readonly string[] {
  return PROCESS_LOGS.get(proc) ?? [];
}

export function waitForServerReady(
  proc: ChildProcess,
  port: number,
  readyPattern: RegExp,
  timeout: number,
  readyDelay?: number,
): Promise<void> {
  return new Promise((resolve, reject) => {
    const deadline = Date.now() + timeout;
    let resolved = false;
    let processExited = false;

    function checkDone() {
      if (resolved) return;
      resolved = true;
      resolve();
    }

    function checkFailed(reason: string) {
      if (resolved) return;
      resolved = true;
      reject(new Error(reason));
    }

    const stripAnsi = (s: string) => s.replace(/\x1b\[[0-9;]*m/g, "");
    const onData = (data: Buffer) => {
      const text = stripAnsi(data.toString());
      if (readyPattern.test(text)) {
        setTimeout(checkDone, readyDelay ?? 1000);
      }
    };
    proc.stdout?.on("data", onData);
    proc.stderr?.on("data", onData);

    proc.on("exit", (code) => {
      processExited = true;
      checkFailed(`Dev server process exited with code ${code} before becoming ready`);
    });

    function attemptTcp() {
      if (resolved || processExited) return;
      if (Date.now() > deadline) {
        checkFailed(`Server did not become ready within ${timeout}ms (port ${port})`);
        return;
      }
      const socket = createConnection({ port, host: "127.0.0.1" }, () => {
        socket.destroy();
        checkDone();
      });
      socket.on("error", () => {
        socket.destroy();
        setTimeout(attemptTcp, 2000);
      });
    }

    setTimeout(attemptTcp, 3000);
  });
}

export function startDevServer(app: AppConfig): ChildProcess {
  const env = {
    ...process.env,
    PATH: `${VITE_PLUS_BIN}:${process.env.PATH}`,
    NODE_ENV: "development",
    BROWSER: "none",
    ...app.env,
  };

  const proc = spawn(app.command, app.args, {
    cwd: app.cwd,
    env,
    stdio: ["ignore", "pipe", "pipe"],
    detached: true,
  });

  proc.stdout?.on("data", (data: Buffer) => {
    recordProcessLines(proc, app.name, "stdout", data);
  });

  proc.stderr?.on("data", (data: Buffer) => {
    recordProcessLines(proc, app.name, "stderr", data);
  });

  return proc;
}

export function startPreviewServer(app: AppConfig): ChildProcess {
  if (!app.preview) throw new Error(`No preview config for ${app.name}`);

  const env = {
    ...process.env,
    PATH: `${VITE_PLUS_BIN}:${process.env.PATH}`,
    NODE_ENV: "production",
    ...app.env,
  };

  const proc = spawn(app.preview.command, app.preview.args, {
    cwd: app.cwd,
    env,
    stdio: ["ignore", "pipe", "pipe"],
    detached: true,
  });

  proc.stdout?.on("data", (data: Buffer) => {
    recordProcessLines(proc, `${app.name}:preview`, "stdout", data);
  });

  proc.stderr?.on("data", (data: Buffer) => {
    recordProcessLines(proc, `${app.name}:preview`, "stderr", data);
  });

  return proc;
}

export async function ensurePortFree(port: number): Promise<void> {
  if (!(await isPortOpen(port))) {
    return;
  }

  console.log(`[warn] Port ${port} is in use, attempting to free it...`);
  const deadline = Date.now() + 15_000;

  while (Date.now() < deadline) {
    const pids = getListeningPids(port);
    if (pids.length === 0) {
      if (!(await isPortOpen(port))) {
        return;
      }
      await sleep(500);
      continue;
    }

    for (const pid of pids) {
      killPid(pid, "SIGTERM");
    }
    await sleep(1000);

    if (!(await isPortOpen(port))) {
      return;
    }

    for (const pid of pids) {
      killPid(pid, "SIGKILL");
    }
    await sleep(1000);

    if (!(await isPortOpen(port))) {
      return;
    }
  }

  throw new Error(`Port ${port} is still in use after cleanup attempts`);
}

export async function waitForHttpReady(url: string, _port: number, maxRetries = 30): Promise<void> {
  for (let i = 0; i < maxRetries; i++) {
    try {
      const res = await fetch(url, { signal: AbortSignal.timeout(5000) });
      if (res.status < 500) {
        const body = await res.text();
        // Nuxt loading screen returns 200 but has minimal HTML - wait for real content
        const isLoadingScreen = body.includes("__nuxt-loading") || body.includes("nuxt-loading");
        // SPA pages (misskey) are small but valid; SSR pages (Nuxt) should be larger
        const hasAppMount = body.includes("__nuxt") || body.includes("__NUXT__") || body.includes("misskey_app");
        if (hasAppMount && !isLoadingScreen) {
          return;
        }
      }
    } catch {
      // retry
    }
    await new Promise((r) => setTimeout(r, 2000));
  }
  console.log(`[warn] HTTP health check did not succeed for ${url} after ${maxRetries} retries`);
}

export function killProcess(proc: ChildProcess | undefined): void {
  if (proc?.pid) {
    try {
      process.kill(-proc.pid, "SIGTERM");
    } catch {
      try {
        proc.kill("SIGTERM");
      } catch {
        // already dead
      }
    }

    const deadline = Date.now() + 5000;
    while (Date.now() < deadline) {
      try {
        process.kill(-proc.pid, 0);
        waitSync(200);
      } catch {
        return;
      }
    }

    try {
      process.kill(-proc.pid, "SIGKILL");
    } catch {
      try {
        proc.kill("SIGKILL");
      } catch {
        // already dead
      }
    }
  }
}

export function runBuild(app: AppConfig): void {
  if (!app.build) throw new Error(`No build config for ${app.name}`);

  const env = {
    ...process.env,
    PATH: `${VITE_PLUS_BIN}:${process.env.PATH}`,
    NODE_ENV: "production",
    ...app.env,
  };

  console.log(`[${app.name}] Running build: ${app.build.command} ${app.build.args.join(" ")}`);
  execSync(`${app.build.command} ${app.build.args.join(" ")}`, {
    cwd: app.cwd,
    env,
    stdio: "inherit",
    timeout: app.build.timeout,
  });
  console.log(`[${app.name}] Build completed`);
}
