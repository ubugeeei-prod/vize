import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { ErrorCode, McpError } from "@modelcontextprotocol/sdk/types.js";
import { resolveProjectPath } from "./musea.ts";
import { assertVueSourcePath, isVueSourcePath } from "./vue-source-path.ts";

test("isVueSourcePath rejects a .vue symlink whose realpath is not a Vue file", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "musea-mcp-vue-source-"));

  try {
    const secret = path.join(root, ".env");
    const decoy = path.join(root, "Evil.vue");
    fs.writeFileSync(secret, "SECRET=1\n");
    fs.symlinkSync(secret, decoy);

    assert.equal(isVueSourcePath(path.join(root, "Missing.vue")), true);
    assert.equal(isVueSourcePath(decoy), false);
    const resolved = resolveProjectPath(root, "Evil.vue", "componentPath");
    assert.throws(
      () => assertVueSourcePath(resolved, "componentPath"),
      (error: unknown) =>
        error instanceof McpError &&
        error.code === ErrorCode.InvalidParams &&
        /componentPath must be a \.vue file/.test(error.message),
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
