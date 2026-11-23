import assert from "node:assert/strict";
import { join } from "node:path";
import { requiredEnv, textFile } from "../shared/files.ts";

const lines = textFile(
  join(requiredEnv("RUNNER_TEMP"), "jsx-authored-native/napi-dto.list.stdout"),
).split(/\r?\n/u);
assert.deepEqual(
  lines.filter((line) => line.endsWith(": test")),
  ["napi::jsx::tests::jsx_compile_result_supports_ssr_output: test"],
);
