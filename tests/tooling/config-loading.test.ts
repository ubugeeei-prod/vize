import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { loadConfig } from "../../npm/vize/src/config.ts";

test("vize.config.ts loads through the public config API", async () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-ts-"));

  try {
    fs.writeFileSync(
      path.join(tempDir, "vize.config.ts"),
      [
        "type Command = 'serve' | 'build';",
        "",
        "export default ({ mode, command }: { mode: string; command: Command }) => ({",
        "  compiler: {",
        "    sourceMap: mode === 'production',",
        "  },",
        "  formatter: {",
        "    lineWidth: command === 'build' ? 100 : 80,",
        "  },",
        "});",
        "",
      ].join("\n"),
    );

    const config = await loadConfig(tempDir, {
      env: { command: "build", mode: "production" },
      mode: "root",
    });

    assert.equal(config?.compiler?.sourceMap, true);
    assert.equal(config?.formatter?.lineWidth, 100);
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});
