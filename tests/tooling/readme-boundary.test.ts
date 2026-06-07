import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("README stays a compact project entry point", () => {
  const readme = readRepoFile("README.md");

  for (const section of ["## What Is Vize?", "## Credits", "## License"]) {
    assert.match(readme, new RegExp(`^${escapeRegExp(section)}$`, "m"), section);
  }

  for (const detailedSection of [
    "## Static Analysis",
    "## Compiler Configuration",
    "## Oxlint Integration",
    "## Musea Component Gallery",
    "## Editor Integration",
  ]) {
    assert.doesNotMatch(readme, new RegExp(`^${escapeRegExp(detailedSection)}$`, "m"));
  }
});

test("README funnels readers to the documentation", () => {
  const readme = readRepoFile("README.md");

  // The README is a thin entry point: it links to the live docs site, not to raw
  // repo files.
  assert.match(readme, /https:\/\/vizejs\.dev/);
  assert.match(readme, /https:\/\/vizejs\.dev\/getting-started/);
});

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
