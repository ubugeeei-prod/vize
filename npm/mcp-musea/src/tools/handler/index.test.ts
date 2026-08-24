import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { ErrorCode, McpError } from "@modelcontextprotocol/sdk/types.js";
import { findArtFiles } from "../../scanner.ts";
import type { ServerContext } from "../../types.ts";
import { handleToolCall } from "./index.ts";

test("handleToolCall validates unknown tool names before loading native binding", async () => {
  const ctx: ServerContext = {
    projectRoot: process.cwd(),
    loadNative() {
      throw new Error("native binding should not load");
    },
    scanArtFiles: async () => new Map(),
    resolveTokensPath: async () => null,
  };

  await assert.rejects(
    handleToolCall(ctx, "unknown_tool", {}),
    (error) =>
      error instanceof McpError &&
      error.code === ErrorCode.MethodNotFound &&
      /Unknown tool: unknown_tool/.test(error.message),
  );
});

test("scanner matches root and nested art files while excluding directories", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "musea-mcp-scan-"));

  try {
    fs.mkdirSync(path.join(root, "src", "nested"), { recursive: true });
    fs.mkdirSync(path.join(root, "node_modules", "pkg"), { recursive: true });
    fs.writeFileSync(path.join(root, "Root.art.vue"), "");
    fs.writeFileSync(path.join(root, "src", "nested", "Card.art.vue"), "");
    fs.writeFileSync(path.join(root, "node_modules", "pkg", "Ignored.art.vue"), "");

    const files = await findArtFiles(root, ["**/*.art.vue"], ["node_modules/**"]);
    const relativeFiles = files
      .map((file) => path.relative(root, file).replaceAll(path.sep, "/"))
      .sort((left, right) => left.localeCompare(right));

    assert.deepEqual(relativeFiles, ["Root.art.vue", "src/nested/Card.art.vue"]);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("scanner skips symbolic links that leave the project root", async () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "musea-mcp-symlink-"));
  const root = path.join(tempDir, "project");
  const outside = path.join(tempDir, "outside");

  try {
    fs.mkdirSync(path.join(root, "src"), { recursive: true });
    fs.mkdirSync(outside, { recursive: true });
    fs.writeFileSync(path.join(root, "src", "Card.art.vue"), "");
    fs.writeFileSync(path.join(outside, "Secret.art.vue"), "secret");
    fs.symlinkSync(path.join(outside, "Secret.art.vue"), path.join(root, "Leak.art.vue"));
    fs.symlinkSync(outside, path.join(root, "linked"), "dir");

    const files = await findArtFiles(root, ["**/*.art.vue"], ["node_modules/**"]);
    const relativeFiles = files
      .map((file) => path.relative(root, file).replaceAll(path.sep, "/"))
      .sort((left, right) => left.localeCompare(right));

    assert.deepEqual(relativeFiles, ["src/Card.art.vue"]);
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});
