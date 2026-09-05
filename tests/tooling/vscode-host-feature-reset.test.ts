import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { FEATURE_SETTING_KEYS } from "../../editors/vscode/src/extension-core.ts";

const require = createRequire(import.meta.url);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const { featureSettingKeys } =
  require("../../editors/vscode/test/suite/extension-host-fixtures.cjs") as {
    featureSettingKeys: string[];
  };

test("VS Code host smokes reset every extension feature switch before starting a server", () => {
  assert.deepEqual([...featureSettingKeys].sort(), [...FEATURE_SETTING_KEYS].sort());

  for (const relativePath of [
    "editors/vscode/test/suite/auto-insert-smoke.cjs",
    "editors/vscode/test/suite/real-server-support.cjs",
  ]) {
    const source = fs.readFileSync(path.join(root, relativePath), "utf8");
    assert.match(source, /require\("\.\/extension-host-fixtures\.cjs"\)/, relativePath);
    assert.doesNotMatch(source, /const featureSettingKeys\s*=\s*\[/, relativePath);
  }
});
