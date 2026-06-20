import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(relativePath: string): string {
  return fs.readFileSync(path.join(root, relativePath), "utf-8");
}

test("nvim ftdetect maps .vue -> vue and .art.vue -> art-vue", () => {
  const ftdetect = readRepoFile("npm/editor/nvim/ftdetect/vize.lua");

  // *.vue files are detected as the "vue" filetype.
  assert.match(ftdetect, /pattern\s*=\s*["']\*\.vue["']/);
  assert.match(ftdetect, /vim\.bo\.filetype\s*=\s*["']vue["']/);

  // *.art.vue files override to the "art-vue" filetype.
  assert.match(ftdetect, /pattern\s*=\s*["']\*\.art\.vue["']/);
  assert.match(ftdetect, /vim\.bo\.filetype\s*=\s*["']art-vue["']/);

  // Detection is wired through BufNewFile/BufRead autocommands.
  assert.match(
    ftdetect,
    /nvim_create_autocmd\(\s*\{\s*["']BufNewFile["']\s*,\s*["']BufRead["']\s*\}/,
  );
});

test("nvim config default cmd, filetypes and root_markers declare the canonical LSP wiring", () => {
  const config = readRepoFile("npm/editor/nvim/lua/vize/config.lua");

  // Default launch command is `vize lsp`.
  assert.match(config, /cmd\s*=\s*\{\s*["']vize["']\s*,\s*["']lsp["']\s*\}/);

  // Both Vue dialects are registered as default filetypes.
  assert.match(config, /filetypes\s*=\s*\{\s*["']vue["']\s*,\s*["']art-vue["']\s*\}/);

  // Root markers are declared with vize.config.pkl FIRST, then json/package.json/.git.
  assert.match(
    config,
    /root_markers\s*=\s*\{\s*["']vize\.config\.pkl["']\s*,\s*["']vize\.config\.json["']\s*,\s*["']package\.json["']\s*,\s*["']\.git["']\s*\}/,
  );
});

test("nvim config defines the lint, recommended and off profiles, defaulting init_options to lint", () => {
  const config = readRepoFile("npm/editor/nvim/lua/vize/config.lua");

  // All three named profiles are defined inside the profiles table.
  assert.match(config, /\blint\s*=\s*\{/);
  assert.match(config, /\brecommended\s*=\s*\{/);
  assert.match(config, /\boff\s*=\s*\{\s*\}/);

  // Default init_options points at the lint profile.
  assert.match(config, /init_options\s*=\s*profiles\.lint/);
});

test("nvim plugin entrypoint wires the plugin via guard + setup command", () => {
  const plugin = readRepoFile("npm/editor/nvim/plugin/vize.lua");

  // Load-guard prevents double sourcing.
  assert.match(plugin, /vim\.g\.loaded_vize/);

  // A user command exposes setup, which calls into the vize module's setup().
  assert.match(plugin, /nvim_create_user_command\(\s*["']VizeSetup["']/);
  assert.match(plugin, /require\(\s*["']vize["']\s*\)\.setup\(\)/);
});
