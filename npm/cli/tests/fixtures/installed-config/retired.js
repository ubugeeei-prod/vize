import assert from "node:assert/strict";
import { loadConfig } from "vize/config";
await assert.rejects(loadConfig(process.cwd()), /vize\.config\.\* is no longer supported/);
await assert.rejects(
  loadConfig(process.cwd(), { configFile: process.argv[2] }),
  /vize\.config\.\* is no longer supported/,
);
