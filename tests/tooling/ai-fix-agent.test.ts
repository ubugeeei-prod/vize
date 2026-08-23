import assert from "node:assert/strict";
import test from "node:test";

import { ensureTool, parseCommandArguments } from "../../tools/ai-fix-agent/core.mjs";

test("parseCommandArguments splits a program and arguments without a shell", () => {
  assert.deepEqual(parseCommandArguments("codex exec --full-auto"), [
    "codex",
    "exec",
    "--full-auto",
  ]);
  assert.deepEqual(parseCommandArguments(`agent --prompt "hello world"`), [
    "agent",
    "--prompt",
    "hello world",
  ]);
});

test("parseCommandArguments rejects shell metacharacters", () => {
  assert.throws(() => parseCommandArguments("codex exec && rm -rf /"), /metacharacters/);
  assert.throws(() => parseCommandArguments("codex $(id)"), /metacharacters/);
});

test("ensureTool rejects interpolated or path-like names", () => {
  assert.throws(() => ensureTool("git; id"), /invalid tool name/);
  assert.throws(() => ensureTool("/usr/bin/git"), /invalid tool name/);
  assert.throws(() => ensureTool("$(id)"), /invalid tool name/);
});
