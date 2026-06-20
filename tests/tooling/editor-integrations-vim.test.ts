import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(relativePath: string): string {
  return fs.readFileSync(path.join(root, relativePath), "utf-8");
}

test("vim ftdetect maps *.vue -> vue and *.art.vue -> art-vue inside an augroup", () => {
  const ftdetect = readRepoFile("npm/editor/vim/ftdetect/vize.vim");

  // Detection lives inside a named augroup that clears itself first so re-sourcing
  // does not stack duplicate autocommands.
  assert.match(ftdetect, /^augroup vize_filetypes$/m);
  assert.match(ftdetect, /^\s*autocmd!\s*$/m);
  assert.match(ftdetect, /^augroup END$/m);

  // *.vue files are detected as the "vue" filetype. The real implementation uses
  // `setlocal filetype=vue` (not `setf`/`set filetype`), so assert what it actually does.
  assert.match(ftdetect, /autocmd\s+BufNewFile,BufRead\s+\*\.vue\s+setlocal\s+filetype=vue/);

  // *.art.vue files override to the "art-vue" filetype.
  assert.match(
    ftdetect,
    /autocmd\s+BufNewFile,BufRead\s+\*\.art\.vue\s+setlocal\s+filetype=art-vue/,
  );
});

test("vim ftdetect autocommands are scoped only to *.vue / *.art.vue (no clobbering)", () => {
  const ftdetect = readRepoFile("npm/editor/vim/ftdetect/vize.vim");

  // Collect every real autocommand definition (excluding the bare `autocmd!` clear).
  const autocmds = ftdetect
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => /^autocmd\s+\S/.test(line));

  // Exactly the two Vue-dialect detectors, nothing more.
  assert.equal(autocmds.length, 2);

  // Every autocommand pattern must target a *.vue or *.art.vue glob — never a bare
  // catch-all like `*` or an unrelated extension.
  for (const cmd of autocmds) {
    assert.match(cmd, /\*(\.art)?\.vue\s/, `autocommand must be scoped to a vue glob: ${cmd}`);
  }

  // Guard: no autocommand silently grabs `*` (which would clobber every filetype).
  assert.doesNotMatch(ftdetect, /BufNewFile,BufRead\s+\*\s/);
});

test("vim autoload wires the vize LSP server registration (vize + lsp tokens)", () => {
  const autoload = readRepoFile("npm/editor/vim/autoload/vize.vim");

  // The default launch command is `vize lsp`.
  assert.match(autoload, /'cmd':\s*\['vize',\s*'lsp'\]/);

  // setup() registers the server through vim-lsp's lsp#register_server.
  assert.match(autoload, /lsp#register_server\(/);

  // The server is registered under the canonical "vize" name and is gated on both
  // Vue dialects via the allowlist.
  assert.match(autoload, /'name':\s*'vize'/);
  assert.match(autoload, /'allowlist':\s*\['vue',\s*'art-vue'\]/);
});

test("vim plugin entrypoint guards double-sourcing and exposes the VizeSetup command", () => {
  const plugin = readRepoFile("npm/editor/vim/plugin/vize.vim");

  // Load-guard prevents the plugin from being sourced twice.
  assert.match(plugin, /exists\('g:loaded_vize'\)/);
  assert.match(plugin, /let g:loaded_vize = 1/);

  // A user command exposes setup, which calls into the vize#setup() autoload function.
  assert.match(plugin, /command!\s+VizeSetup\s+call\s+vize#setup\(\)/);
});
