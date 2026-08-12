import { spawnSync } from "node:child_process";

/**
 * Child-process helpers shared by the release smoke scripts.
 *
 * They lived in three copies before #3956 added a fourth caller; one copy keeps
 * the Windows batch workaround and the failure rendering identical everywhere.
 */

/**
 * Runs a command and returns the full result, including a non-zero status.
 *
 * Node 22+ refuses to spawn `.cmd` / `.bat` directly (CVE-2024-27980) and
 * returns EINVAL. The Windows runner reaches this code for the moonbit helper
 * (`MOON_BIN: …\moon.cmd`). Route through cmd.exe via `shell: true` when the
 * resolved command ends in a Windows batch suffix; the smoke args contain no
 * shell metacharacters, so quoting them is a no-op.
 */
export function runResult(command, args, options = {}) {
  const isWindowsBatch = process.platform === "win32" && /\.(cmd|bat)$/i.test(command);
  const result = spawnSync(command, args, {
    cwd: options.cwd,
    encoding: "utf8",
    env: options.env ?? process.env,
    input: options.input,
    stdio: ["pipe", "pipe", "pipe"],
    shell: isWindowsBatch,
  });

  if (result.error != null) {
    throw result.error;
  }

  return result;
}

/** Renders a completed process's output the way the smoke reports failures. */
export function renderOutput(result) {
  return [result.stdout, result.stderr].filter(Boolean).join("\n").trim();
}

/** Runs a command and throws with its output unless it exits zero. */
export function run(command, args, options = {}) {
  const result = runResult(command, args, options);
  if (result.status !== 0) {
    throw new Error(
      [`${command} ${args.join(" ")} failed with exit ${result.status}`, renderOutput(result)]
        .filter(Boolean)
        .join("\n"),
    );
  }
  return result.stdout;
}
