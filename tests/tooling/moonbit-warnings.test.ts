import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { repoRoot, runMoonScript } from "./_helpers/moonbit.ts";

const scriptsRoot = path.join(repoRoot, "tools", "moon", "scripts");

function collectMoonScriptNames(directory: string): string[] {
  const names: string[] = [];
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const fullPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      names.push(...collectMoonScriptNames(fullPath));
      continue;
    }
    if (!entry.name.endsWith(".mbtx")) {
      continue;
    }
    names.push(
      path
        .relative(scriptsRoot, fullPath)
        .replace(/\.mbtx$/, "")
        .split(path.sep)
        .join("/"),
    );
  }
  return names.sort();
}

test("all MoonBit scripts compile without warnings", () => {
  const scriptNames = collectMoonScriptNames(scriptsRoot);
  assert.ok(scriptNames.length > 0, "expected repository MoonBit scripts");

  for (const scriptName of scriptNames) {
    const result = runMoonScript(scriptName, [], {
      buildOnly: true,
      denyWarn: true,
    });

    assert.equal(
      result.status,
      0,
      [
        `${scriptName} failed to compile with --deny-warn`,
        result.stderr.trim(),
        result.stdout.trim(),
      ]
        .filter(Boolean)
        .join("\n\n"),
    );
  }
});
