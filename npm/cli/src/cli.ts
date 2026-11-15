import { createRequire } from "node:module";
import { randomUUID } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import {
  answerContentMapperInitialize,
  ContentMapperFramingError,
} from "./content-mapper-launch.js";
import { runInitCli } from "./init.js";
import { assertMatchingNativeVersion } from "./native-version-guard.js";
import { runSetupCli } from "./setup.js";

const require = createRequire(import.meta.url);

function runNativeCli(native: typeof import("@vizejs/native"), args: string[]): void {
  const configStdin = args.indexOf("--config-stdin");
  if (configStdin === -1 || native.supportsCliConfigStdin?.() === true) {
    native.runCli(args);
    return;
  }

  const payload = fs.readFileSync(0);
  const file = path.join(os.tmpdir(), `.vize-vp-${randomUUID()}.json`);
  try {
    fs.writeFileSync(file, payload, { flag: "wx", mode: 0o600 });
    native.runCli([
      ...args.slice(0, configStdin),
      "--config",
      file,
      ...args.slice(configStdin + 1),
    ]);
  } finally {
    fs.rmSync(file, { force: true });
  }
}

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
    const capabilityArgs = answerContentMapperInitialize();
    if (capabilityArgs !== null) {
      const native = require("@vizejs/native") as typeof import("@vizejs/native");
      native.runCli([
        ...args,
        "--initialized",
        "--reply-when-ready",
        ...capabilityArgs,
      ]);
    }
  } else {
    assertMatchingNativeVersion();
    const native = require("@vizejs/native") as typeof import("@vizejs/native");
    runNativeCli(native, args);
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
