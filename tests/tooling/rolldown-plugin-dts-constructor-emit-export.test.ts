import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { build } from "tsdown";

const root = fileURLToPath(new URL("../../", import.meta.url));

void test("rolldown-plugin-dts links the split constructor emit helper", async () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-constructor-emit-export-"));
  const outDir = path.join(temporary, "dist");
  try {
    await build({
      clean: true,
      cwd: root,
      deps: { neverBundle: [/^vue$/u] },
      dts: { emitDtsOnly: true },
      entry: ["npm/cli/virtual-ts-emits.d.ts"],
      format: "esm",
      logLevel: "silent",
      outDir,
      platform: "neutral",
      tsconfig: false,
    });
    const bundled = fs.readFileSync(path.join(outDir, "virtual-ts-emits.d.ts"), "utf8");
    assert.match(bundled, /^type \$ConstructorEmitPropsFactory</mu);
    assert.match(bundled, /export type \{ \$EmitPropsFactory, \$NativeEmitPropsFactory \};/u);
  } finally {
    fs.rmSync(temporary, { recursive: true });
  }
});
