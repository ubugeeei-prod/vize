import assert from "node:assert/strict";
import { loadConfig } from "vize/config";
await assert.rejects(loadConfig(process.cwd()), {
  message: "oxc-transform: deliberately unavailable binding",
});
