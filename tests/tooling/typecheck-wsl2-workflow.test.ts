import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const workflow = readFileSync(
  new URL("../../.github/workflows/typecheck-wsl2.yml", import.meta.url),
  "utf8",
);

test("the Windows mount path survives PowerShell and WSL argument parsing", () => {
  assert.match(workflow, /\$windowsPath = \[System\.IO\.Path\]::GetFullPath/);
  assert.match(workflow, /\$tail = \$windowsPath\.Substring\(2\)\.Replace\('\\', '\/'\)/);
  assert.match(workflow, /\$mount = "\/mnt\/\$drive\$tail"/);
  assert.match(workflow, /wsl\.exe test -d \$mount/);
  assert.doesNotMatch(workflow, /wsl\.exe wslpath/);
});
