import { downloadAndUnzipVSCode } from "@vscode/test-electron";
import { inspect } from "node:util";

const version = process.env.VIZE_TEST_VSCODE_VERSION ?? "1.107.1";
try {
  // Upstream bounds idle reads and retries; the dedicated Actions step bounds
  // the entire download/extraction, independently of the actual editor suite.
  const executable = await downloadAndUnzipVSCode({ version, timeout: 30_000 });
  console.log(`Verified VS Code ${version}: ${executable}`);
} catch (error) {
  console.error(`VS Code ${version} download/extraction failed after upstream retries:`);
  console.error(inspect(error, { depth: 6 }));
  process.exitCode = 1;
}
