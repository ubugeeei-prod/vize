import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

void test("actual whole browser controls exit failed and close their server when Chromium is missing", async () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vuefes-real-browser-refusal-"));
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const childEnv = { ...process.env, PLAYWRIGHT_BROWSERS_PATH: directory };
  delete childEnv.NODE_TEST_CONTEXT;
  const child = spawn(
    process.execPath,
    ["--test", "tools/support/compat/vuefes/passive-failure-sidecar.browser.test.mjs"],
    {
      cwd: root,
      env: childEnv,
      detached: process.platform !== "win32",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  const output = [];
  child.stdout.on("data", (chunk) => output.push(chunk));
  child.stderr.on("data", (chunk) => output.push(chunk));
  let expired = false;
  const timer = setTimeout(() => {
    expired = true;
    if (process.platform === "win32") child.kill();
    else if (typeof child.pid === "number") process.kill(-child.pid, "SIGTERM");
  }, 5_000);
  try {
    const code = await new Promise((resolve, reject) => {
      child.once("error", reject);
      child.once("close", resolve);
    });
    const completeOutput = Buffer.concat(output).toString();
    assert.equal(expired, false, completeOutput);
    assert.equal(code, 1, completeOutput);
    assert.match(completeOutput, /Executable doesn't exist/);
    assert.match(completeOutput, /fail 2/);
    assert.doesNotMatch(completeOutput, /cancelled 1/);
  } finally {
    clearTimeout(timer);
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
