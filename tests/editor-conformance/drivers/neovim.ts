// Real headless Neovim (pinned in CI) running `neovim.lua` with the packaged
// vize.nvim plugin on the runtimepath.
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import { repoRoot, suiteRoot, type Driver, type DriverContext } from "../support/context.ts";

const nvim = process.env.VIZE_TEST_NVIM_PATH ?? "nvim";

function version(): string {
  const output = spawnSync(nvim, ["--version"], { encoding: "utf8" }).stdout ?? "";
  return /^NVIM v(\S+)/mu.exec(output)?.[1] ?? "unknown";
}

async function run(context: DriverContext): Promise<void> {
  const scenarioFile = path.join(context.scratch, "scenario.resolved.json");
  fs.writeFileSync(scenarioFile, JSON.stringify(context.scenario));
  const plugin = path.join(repoRoot, "editors", "nvim");
  const driver = path.join(suiteRoot, "drivers", "neovim.lua");
  const child = spawn(
    nvim,
    [
      "--headless",
      "-u",
      "NONE",
      "-i",
      "NONE",
      "-n",
      "--cmd",
      `set runtimepath^=${plugin.replaceAll(" ", "\\ ")}`,
      "-c",
      `luafile ${driver.replaceAll(" ", "\\ ")}`,
    ],
    {
      cwd: context.workspace,
      env: {
        ...context.env,
        VIZE_TS45_SCENARIO: scenarioFile,
        VIZE_TS45_WORKSPACE: context.workspace,
        XDG_CONFIG_HOME: path.join(context.scratch, "config"),
        XDG_DATA_HOME: path.join(context.scratch, "data"),
        XDG_STATE_HOME: path.join(context.scratch, "state"),
      },
      stdio: ["ignore", "inherit", "inherit"],
    },
  );
  const code = await new Promise<number | null>((resolve) => child.on("close", resolve));
  if (code !== 0) throw new Error(`headless Neovim exited with ${code}`);
}

export const driver: Driver = { client: "Neovim", version, mode: "editor", run };
