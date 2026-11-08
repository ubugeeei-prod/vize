import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const command = path.join(root, "tools/commands/editors/e2e/real-vue-workspace.ts");

test("the real Vue editor workspace command exposes its stable Node interface", () => {
  const help = execFileSync(process.execPath, [command, "--help"], {
    cwd: root,
    encoding: "utf8",
  });
  assert.match(help, /^Usage: node tools\/commands\/editors\/e2e\/real-vue-workspace\.ts/u);
  assert.match(help, /prepare <workspace> materializes the shared real-vue editor fixture/u);

  const unknown = spawnSync(process.execPath, [command, "foreign"], {
    cwd: root,
    encoding: "utf8",
  });
  assert.equal(unknown.status, 1);
  assert.equal(unknown.stdout, "");
  assert.match(unknown.stderr, /^unknown command foreign/u);
});
