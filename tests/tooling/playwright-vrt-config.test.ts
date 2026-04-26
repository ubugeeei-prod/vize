import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}

test("VRT Playwright configs keep failure artifacts and CI-readable output", () => {
  for (const configPath of [
    ["playground", "playwright.config.ts"],
    ["examples", "vite-musea", "playwright.config.ts"],
  ]) {
    const config = readRepoFile(...configPath);

    assert.match(config, /reporter:\s*\[\["list"\], \["html", \{ open: "never" \}\]\]/);
    assert.match(config, /screenshot:\s*"only-on-failure"/);
    assert.match(config, /trace:\s*"on-first-retry"/);
    assert.match(config, /video:\s*"retain-on-failure"/);
  }
});
