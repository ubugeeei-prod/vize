import { spawn } from "node:child_process";
import path from "node:path";
import process from "node:process";
import { elkApp, misskeyApp, npmxApp, vuefesApp, type AppConfig } from "../../../_helpers/apps.ts";
import { BASE_ENV, REPO_ROOT } from "./env.ts";
import { ensureMisskeyDevConfig, getMisskeyPnpmCommand, runMisskeyBeforeStart } from "./misskey.ts";
import { replacePortInArgs, replacePortInUrl, resolveAvailablePort } from "./ports.ts";
import type { LaunchConfig, Target } from "./types.ts";

async function toLaunchConfig(
  app: AppConfig,
  targetName: Exclude<Target, "playground" | "misskey">,
): Promise<LaunchConfig> {
  const port = await resolveAvailablePort(app.port);
  if (port !== app.port) {
    console.log(`[${targetName}] Port ${app.port} is busy. Using ${port}.`);
  }

  return {
    target: targetName,
    url: replacePortInUrl(app.url, app.port, port),
    setup: app.setup,
    cwd: app.cwd,
    command: app.command,
    args: replacePortInArgs(app.args, port),
    env: app.env,
  };
}

/**
 * Creates the foreground launch plan for a selected real-world dev target.
 *
 * The plan is intentionally data-shaped: setup, preflight, working directory,
 * command, arguments, and environment are resolved before any foreground process
 * is started. That makes the launcher testable without having to boot Vite,
 * Nuxt, or Misskey in unit tests.
 */
export async function createLaunchConfig(currentTarget: Target): Promise<LaunchConfig> {
  if (currentTarget === "playground") {
    return {
      target: "playground",
      url: "http://127.0.0.1:4173",
      cwd: REPO_ROOT,
      command: "pnpm",
      args: ["-C", "playground", "dev"],
      env: {
        CI: "true",
      },
    };
  }

  if (currentTarget === "misskey") {
    const misskeyRoot = path.resolve(misskeyApp.cwd, "../..");
    const port = await resolveAvailablePort(3000);
    const configName = "vize-dev.yml";
    const misskeyCommand = getMisskeyPnpmCommand(misskeyRoot, ["dev"]);
    return {
      target: "misskey",
      url: `http://127.0.0.1:${port}`,
      setup: misskeyApp.setup,
      beforeStart: () => {
        ensureMisskeyDevConfig(misskeyRoot, port);
        runMisskeyBeforeStart(misskeyRoot, configName);
      },
      cwd: misskeyRoot,
      command: misskeyCommand.command,
      args: misskeyCommand.args,
      env: {
        MISSKEY_CONFIG_YML: configName,
      },
    };
  }

  if (currentTarget === "npmx") {
    return await toLaunchConfig(npmxApp, "npmx");
  }

  if (currentTarget === "elk") {
    return await toLaunchConfig(elkApp, "elk");
  }

  return await toLaunchConfig(vuefesApp, "vuefes");
}

/**
 * Starts the selected dev server in the foreground and forwards termination
 * signals to the child process.
 *
 * This function deliberately never resolves on success: the child process owns
 * the terminal until it exits, and the parent mirrors its exit code or signal so
 * Vite+ reports the same lifecycle result the server produced.
 */
export async function startForeground(config: LaunchConfig): Promise<never> {
  console.log(`Starting ${config.target} on ${config.url}`);

  return await new Promise<never>((_, reject) => {
    const child = spawn(config.command, config.args, {
      cwd: config.cwd,
      env: {
        ...BASE_ENV,
        ...config.env,
      },
      stdio: "inherit",
    });

    const forwardSignal = (signal: NodeJS.Signals) => {
      if (!child.killed) {
        child.kill(signal);
      }
    };

    process.on("SIGINT", () => forwardSignal("SIGINT"));
    process.on("SIGTERM", () => forwardSignal("SIGTERM"));

    child.on("error", reject);
    child.on("exit", (code, signal) => {
      if (signal) {
        process.kill(process.pid, signal);
        return;
      }
      process.exit(code ?? 0);
    });
  });
}
