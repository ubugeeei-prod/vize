import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const explicitEntrypoints = [
  "tools/fixtures/glyph-corpus-waiver-audit.mjs",
  "tools/fixtures/patina-rule-map.mjs",
  "tools/fixtures/real-project-surface-verdict.mjs",
  "tools/github/release-platforms.mjs",
  "tools/github/require-needs-success.mjs",
  "tools/github/semver-change-marker.mjs",
];

test("tool command surface is Rust Script first", () => {
  const source = readRepoFile("tools/rust/verify-layout.rs");
  assert.match(source, /rust-script tools: verified/);
  assert.match(source, /tools\/commands/);

  const wrappers = collectFiles(path.join(root, "tools", "commands")).filter((file) =>
    file.endsWith(".rs"),
  );
  assert.ok(wrappers.length > 0, "expected Rust Script command wrappers");

  for (const wrapper of wrappers) {
    const relative = path.relative(root, wrapper);
    const text = fs.readFileSync(wrapper, "utf8");
    assert.match(text, /^#!\/usr\/bin\/env rust-script\n/, relative);
    assert.match(text, /legacy_command::run\(/, relative);
    assert.doesNotMatch(relative, /-vize\//, "editor command buckets use product-neutral names");
  }
});

test("legacy tool entrypoints are covered by Rust Script wrappers", () => {
  const expected = legacyEntrypoints()
    .map((legacy) => rustCommandPath(legacy))
    .sort();
  const wrappers = collectFiles(path.join(root, "tools", "commands"))
    .filter((file) => file.endsWith(".rs"))
    .map((file) => path.relative(root, file).split(path.sep).join("/"))
    .sort();

  assert.deepEqual(wrappers, expected);
});

function legacyEntrypoints(): string[] {
  return collectFiles(path.join(root, "tools"))
    .filter((file) => {
      const relative = path.relative(root, file).split(path.sep).join("/");
      if (relative.startsWith("tools/commands/")) return false;
      if (relative.startsWith("tools/rust/")) return false;
      if (relative.startsWith("tools/moon/.mooncakes/")) return false;
      if (!/\.(?:mjs|js|ts|sh)$/.test(relative)) return false;
      return (
        fs.readFileSync(file, "utf8").startsWith("#!") || explicitEntrypoints.includes(relative)
      );
    })
    .map((file) => path.relative(root, file).split(path.sep).join("/"))
    .sort();
}

function rustCommandPath(legacy: string): string {
  const buckets: Record<string, string> = {
    "ai-fix-agent.mjs": "agents",
    davinci: "davinci",
    "editor-e2e": "editors/e2e",
    "emacs-vize": "editors/emacs",
    fixtures: "fixtures",
    fuzz: "ci/fuzz",
    github: "ci/github",
    "helix-vize": "editors/helix",
    npm: "release/npm",
    "nvim-vize": "editors/neovim",
    release: "release",
    "vim-vize": "editors/vim",
    "vscode-vize": "editors/vscode",
    "zed-vize": "editors/zed",
  };
  const parts = legacy.slice("tools/".length).split("/");
  const file = parts.pop();
  assert.ok(file);
  const stem = file.replace(/\.(?:mjs|js|ts|sh)$/, "");
  const bucket = buckets[parts[0] ?? file];
  assert.ok(bucket, `missing Rust Script bucket for ${legacy}`);
  return `tools/commands/${bucket}/${stem}.rs`;
}

function collectFiles(dir: string): string[] {
  const files: string[] = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const absolute = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...collectFiles(absolute));
    } else if (entry.isFile()) {
      files.push(absolute);
    }
  }
  return files;
}

function readRepoFile(filePath: string): string {
  return fs.readFileSync(path.join(root, filePath), "utf8");
}
