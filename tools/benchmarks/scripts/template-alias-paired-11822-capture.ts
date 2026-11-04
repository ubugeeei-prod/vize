import { copyFileSync, mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

// Consumer Node cannot strip TypeScript from node_modules. Keep this byte-for-byte
// runtime asset with consumer fixtures, outside repository-tool retirement.
export const mapperCaptureAsset = fileURLToPath(
  new URL("tests/fixtures/mapper-protocol/capture.mjs", new URL("../../../", import.meta.url)),
);

export function installMapperCapture(
  packageRoot: string,
  captureDirectory: string,
  command: readonly string[],
): string[] {
  mkdirSync(captureDirectory);
  const runtime = join(packageRoot, "capture.mjs");
  copyFileSync(mapperCaptureAsset, runtime);
  writeFileSync(
    join(packageRoot, "capture-command.json"),
    JSON.stringify(command, null, 2) + "\n",
    {
      flag: "wx",
    },
  );
  return [process.execPath, runtime];
}
