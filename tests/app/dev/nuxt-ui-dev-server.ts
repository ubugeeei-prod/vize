import type { ChildProcess } from "node:child_process";
import { nuxtUiApp } from "../../_helpers/apps";
import {
  ensurePortFree,
  getProcessLogs,
  killProcess,
  startDevServer,
  waitForServerReady,
} from "../../_helpers/server";

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
const SSR_READY_TIMEOUT_MS = 90_000;
const SSR_READY_RETRY_DELAY_MS = 2_000;

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

async function waitForHealthySsrResponse(devServer: ChildProcess): Promise<boolean> {
  for (let attempt = 1; attempt <= SSR_READY_ATTEMPTS; attempt++) {
    if (hasDeadSsrBridge(devServer)) return false;

    try {
      const response = await fetch(app.url, {
        signal: AbortSignal.timeout(SSR_READY_TIMEOUT_MS),
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
