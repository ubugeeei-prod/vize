import { writeSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { inspect } from "node:util";
import { downloadAttempts } from "./download-attempts.mjs";

const version = process.env.VIZE_TEST_VSCODE_VERSION ?? "1.107.1";
if (process.argv[2] === "--attempt") {
  try {
    const { downloadAndUnzipVSCode } = await import("@vscode/test-electron");
    // Keep upstream retries, SHA256 validation, extraction and exact version.
    const executable = await downloadAndUnzipVSCode({ version, timeout: 30_000 });
    writeSync(1, `Verified VS Code ${version}: ${executable}\n`);
    process.exit(0);
  } catch (error) {
    writeSync(2, `VS Code ${version} download/extraction failed after upstream retries:\n`);
    writeSync(2, `${inspect(error, { depth: 6 })}\n`);
    // Upstream's failed streams must not keep this attempt alive indefinitely.
    process.exit(1);
  }
} else {
  process.exitCode = await downloadAttempts(process.execPath, [
    fileURLToPath(import.meta.url),
    "--attempt",
  ]);
}
