import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(relativePath: string): string {
  return fs.readFileSync(path.join(root, relativePath), "utf-8");
}

test("emacs vize.el defines the eglot command default and customizable profile", () => {
  const el = readRepoFile("npm/emacs-vize/vize.el");

  // The LSP launch command defaults to running the `vize lsp` subcommand.
  assert.match(el, /\(defcustom\s+vize-eglot-command\s+'\("vize"\s+"lsp"\)/);

  // The default feature profile is `lint`.
  assert.match(el, /\(defcustom\s+vize-eglot-profile\s+'lint\b/);

  // The profile is a customizable choice exposing lint / recommended / off.
  assert.match(
    el,
    /:type\s+'\(choice\b[\s\S]*?vize-eglot-profile|defcustom\s+vize-eglot-profile[\s\S]*?:type\s+'\(choice/,
  );
  assert.match(el, /\(const[^)]*\blint\b\)/);
  assert.match(el, /\(const[^)]*\brecommended\b\)/);
  assert.match(el, /\(const[^)]*\boff\b\)/);
});

test("emacs vize.el maps each profile to its initialization options", () => {
  const el = readRepoFile("npm/emacs-vize/vize.el");

  // The profiles are defined in a single alist constant.
  assert.match(el, /\(defconst\s+vize--profiles\b/);

  // lint => (:lint t)
  assert.match(el, /\(lint\s*\.\s*\(:lint\s+t\)\)/);

  // off => nil
  assert.match(el, /\(off\s*\.\s*nil\)/);

  // recommended => (:editor t :ecosystem t :lint t :typecheck t)
  assert.match(
    el,
    /\(recommended\s*\.\s*\(:editor\s+t\s+:ecosystem\s+t\s+:lint\s+t\s+:typecheck\s+t\)\)/,
  );
});

test("emacs vize.el associates the .vue and .art.vue file patterns with derived modes", () => {
  const el = readRepoFile("npm/emacs-vize/vize.el");

  // auto-mode-alist: "\.vue\'" => vize-vue-mode
  assert.match(el, /add-to-list\s+'auto-mode-alist\s+'\("\\\\\.vue\\\\'"\s*\.\s*vize-vue-mode\)/);

  // auto-mode-alist: "\.art\.vue\'" => vize-art-vue-mode
  assert.match(
    el,
    /add-to-list\s+'auto-mode-alist\s+'\("\\\\\.art\\\\\.vue\\\\'"\s*\.\s*vize-art-vue-mode\)/,
  );
});

test("emacs vize.el defines vue / art-vue fallback modes deriving from prog-mode", () => {
  const el = readRepoFile("npm/emacs-vize/vize.el");

  assert.match(el, /\(define-derived-mode\s+vize-vue-mode\s+prog-mode\b/);
  assert.match(el, /\(define-derived-mode\s+vize-art-vue-mode\s+prog-mode\b/);
});

test("emacs vize.el setup registers the server with eglot-server-programs", () => {
  const el = readRepoFile("npm/emacs-vize/vize.el");

  assert.match(el, /\(defun\s+vize-setup-eglot\b/);
  assert.match(el, /add-to-list\s+'eglot-server-programs/);
});
