import assert from "node:assert/strict";
import { textFile } from "../shared/files.ts";
assert.match(
  textFile("scheduler-evidence/carrier.log"),
  /test result: ok[.] 1 passed; 0 failed; 0 ignored/u,
);
