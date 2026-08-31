import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { ErrorCode, McpError } from "@modelcontextprotocol/sdk/types.js";
import { resolveConfiguredTokensPath } from "./tokens-path.ts";

test("resolveConfiguredTokensPath accepts an in-project tokens directory", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "musea-mcp-tokens-"));
  try {
    const tokensDir = path.join(root, "tokens");
    fs.mkdirSync(tokensDir);
    fs.writeFileSync(path.join(tokensDir, "color.json"), '{"color":{"brand":{"value":"#111"}}}');

    assert.equal(await resolveConfiguredTokensPath(root), tokensDir);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("resolveConfiguredTokensPath ignores a tokens symlink that leaves the project", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "musea-mcp-tokens-link-"));
  const outside = path.join(path.dirname(root), `${path.basename(root)}-secret`);

  try {
    fs.mkdirSync(outside);
    fs.writeFileSync(
      path.join(outside, "secret.json"),
      '{"secret":{"leak":{"value":"should-not-appear"}}}',
    );
    fs.symlinkSync(outside, path.join(root, "tokens"));

    assert.equal(await resolveConfiguredTokensPath(root), null);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
    fs.rmSync(outside, { recursive: true, force: true });
  }
});

test("resolveConfiguredTokensPath rejects an explicit tokensPath outside the project", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "musea-mcp-tokens-explicit-"));
  const outside = path.join(path.dirname(root), `${path.basename(root)}-secret.json`);

  try {
    fs.writeFileSync(outside, '{"secret":{"leak":{"value":"nope"}}}');

    await assert.rejects(
      resolveConfiguredTokensPath(root, outside),
      (error) =>
        error instanceof McpError &&
        error.code === ErrorCode.InvalidParams &&
        /tokensPath must stay inside the project root/.test(error.message),
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
    fs.rmSync(outside, { force: true });
  }
});
