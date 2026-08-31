import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { ErrorCode, McpError } from "@modelcontextprotocol/sdk/types.js";
import { resolveProjectVueFile } from "./vue-source-path.ts";

test("resolveProjectVueFile accepts a regular in-project Vue file", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "musea-mcp-vue-"));
  try {
    const vuePath = path.join(root, "Button.vue");
    fs.writeFileSync(vuePath, "<template><button /></template>\n");

    assert.equal(resolveProjectVueFile(root, "Button.vue", "componentPath"), vuePath);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("resolveProjectVueFile rejects a non-vue path before reading", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "musea-mcp-vue-env-"));
  try {
    fs.writeFileSync(path.join(root, ".env"), "SECRET=1\n");

    assert.throws(
      () => resolveProjectVueFile(root, ".env", "componentPath"),
      (error) =>
        error instanceof McpError &&
        error.code === ErrorCode.InvalidParams &&
        /componentPath must be a \.vue file/.test(error.message),
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("resolveProjectVueFile rejects a .vue symlink that targets a non-vue file", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "musea-mcp-vue-link-"));
  try {
    const envPath = path.join(root, ".env");
    fs.writeFileSync(envPath, "SECRET=1\n");
    fs.symlinkSync(envPath, path.join(root, "Evil.vue"));

    assert.throws(
      () => resolveProjectVueFile(root, "Evil.vue", "componentPath"),
      (error) =>
        error instanceof McpError &&
        error.code === ErrorCode.InvalidParams &&
        /componentPath must be a \.vue file/.test(error.message),
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
