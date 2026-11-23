import assert from "node:assert/strict";
import { join } from "node:path";
import { requiredEnv, textFile } from "../shared/files.ts";

const root = requiredEnv("LINTER_CONTROL_DIRECTORY");
const selected = textFile(join(root, "linter-control.list.stdout"))
  .split(/\r?\n/u)
  .filter((line) => line.endsWith(": test"));
assert.deepEqual(selected, [requiredEnv("LINTER_CONTROL_SELECTOR") + ": test"]);
assert.match(
  textFile(join(root, "linter-control.stdout")),
  /test result: ok[.] 1 passed; 0 failed; 0 ignored;/u,
);
