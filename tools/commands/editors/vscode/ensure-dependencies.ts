import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { windowsInvocation } from "../../../support/command-invocation.ts";

const cwd = fileURLToPath(new URL("../../../../editors/vscode/", import.meta.url));
const probe = spawnSync(
  process.execPath,
  ["../../tools/commands/editors/vscode/run-package-bin.ts", "vite-plus", "vp", "--version"],
  { cwd, stdio: "ignore" },
);
if (probe.status !== 0) {
  const install = windowsInvocation("corepack", [
    "pnpm",
    "install",
    "--ignore-workspace",
    "--lockfile-dir",
    ".",
    "--frozen-lockfile",
    "--prefer-offline",
    "--ignore-scripts",
  ]);
  const result = spawnSync(install.command, install.args, {
    cwd,
    stdio: "inherit",
    windowsVerbatimArguments: install.verbatim,
  });
  if (result.error) console.error(result.error.message);
  process.exitCode = result.status ?? 1;
}
