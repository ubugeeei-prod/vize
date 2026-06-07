import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(relativePath: string): string {
  return fs.readFileSync(path.join(root, relativePath), "utf-8");
}

test("helix languages.toml registers the vize language server with the lsp command", () => {
  const config = readRepoFile("npm/helix-vize/languages.toml");

  // Language server registration: command `vize` invoked with the `lsp` subcommand.
  assert.match(config, /^\[language-server\.vize\]$/m);
  assert.match(config, /^command = "vize"$/m);
  assert.match(config, /^args = \["lsp"\]$/m);

  // Server-level config enables linting.
  assert.match(config, /^\[language-server\.vize\.config\]$/m);
  assert.match(config, /^lint = true$/m);
});

test("helix languages.toml declares the vue language wired to the vize server", () => {
  const config = readRepoFile("npm/helix-vize/languages.toml");

  // The plain `vue` language entry: scope, file-types and language-servers.
  const vueEntry = config.match(
    /^\[\[language\]\]\nname = "vue"\n(?:.*\n)*?language-servers = \[[^\]]*\]/m,
  )?.[0];
  assert.ok(vueEntry, "languages.toml must contain a [[language]] entry named vue");
  assert.match(vueEntry, /^scope = "source\.vue"$/m);
  assert.match(vueEntry, /^file-types = \["vue"\]$/m);
  assert.match(vueEntry, /^language-servers = \["vize"\]$/m);
});

test("helix languages.toml declares the art-vue language with a glob file-type", () => {
  const config = readRepoFile("npm/helix-vize/languages.toml");

  // The `art-vue` language entry: language-id, scope, glob file-type and server.
  const artEntry = config.match(
    /^\[\[language\]\]\nname = "art-vue"\n(?:.*\n)*?language-servers = \[[^\]]*\]/m,
  )?.[0];
  assert.ok(artEntry, "languages.toml must contain a [[language]] entry named art-vue");
  assert.match(artEntry, /^language-id = "art-vue"$/m);
  assert.match(artEntry, /^scope = "source\.art-vue"$/m);
  assert.match(artEntry, /^file-types = \[\{ glob = "\*\.art\.vue" \}\]$/m);
  assert.match(artEntry, /^language-servers = \["vize"\]$/m);
});

test("helix languages.toml root markers cover vize config, package.json and .git", () => {
  const config = readRepoFile("npm/helix-vize/languages.toml");

  // Both language entries declare the same set of project root markers via `roots`.
  const rootsLines = config.match(/^roots = \[[^\]]*\]$/gm) ?? [];
  assert.equal(rootsLines.length, 2, "both vue and art-vue entries must declare roots");
  for (const roots of rootsLines) {
    assert.match(roots, /"vize\.config\.pkl"/);
    assert.match(roots, /"vize\.config\.json"/);
    assert.match(roots, /"package\.json"/);
    assert.match(roots, /"\.git"/);
  }
});
