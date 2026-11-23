import { shellLauncher } from "../../../process-launcher.ts";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

/** The shell execs the original binary, so sampling keeps the actual LSP PID.
 * Version probes pass through without creating an unrelated profile window. */
export function profiledLauncher(server: string, workspace: string, profile: string): string {
  assert.ok(path.isAbsolute(server) && path.isAbsolute(profile));
  assert.equal(fs.existsSync(profile), false, "each session owns a new profile file");
  fs.mkdirSync(path.dirname(profile), { recursive: true });
  const launcher = path.join(workspace, ".vize-profile-lsp.sh");
  fs.writeFileSync(launcher, shellLauncher("profile-lsp", { COMMAND: server, PROFILE: profile }), {
    mode: 0o700,
    flag: "wx",
  });
  return launcher;
}
