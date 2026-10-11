import path from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import {
  ciBuildCommand,
  beforeCiBuild,
  captureCiProcess,
  captureCiBuild,
  preserveCiBuildAttempt,
} from "./formatter-history-ci-build.mjs";

export function runCiBuild(
  nativeDir,
  { capture = process.env.VIZE_NATIVE_CI_FORMATTER_HISTORY === "1", run = spawnSync } = {},
) {
  if (capture && (process.env.GITHUB_ACTIONS !== "true" || process.platform !== "linux"))
    throw Error("ci formatter custody requires the actual Linux Actions build");
  const command = capture ? ciBuildCommand : ciBuildCommand.slice(0, -3);
  const before = capture ? beforeCiBuild(nativeDir) : null;
  const options = {
    cwd: nativeDir,
    env: process.env,
    stdio: capture ? ["inherit", "pipe", "pipe"] : "inherit",
    shell: process.platform === "win32",
    maxBuffer: Infinity,
  };
  const build = run(command[0], command.slice(1), options);
  if (capture) {
    process.stdout.write(build.stdout ?? Buffer.alloc(0));
    process.stderr.write(build.stderr ?? Buffer.alloc(0));
  }
  const buildPacket = capture ? captureCiProcess(command, build) : null;
  if (capture) preserveCiBuildAttempt(nativeDir, before, buildPacket);
  if (build.status !== 0) return build.status ?? 1;
  const syncCommand = [process.execPath, path.join(nativeDir, "scripts/sync-entrypoint.mjs")];
  const sync = run(syncCommand[0], syncCommand.slice(1), { ...options, shell: false });
  if (capture) {
    process.stdout.write(sync.stdout ?? Buffer.alloc(0));
    process.stderr.write(sync.stderr ?? Buffer.alloc(0));
  }
  const syncPacket = capture ? captureCiProcess(syncCommand, sync) : null;
  if (capture) preserveCiBuildAttempt(nativeDir, before, buildPacket, syncPacket);
  if (sync.status !== 0) return sync.status ?? 1;
  if (capture) captureCiBuild(nativeDir, before, buildPacket, syncPacket);

  return 0;
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const nativeDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  process.exitCode = runCiBuild(nativeDir);
}
