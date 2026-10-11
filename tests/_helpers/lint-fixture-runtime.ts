import assert from "node:assert/strict";
import fs from "node:fs";
import { homedir } from "node:os";
import path from "node:path";

/** Package scripts prepend a project-local vp that has no environment commands. */
export function resolveLintEnvironmentCli(home = homedir(), platform = process.platform): string {
  const cli = path.join(home, ".vite-plus", "bin", platform === "win32" ? "vp.exe" : "vp");
  assert.ok(fs.existsSync(cli), `lint setup requires the native CLI installed by setup-vp: ${cli}`);
  const resolved = fs.realpathSync(cli);
  assert.ok(fs.statSync(resolved).isFile(), "the environment CLI must be an installed file");
  fs.accessSync(resolved, platform === "win32" ? fs.constants.F_OK : fs.constants.X_OK);
  return resolved;
}
