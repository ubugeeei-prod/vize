import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}

test("app e2e Playwright config keeps CI runs guarded and debuggable", () => {
  const config = readRepoFile("tests", "app", "playwright.config.ts");

  assert.match(config, /forbidOnly:\s*!!process\.env\.CI/);
  assert.match(config, /retries:\s*process\.env\.CI \? 2 : 0/);
  assert.match(
    config,
    /reporter:\s*process\.env\.CI \? \[\["list"\], \["html", \{ open: "never" \}\]\] : "list"/,
  );
  assert.match(config, /screenshot:\s*"only-on-failure"/);
  assert.match(config, /trace:\s*"retain-on-failure"/);
  assert.match(config, /video:\s*"retain-on-failure"/);
});
