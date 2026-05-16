import { execFileSync, spawnSync } from "node:child_process";
import type { Plugin } from "vite";

const commandExists = (command: string) =>
  spawnSync("sh", ["-c", `command -v ${command}`], { stdio: "ignore" }).status === 0;

/**
 * Builds the root library artifact by delegating to the workspace build task.
 *
 * The root Vite build exists only as a stable Vite+ entry point for task
 * orchestration. The actual production artifacts still come from the workspace
 * build task, and this plugin chooses `nix develop` only when the local machine
 * needs Nix to provide missing native tools.
 */
export const rootBuildTaskPlugin = (): Plugin => ({
  name: "vize-root-build-task",
  apply: "build",
  closeBundle() {
    if (process.env.VIZE_SKIP_ROOT_BUILD_TASK === "1") {
      return;
    }

    const buildCommand = ["vp", "run", "--workspace-root", "build"];
    const command = commandExists("wasm-pack") || !commandExists("nix") ? "vp" : "nix";
    const args =
      command === "vp"
        ? buildCommand.slice(1)
        : ["--option", "warn-dirty", "false", "develop", "--command", ...buildCommand];

    execFileSync(command, args, {
      env: {
        ...process.env,
        VIZE_SKIP_ROOT_BUILD_TASK: "1",
      },
      stdio: "inherit",
    });
  },
});
