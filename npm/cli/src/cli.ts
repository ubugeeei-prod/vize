import { createRequire } from "node:module";

import { runSetupCli } from "./setup.ts";

const require = createRequire(import.meta.url);

try {
  const args = process.argv.slice(2);
  if (args[0] === "setup") {
    runSetupCli(args.slice(1));
  } else {
    const native = require("@vizejs/native") as typeof import("@vizejs/native");
    native.runCli(args);
  }
} catch (error) {
  const message = error instanceof Error ? error.message : String(error);
  process.stderr.write(`[vize] ${message}\n`);
  process.exitCode = 1;
}
