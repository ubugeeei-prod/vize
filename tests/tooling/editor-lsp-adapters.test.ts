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
      typescriptServerPlugins?: Array<{
        enableForWorkspaceTypeScriptVersions?: boolean;
        name?: string;
      }>;
    };
  }>("editors/vscode/package.json");
  for (const language of ["vue", "art-vue", "html"]) {
    assert.ok(
      vscodeManifest.activationEvents?.includes(`onLanguage:${language}`),
      `missing VS Code activation for ${language}`,
    );
  }
  assert.deepEqual(vscodeManifest.contributes?.typescriptServerPlugins, [
    {
      enableForWorkspaceTypeScriptVersions: true,
      name: "@vizejs/typescript-vue-plugin",
    },
  ]);

  const nvimConfig = fs.readFileSync(path.join(root, "editors/nvim/lua/vize/config.lua"), "utf-8");
  assert.match(nvimConfig, /cmd = \{ "vize", "lsp" \}/);
  assert.match(nvimConfig, /filetypes = \{ "vue", "art-vue" \}/);
  assert.match(nvimConfig, /init_options = vim\.empty_dict\(\)/);
  assert.match(nvimConfig, /lint = \{\s*lint = true,\s*\}/);
  assert.match(nvimConfig, /off = \{\}/);
  assert.match(nvimConfig, /editor = true/);
  assert.match(nvimConfig, /ecosystem = true/);
  assert.match(nvimConfig, /lint = true/);
  assert.match(nvimConfig, /typecheck = true/);

  const vimConfig = fs.readFileSync(path.join(root, "editors/vim/autoload/vize.vim"), "utf-8");
  assert.match(vimConfig, /'cmd': \['vize', 'lsp'\]/);
  assert.match(vimConfig, /'allowlist': \['vue', 'art-vue'\]/);
  assert.match(vimConfig, /'initialization_options': \{\}/);
  assert.match(vimConfig, /'lint': \{'lint': v:true\}/);
  assert.match(vimConfig, /'off': \{\}/);
  assert.match(vimConfig, /'editor': v:true/);
  assert.match(vimConfig, /'ecosystem': v:true/);
  assert.match(vimConfig, /'lint': v:true/);
  assert.match(vimConfig, /'typecheck': v:true/);

  const helixConfig = fs.readFileSync(path.join(root, "editors/helix/languages.toml"), "utf-8");
  assert.match(helixConfig, /\[language-server\.vize\][\s\S]*command = "vize"/);
  assert.match(helixConfig, /\[language-server\.vize\][\s\S]*args = \["lsp"\]/);
  assert.doesNotMatch(helixConfig, /\[language-server\.vize\.config\]/);
  assert.match(helixConfig, /name = "vue"[\s\S]*language-servers = \["vize"\]/);
  assert.match(helixConfig, /name = "art-vue"[\s\S]*language-id = "art-vue"/);

  const emacsConfig = fs.readFileSync(path.join(root, "editors/emacs/vize.el"), "utf-8");
  assert.match(emacsConfig, /defcustom vize-eglot-command '\("vize" "lsp"\)/);
  assert.match(emacsConfig, /defcustom vize-eglot-profile nil/);
  assert.match(emacsConfig, /vue-mode vue-ts-mode web-mode vize-vue-mode vize-art-vue-mode/);
  assert.match(emacsConfig, /\(lint \. \(:lint t\)\)/);
  assert.match(emacsConfig, /\(off \. nil\)/);
  assert.match(emacsConfig, /recommended \. \(:editor t :ecosystem t :lint t :typecheck t\)/);

  const zedManifest = fs.readFileSync(path.join(root, "editors/zed/extension.toml"), "utf-8");
  assert.match(zedManifest, /\[language_servers\.vize\][\s\S]*languages = \["Vue.js", "Art Vue"\]/);
  assert.match(zedManifest, /"Vue.js" = "vue"/);
  assert.match(zedManifest, /"Art Vue" = "art-vue"/);

  const zedSource = fs.readFileSync(path.join(root, "editors/zed/src/lib.rs"), "utf-8");
  assert.match(zedSource, /const SERVER_BINARY: &'static str = "vize"/);
  assert.match(zedSource, /unwrap_or_else\(\|\| vec!\["lsp"\.to_string\(\)\]\)/);
  assert.match(zedSource, /settings\.initialization_options/);
  assert.doesNotMatch(zedSource, /read_text_file/);
  const zedProfile = fs.readFileSync(
    path.join(root, "editors/zed/src/initialization_options.rs"),
    "utf-8",
  );
  assert.match(zedProfile, /explicit\.unwrap_or_else\(\|\| serde_json::json!\(\{\}\)\)/);
});
