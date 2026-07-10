import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { repoRoot, runMoonScript } from "./_helpers/moonbit.ts";

const commandsRoot = path.join(repoRoot, "tools", "moon", "cmd");

function collectMoonCommandNames(directory: string): string[] {
  const names: string[] = [];
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const fullPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      names.push(...collectMoonCommandNames(fullPath));
      continue;
    }
    if (entry.name !== "main.mbt") {
      continue;
    }
    names.push(path.relative(commandsRoot, path.dirname(fullPath)).split(path.sep).join("/"));
  }
  return names.sort();
}

test("all MoonBit command packages compile without warnings", () => {
  const scriptNames = collectMoonCommandNames(commandsRoot);
  assert.ok(scriptNames.length > 0, "expected repository MoonBit command packages");

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
