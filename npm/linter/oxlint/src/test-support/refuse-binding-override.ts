import assert from "node:assert/strict";
import { runOriginalSfcHostMatrix } from "./original-sfc-host-matrix.ts";

const [options, key] = process.argv.slice(2);
await assert.rejects(runOriginalSfcHostMatrix(JSON.parse(options)), {
  message: `public baseline cannot inherit${key}`,
});
