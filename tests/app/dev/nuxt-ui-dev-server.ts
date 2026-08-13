import type { ChildProcess } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { nuxtUiApp } from "../../_helpers/apps";
import {
  ensurePortFree,
  getProcessLogs,
  killProcess,
  startDevServer,
  waitForServerReady,
} from "../../_helpers/server";
import { withViteNodeRequestBudget } from "./nuxt-ui-vite-node";

const app = nuxtUiApp;

// Nuxt renders this playground through a vite-node bridge that lives in a second
// process and talks over an IPC socket. If that process dies, Nuxt neither
// respawns it nor stops the dev server: every later request answers
// `500 IPC connection closed`, so a suite that keeps polling burns its whole
// budget against a permanently broken server. Smaller CI runners lose the bridge
// during startup, so boot until it survives readiness instead.
export const DEAD_NUXT_UI_SSR_BRIDGE = /IPC connection closed/;
const BOOT_ATTEMPTS = 3;
const SSR_READY_ATTEMPTS = 2;
// One shared window for every readiness probe of a boot: a per-probe timeout would
// multiply into the row budget once it is wide enough for a slow hosted compile.
const SSR_READY_TIMEOUT_MS = 180_000;
const SSR_READY_RETRY_DELAY_MS = 2_000;

const NUXT_UI_CONFIG_SEGMENTS = ["playgrounds", "nuxt", "nuxt.config.ts"] as const;

export type NuxtUiDevServer = {
  devServer: ChildProcess;
  /** Log index the server started at, for HMR startup-noise filtering. */
  startupLogStart: number;
};

type NuxtUiDevServerBoot = NuxtUiDevServer & {
  ssrReady: boolean;
};

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function hasDeadNuxtUiSsrBridgeLog(logs: readonly string[]): boolean {
  return logs.some((line) => DEAD_NUXT_UI_SSR_BRIDGE.test(line));
}

export function isHealthyNuxtUiSsrResponse(status: number, body: string): boolean {
  const hasAppMount = body.includes("__nuxt") || body.includes("__NUXT__");
  const isLoadingScreen = body.includes("__nuxt-loading") || body.includes("nuxt-loading");
  return status < 500 && hasAppMount && !isLoadingScreen && !DEAD_NUXT_UI_SSR_BRIDGE.test(body);
}

function hasDeadSsrBridge(devServer: ChildProcess): boolean {
  return hasDeadNuxtUiSsrBridgeLog(getProcessLogs(devServer));
}

/** Widen the playground's vite-node request budget so slow SSR compiles survive. */
function ensureViteNodeRequestBudget(): void {
  const configPath = path.join(app.cwd, ...NUXT_UI_CONFIG_SEGMENTS);
  if (!fs.existsSync(configPath)) return;

  const config = fs.readFileSync(configPath, "utf-8");
  const patched = withViteNodeRequestBudget(config);
  if (patched !== config) fs.writeFileSync(configPath, patched);
}

async function waitForHealthySsrResponse(devServer: ChildProcess): Promise<boolean> {
  const deadline = Date.now() + SSR_READY_TIMEOUT_MS;

  for (let attempt = 1; attempt <= SSR_READY_ATTEMPTS; attempt++) {
    if (hasDeadSsrBridge(devServer)) return false;
    const remainingMs = deadline - Date.now();
    if (remainingMs <= 0) break;

    try {
      const response = await fetch(app.url, {
        signal: AbortSignal.timeout(remainingMs),
      });
      const body = await response.text();
      if (isHealthyNuxtUiSsrResponse(response.status, body)) {
        return true;
      }
      if (DEAD_NUXT_UI_SSR_BRIDGE.test(body) || hasDeadSsrBridge(devServer)) {
        return false;
      }
      console.log(
        `[${app.name}] SSR readiness probe returned status ${response.status} ` +
          `(attempt ${attempt}/${SSR_READY_ATTEMPTS}); retrying...`,
      );
    } catch (error) {
      if (hasDeadSsrBridge(devServer)) return false;
      console.log(
        `[${app.name}] SSR readiness probe failed ` +
          `(attempt ${attempt}/${SSR_READY_ATTEMPTS}): ${String(error)}`,
      );
    }

    if (attempt < SSR_READY_ATTEMPTS) await sleep(SSR_READY_RETRY_DELAY_MS);
  }

  return false;
}

async function bootDevServer(): Promise<NuxtUiDevServerBoot> {
  ensureViteNodeRequestBudget();
  await ensurePortFree(app.port);

  console.log(`Starting dev server for ${app.name}...`);
  const devServer = startDevServer(app);
  const startupLogStart = getProcessLogs(devServer).length;
  devServer.on("exit", (code) => {
    console.log(`[${app.name}] dev server exited with code ${code}`);
  });

  console.log(`Waiting for ${app.name} server to be ready (port ${app.port})...`);
  await waitForServerReady(
    devServer,
    app.port,
    app.readyPattern,
    app.startupTimeout,
    app.readyDelay,
  );
  const ssrReady = await waitForHealthySsrResponse(devServer);

  return { devServer, startupLogStart, ssrReady };
}

/**
 * Start the nuxt-ui dev server, replacing one whose SSR bridge died before it
 * served a real page. Bounded restarts keep a genuinely broken playground
 * failing with a startup error instead of burning the row timeout in warmups.
 */
export async function startNuxtUiDevServer(): Promise<NuxtUiDevServer> {
  let lastFailure = "";

  for (let attempt = 1; attempt <= BOOT_ATTEMPTS; attempt++) {
    const started = await bootDevServer();
    const bridgeDied = hasDeadSsrBridge(started.devServer);
    if (started.ssrReady && !bridgeDied) {
      return {
        devServer: started.devServer,
        startupLogStart: started.startupLogStart,
      };
    }

    lastFailure = bridgeDied
      ? "SSR bridge closed during startup"
      : "SSR readiness probe did not produce a healthy page";
    const nextAction =
      attempt < BOOT_ATTEMPTS ? "restarting the dev server..." : "no attempts left.";
    console.log(
      `[${app.name}] ${lastFailure} (attempt ${attempt}/${BOOT_ATTEMPTS}); ` + nextAction,
    );
    killProcess(started.devServer);
    if (attempt < BOOT_ATTEMPTS) await sleep(2_000);
  }

  throw new Error(`${app.name} dev server failed startup: ${lastFailure}`);
}
