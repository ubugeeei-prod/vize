/**
 * Installs the Linux ARM64 cross compiler on an Ubuntu runner.
 *
 * Usage: node tools/support/github/install-cross-compile-tools.ts
 */
import { isEntrypoint, runCommand, runEntrypoint } from "../command-runtime.ts";

export function main(): number {
  const updateExit = runCommand("sudo", ["apt-get", "update"]);
  if (updateExit !== 0) return updateExit;
  return runCommand("sudo", ["apt-get", "install", "-y", "gcc-aarch64-linux-gnu"]);
}

if (isEntrypoint(import.meta.url)) runEntrypoint(main);
