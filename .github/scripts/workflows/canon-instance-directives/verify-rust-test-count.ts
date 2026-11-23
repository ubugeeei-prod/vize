import assert from "node:assert/strict";
import { join } from "node:path";
import { requiredEnv, textFile } from "../shared/files.ts";
assert.ok(
  textFile(join(requiredEnv("RUNNER_TEMP"), "instance-directives/rust-tests.txt")).includes(
    "13 passed; 0 failed; 0 ignored;",
  ),
);
