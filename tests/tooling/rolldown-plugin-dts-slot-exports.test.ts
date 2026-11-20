import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { build } from "tsdown";

const root = fileURLToPath(new URL("../../", import.meta.url));

void test("rolldown-plugin-dts resolves forwarded slot helpers", async () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-slot-exports-"));
  const outDir = path.join(temporary, "dist");
  try {
    await build({
      clean: true,
      cwd: root,
      deps: { neverBundle: [/^vue$/u] },
      dts: { emitDtsOnly: true },
      entry: ["npm/cli/virtual-ts.d.ts"],
      failOnWarn: true,
      format: "esm",
      logLevel: "silent",
      outDir,
      platform: "neutral",
      tsconfig: false,
    });

    const bundled = fs.readFileSync(path.join(outDir, "virtual-ts.d.ts"), "utf8");
    assert.match(bundled, /\$SlotsFactory/u);
  } finally {
    fs.rmSync(temporary, { recursive: true });
  }
});
