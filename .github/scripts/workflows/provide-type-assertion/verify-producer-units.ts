import assert from "node:assert/strict";
import { join } from "node:path";
import { requiredEnv, textFile } from "../shared/files.ts";
for (const name of ["producer-unit", "classifier-unit"])
  assert.match(
    textFile(join(requiredEnv("RUNNER_TEMP"), "provide-assertion-packets", name + ".stdout")),
    /test result: ok[.] 1 passed; 0 failed;/u,
  );
