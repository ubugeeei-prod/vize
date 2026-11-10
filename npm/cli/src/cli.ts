import { createRequire } from "node:module";

import {
  answerContentMapperInitialize,
  ContentMapperFramingError,
} from "./content-mapper-launch.js";
import { runInitCli } from "./init.js";
import { assertMatchingNativeVersion } from "./native-version-guard.js";
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
  } else if (args[0] === "content-mapper") {
    assertMatchingNativeVersion();
    // Reply before the addon loads; the host's `initialize` wait is fixed (#12530).
    if (answerContentMapperInitialize()) {
      const native = require("@vizejs/native") as typeof import("@vizejs/native");
      native.runCli([...args, "--initialized"]);
    }
  } else {
    assertMatchingNativeVersion();
    const native = require("@vizejs/native") as typeof import("@vizejs/native");
    native.runCli(args);
  }
} catch (error) {
  if (error instanceof ContentMapperFramingError) {
    // The native server reports a broken stream the same way.
    process.stderr.write(`vize content-mapper: ${error.message}\n`);
    process.exitCode = 1;
  } else {
    fail(error);
  }
}
