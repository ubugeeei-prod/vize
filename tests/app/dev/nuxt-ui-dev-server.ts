import type { ChildProcess } from "node:child_process";
import { nuxtUiApp } from "../../_helpers/apps";
import {
  ensurePortFree,
  getProcessLogs,
  killProcess,
  startDevServer,
  waitForHttpReady,
  waitForServerReady,
} from "../../_helpers/server";

const app = nuxtUiApp;

// Nuxt renders this playground through a vite-node bridge that lives in a second
// process and talks over an IPC socket. If that process dies, Nuxt neither
// respawns it nor stops the dev server: every later request answers
// `500 IPC connection closed`, so a suite that keeps polling burns its whole
// budget against a permanently broken server. Smaller CI runners lose the bridge
// during startup, so boot until it survives readiness instead.
const DEAD_SSR_BRIDGE = /IPC connection closed/;
const BOOT_ATTEMPTS = 2;

export type NuxtUiDevServer = {
  devServer: ChildProcess;
  /** Log index the server started at, for HMR startup-noise filtering. */
  startupLogStart: number;
};

function hasDeadSsrBridge(devServer: ChildProcess): boolean {
  return getProcessLogs(devServer).some((line) => DEAD_SSR_BRIDGE.test(line));
}

async function bootDevServer(): Promise<NuxtUiDevServer> {
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
  await waitForHttpReady(app.url, app.port);

  return { devServer, startupLogStart };
}

/**
 * Start the nuxt-ui dev server, replacing one whose SSR bridge died before the
 * server became ready. Bounded restarts keep a genuinely broken playground
 * failing on its own assertions instead of looping here.
 */
export async function startNuxtUiDevServer(): Promise<NuxtUiDevServer> {
  let started = await bootDevServer();

  for (let attempt = 1; attempt < BOOT_ATTEMPTS; attempt++) {
    if (!hasDeadSsrBridge(started.devServer)) break;
    console.log(
      `[${app.name}] SSR bridge closed during startup (attempt ${attempt}); ` +
        "restarting the dev server...",
    );
    killProcess(started.devServer);
    await new Promise((resolve) => setTimeout(resolve, 2_000));
    started = await bootDevServer();
  }

  return started;
}
