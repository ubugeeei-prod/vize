import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("README stays a compact project entry point", () => {
  const readme = readRepoFile("README.md");

  for (const section of [
    "## What Is Vize?",
    "## Quick Start",
    "## Documentation Map",
    "## Local Development",
    "## Community",
    "## Credits",
    "## License",
  ]) {
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

test("README documentation map points to the detailed docs", () => {
  const readme = readRepoFile("README.md");

  for (const link of [
    "./docs/content/getting-started.md",
    "./docs/content/guide/vite-plugin.md",
    "./docs/content/guide/cli.md",
    "./docs/content/guide/static-analysis.md",
    "./docs/content/guide/configuration.md",
    "./docs/content/rules/index.md",
    "./docs/content/guide/musea.md",
    "./docs/content/integrations/vscode.md",
    "./docs/content/architecture/overview.md",
    "./docs/content/architecture/language-engineering-practices.md",
    "./docs/release/production-readiness.md",
  ]) {
    assert.match(readme, new RegExp(escapeRegExp(link)), link);
    assert.ok(fs.existsSync(path.join(root, link)), `${link} should exist`);
  }
});

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
