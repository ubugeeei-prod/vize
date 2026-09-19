import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import path from "node:path";

const require = createRequire(import.meta.url);

export function assertVueTsc(directory: string): void {
  const cli = path.join(path.dirname(require.resolve("vue-tsc/package.json")), "bin/vue-tsc.js");
  const result = spawnSync(process.execPath, [cli, "--noEmit", "--pretty", "false"], {
    cwd: directory,
    encoding: "utf8",
    timeout: 60_000,
  });
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, result.stdout + result.stderr);
}
