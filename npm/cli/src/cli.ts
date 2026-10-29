import { createRequire } from "node:module";

import { runInitCli } from "./init.js";
import { runSetupCli } from "./setup.js";

const require = createRequire(import.meta.url);

function fail(error: unknown): void {
  const message = error instanceof Error ? error.message : String(error);
  process.stderr.write(`[vize] ${message}\n`);
  process.exitCode = 1;
}

try {
  const args = process.argv.slice(2);
  if (args[0] === "setup") {
    runSetupCli(args.slice(1));
  } else if (args[0] === "init") {
    // `init` prompts, so it is the one command that has to be async. Failures
    // land in the same reporter as the synchronous commands.
    runInitCli(args.slice(1)).catch(fail);
  } else {
    const native = require("@vizejs/native") as typeof import("@vizejs/native");
    native.runCli(args);
  }
} catch (error) {
  fail(error);
}
