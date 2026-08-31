import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import type { Plugin } from "vite";

const pathDelimiterForPlatform = (platform: NodeJS.Platform) => (platform === "win32" ? ";" : ":");

const pathDirectoriesForEnv = (env: NodeJS.ProcessEnv, platform: NodeJS.Platform) =>
  (env.PATH ?? "")
    .split(pathDelimiterForPlatform(platform))
    .filter((directory, index, directories) => directory !== "" || directories.length > 1)
    .map((directory) => (directory === "" ? "." : directory));

const commandExtensionsForPlatform = (
  command: string,
  env: NodeJS.ProcessEnv,
  platform: NodeJS.Platform,
) => {
  if (platform !== "win32" || path.extname(command) !== "") {
    return [""];
  }

  return (env.PATHEXT ?? ".COM;.EXE;.BAT;.CMD")
    .split(";")
    .map((extension) => extension.trim())
    .filter(Boolean);
};

const isExecutableFile = (candidate: string, platform: NodeJS.Platform) => {
  try {
    const stat = fs.statSync(candidate);
    if (!stat.isFile()) return false;
    if (platform === "win32") return true;
    fs.accessSync(candidate, fs.constants.X_OK);
    return true;
  } catch {
    return false;
  }
};

export const commandExists = (
  command: string,
  env: NodeJS.ProcessEnv = process.env,
  platform: NodeJS.Platform = process.platform,
) => {
  const directories = /[/\\]/.test(command) ? [""] : pathDirectoriesForEnv(env, platform);
  const extensions = commandExtensionsForPlatform(command, env, platform);

  return directories.some((directory) =>
    extensions.some((extension) =>
      isExecutableFile(
        directory === "" ? `${command}${extension}` : path.join(directory, command + extension),
        platform,
      ),
    ),
  );
};

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
