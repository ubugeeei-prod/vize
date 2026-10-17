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
  const ftdetect = readRepoFile("editors/nvim/ftdetect/vize.lua");

  // *.vue files are detected as the "vue" filetype.
  assert.match(ftdetect, /pattern\s*=\s*["']\*\.vue["']/);
  assert.match(ftdetect, /vim\.bo\.filetype\s*=\s*["']vue["']/);

  // *.art.vue files override to the "art-vue" filetype. The resulting buffer
  // syntax and tree-sitter parser resolution are observable behavior and are
  // asserted against a real editor in editors/nvim/test/vize_spec.lua, so they
  // are deliberately not re-asserted here as ftdetect source text.
  assert.match(ftdetect, /pattern\s*=\s*["']\*\.art\.vue["']/);
  assert.match(ftdetect, /vim\.bo\.filetype\s*=\s*["']art-vue["']/);

  // Detection is wired through BufNewFile/BufRead autocommands.
  assert.match(
    ftdetect,
    /nvim_create_autocmd\(\s*\{\s*["']BufNewFile["']\s*,\s*["']BufRead["']\s*\}/,
  );
});

test("nvim config default cmd, filetypes and root_markers declare the canonical LSP wiring", () => {
  const config = readRepoFile("editors/nvim/lua/vize/config.lua");

  // Default launch command is `vize lsp`.
  assert.match(config, /cmd\s*=\s*\{\s*["']vize["']\s*,\s*["']lsp["']\s*\}/);

  // Both Vue dialects are registered as default filetypes.
  assert.match(config, /filetypes\s*=\s*\{\s*["']vue["']\s*,\s*["']art-vue["']\s*\}/);

  const markers = config.match(/root_markers\s*=\s*\{([^}]+)\}/)?.[1] ?? "";
  for (const marker of [
    "vite.config.ts",
    "vite.config.js",
    "vite.config.mts",
    "vite.config.mjs",
    "vite.config.cts",
    "vite.config.cjs",
    "tsconfig.json",
    "package.json",
    ".git",
  ]) {
    assert.ok(markers.includes(`"${marker}"`));
  }
  assert.doesNotMatch(markers, /vize\.config/);
});

test("nvim config defines profiles and leaves feature defaults to the server", () => {
  const config = readRepoFile("editors/nvim/lua/vize/config.lua");

  // All three named profiles are defined inside the profiles table.
  assert.match(config, /\blint\s*=\s*exclusive_profile\(true\)/);
  assert.match(config, /\brecommended\s*=\s*\{/);
  assert.match(config, /\boff\s*=\s*exclusive_profile\(false\)/);

  // An empty override preserves authored project settings.
  assert.match(config, /init_options\s*=\s*vim\.empty_dict\(\)/);
});

test("nvim plugin entrypoint wires the plugin via guard + setup command", () => {
  const plugin = readRepoFile("editors/nvim/plugin/vize.lua");

  // Load-guard prevents double sourcing.
  assert.match(plugin, /vim\.g\.loaded_vize/);

  // A user command exposes setup, which calls into the vize module's setup().
  assert.match(plugin, /nvim_create_user_command\(\s*["']VizeSetup["']/);
  assert.match(plugin, /require\(\s*["']vize["']\s*\)\.setup\(\)/);
});
