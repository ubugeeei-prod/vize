import assert from "node:assert/strict";
import { resolveConfigExport } from "vize/config";
await assert.rejects(resolveConfigExport({ linter: { enabled: true } }), {
  message: "@vizejs/native: deliberately unavailable binding",
});
