import assert from "node:assert/strict";
import { loadConfig } from "vize/config";
await assert.rejects(loadConfig(process.cwd()), (error) =>
  error.message.endsWith("@vizejs/native: deliberately unavailable binding"),
);
