import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { verifyEmitExportSurface } from "./support/sdk-emit-export-surface.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
for (const mode of ["esm", "commonjs", "direct"] as const) {
  void test(`the ${mode} SDK preserves its full public surface and refuses private emit helpers`, () => {
    verifyEmitExportSurface(root, mode);
  });
}
