import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readJson<T>(relativePath: string): T {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf-8")) as T;
}

test("editor adapters consistently route Vue documents to the Vize LSP", () => {
  const vscodeManifest = readJson<{
    activationEvents?: string[];
    contributes?: {
      typescriptServerPlugins?: unknown;
    };
  }>("editors/vscode/package.json");
  for (const language of ["vue", "art-vue", "html"]) {
    assert.ok(
      vscodeManifest.activationEvents?.includes(`onLanguage:${language}`),
      `missing VS Code activation for ${language}`,
    );
  }
  assert.equal(vscodeManifest.contributes?.typescriptServerPlugins, undefined);

  const nvimConfig = fs.readFileSync(path.join(root, "editors/nvim/lua/vize/config.lua"), "utf-8");
  assert.match(nvimConfig, /cmd = \{ "vize", "lsp" \}/);
  assert.match(nvimConfig, /filetypes = \{ "vue", "art-vue" \}/);
  assert.match(nvimConfig, /init_options = profiles\.recommended/);
  assert.match(nvimConfig, /lint = \{\s*lint = true,\s*\}/);
  assert.match(nvimConfig, /off = \{\}/);
  assert.match(nvimConfig, /editor = true/);
  assert.match(nvimConfig, /ecosystem = true/);
  assert.match(nvimConfig, /lint = true/);
  assert.match(nvimConfig, /typecheck = true/);

  const vimConfig = fs.readFileSync(path.join(root, "editors/vim/autoload/vize.vim"), "utf-8");
  assert.match(vimConfig, /'cmd': \['vize', 'lsp'\]/);
  assert.match(vimConfig, /'allowlist': \['vue', 'art-vue'\]/);
  assert.match(vimConfig, /'initialization_options': s:profiles\.recommended/);
  assert.match(vimConfig, /'lint': \{'lint': v:true\}/);
  assert.match(vimConfig, /'off': \{\}/);
  assert.match(vimConfig, /'editor': v:true/);
  assert.match(vimConfig, /'ecosystem': v:true/);
  assert.match(vimConfig, /'lint': v:true/);
  assert.match(vimConfig, /'typecheck': v:true/);

  const helixConfig = fs.readFileSync(path.join(root, "editors/helix/languages.toml"), "utf-8");
  assert.match(helixConfig, /\[language-server\.vize\][\s\S]*command = "vize"/);
  assert.match(helixConfig, /\[language-server\.vize\][\s\S]*args = \["lsp"\]/);
  assert.match(helixConfig, /\[language-server\.vize\.config\][\s\S]*editor = true/);
  assert.match(helixConfig, /\[language-server\.vize\.config\][\s\S]*ecosystem = true/);
  assert.match(helixConfig, /\[language-server\.vize\.config\][\s\S]*lint = true/);
  assert.match(helixConfig, /\[language-server\.vize\.config\][\s\S]*typecheck = true/);
  assert.match(helixConfig, /name = "vue"[\s\S]*language-servers = \["vize"\]/);
  assert.match(helixConfig, /name = "art-vue"[\s\S]*language-id = "art-vue"/);

  const emacsConfig = fs.readFileSync(path.join(root, "editors/emacs/vize.el"), "utf-8");
  assert.match(emacsConfig, /defcustom vize-eglot-command '\("vize" "lsp"\)/);
  assert.match(emacsConfig, /defcustom vize-eglot-profile 'recommended/);
  assert.match(emacsConfig, /vue-mode vue-ts-mode web-mode vize-vue-mode vize-art-vue-mode/);
  assert.match(emacsConfig, /\(lint \. \(:lint t\)\)/);
  assert.match(emacsConfig, /\(off \. nil\)/);
  assert.match(emacsConfig, /recommended \. \(:editor t :ecosystem t :lint t :typecheck t\)/);

  const zedManifest = fs.readFileSync(path.join(root, "editors/zed/extension.toml"), "utf-8");
  assert.match(zedManifest, /\[language_servers\.vize\][\s\S]*languages = \["Vue", "Art Vue"\]/);
  assert.match(zedManifest, /"Vue" = "vue"/);
  assert.match(zedManifest, /"Art Vue" = "art-vue"/);

  const zedSource = fs.readFileSync(path.join(root, "editors/zed/src/lib.rs"), "utf-8");
  assert.match(zedSource, /const SERVER_BINARY: &'static str = "vize"/);
  assert.match(zedSource, /unwrap_or_else\(\|\| vec!\["lsp"\.to_string\(\)\]\)/);
  assert.match(zedSource, /recommended_initialization_options/);
  assert.match(zedSource, /"editor": true/);
  assert.match(zedSource, /"ecosystem": true/);
  assert.match(zedSource, /"lint": true/);
  assert.match(zedSource, /"typecheck": true/);
});
