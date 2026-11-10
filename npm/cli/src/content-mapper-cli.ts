import { createRequire } from "node:module";

import {
  answerContentMapperInitialize,
  ContentMapperFramingError,
} from "./content-mapper-launch.js";
import { assertMatchingNativeVersion } from "./native-version-guard.js";

const require = createRequire(import.meta.url);

try {
  // Reply before package probing and addon loading; the host's `initialize`
  // wait is fixed (#12530), and neither changes these static capabilities.
  if (answerContentMapperInitialize()) {
    assertMatchingNativeVersion();
    const native = require("@vizejs/native") as typeof import("@vizejs/native");
    native.runCli(["content-mapper", "--initialized"]);
  }
} catch (error) {
  if (error instanceof ContentMapperFramingError) {
    // The native server reports a broken stream the same way.
    process.stderr.write(`vize content-mapper: ${error.message}\n`);
    process.exitCode = 1;
  } else {
    const message = error instanceof Error ? error.message : String(error);
    process.stderr.write(`[vize] ${message}\n`);
    process.exitCode = 1;
  }
}
