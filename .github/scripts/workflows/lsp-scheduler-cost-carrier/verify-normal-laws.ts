import assert from "node:assert/strict";
import { textFile } from "../shared/files.ts";
assert.match(
  textFile("scheduler-evidence/normal-laws.log"),
  /test result: ok[.] 37 passed; 0 failed; 0 ignored/u,
);
