import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readText(relativePath: string): string {
  return fs.readFileSync(path.join(root, relativePath), "utf-8");
}

// The canonical language ids and the canonical LSP feature-profile names that
// every editor integration must agree on. editor-integrations-consistency.test.ts
// already reconciles VS Code <-> Zed for *language identity*; this file extends
// the invariant to the OTHER editors and adds the *profile-name* invariant.
const CANONICAL_LANGUAGE_IDS = ["art-vue", "vue"];
const CANONICAL_PROFILES = new Set(["lint", "recommended", "off"]);

// Each editor that declares Vue languages, with a parser that extracts the set
// of canonical ids ("vue" / "art-vue") it references. We collect a per-editor
// set and assert it contains BOTH ids, so no editor silently drops one variant.
type LanguageDeclaration = {
  editor: string;
  file: string;
  extract: (text: string) => Set<string>;
};

function collectCanonicalIds(text: string): Set<string> {
  const found = new Set<string>();
  // Match "art-vue" before "vue" by checking the longer token first; we scan for
  // each canonical id as a whole token wrapped in common delimiters (quotes,
  // brackets, equals, whitespace) so "vue" inside "art-vue" is not double counted
  // for the bare "vue" id.
  for (const id of CANONICAL_LANGUAGE_IDS) {
    const escaped = id.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    // A token boundary that is NOT a hyphen/word char on the left (so "art-vue"
    // does not satisfy the bare "vue" probe) and not a word char on the right.
    const re = new RegExp(`(?<![\\w-])${escaped}(?![\\w-])`);
    if (re.test(text)) {
      found.add(id);
    }
  }
  return found;
}

const LANGUAGE_DECLARATIONS: LanguageDeclaration[] = [
  // VS Code declares ids in contributes.languages[].id; the JSON is read as text
  // and probed for the canonical id tokens.
  {
    editor: "vscode",
    file: "npm/editor/vscode/package.json",
    extract: collectCanonicalIds,
  },
  // Zed maps display name -> id in [language_servers.vize.language_ids].
  {
    editor: "zed",
    file: "npm/editor/zed/extension.toml",
    extract: collectCanonicalIds,
  },
  // Neovim lists filetypes = { "vue", "art-vue" }.
  {
    editor: "neovim",
    file: "npm/editor/nvim/lua/vize/config.lua",
    extract: collectCanonicalIds,
  },
  // Emacs declares auto-mode-alist patterns and derived modes for both variants.
  // Unlike the other editors it never writes the bare "art-vue" token: the id
  // surface is the derived mode name fragment (vize-vue-mode / vize-art-vue-mode)
  // and the auto-mode-alist file patterns (\\.vue\\' / \\.art\\.vue\\'). We map
  // those Emacs-native conventions onto the canonical id set.
  {
    editor: "emacs",
    file: "npm/editor/emacs/vize.el",
    extract: (text) => {
      const found = new Set<string>();
      // vize-vue-mode (vue) requires "vize-vue-mode" with no extra leading "-art".
      if (/\bvize-vue-mode\b/.test(text) && /\\\\\.vue\\\\'/.test(text)) {
        found.add("vue");
      }
      // vize-art-vue-mode (art-vue) + the \\.art\\.vue\\' auto-mode pattern.
      if (/\bvize-art-vue-mode\b/.test(text) && /\\\\\.art\\\\\.vue\\\\'/.test(text)) {
        found.add("art-vue");
      }
      return found;
    },
  },
  // Helix declares [[language]] name = "vue" / "art-vue".
  {
    editor: "helix",
    file: "npm/editor/helix/languages.toml",
    extract: collectCanonicalIds,
  },
  // Vim's ftdetect sets filetype=vue / filetype=art-vue.
  {
    editor: "vim",
    file: "npm/editor/vim/ftdetect/vize.vim",
    extract: collectCanonicalIds,
  },
];

// Each editor that declares an LSP feature-PROFILE table, with a parser that
// extracts the set of profile names it defines. VS Code and Zed/Helix/Vim's
// ftdetect deliberately do NOT carry a lint/recommended/off profile table, so
// they are excluded from the profile invariant (see PROFILE_LESS_EDITORS).
type ProfileDeclaration = {
  editor: string;
  file: string;
  extract: (text: string) => string[];
};

const PROFILE_DECLARATIONS: ProfileDeclaration[] = [
  // Neovim: `local profiles = { lint = {...}, off = {}, recommended = {...} }`.
  {
    editor: "neovim",
    file: "npm/editor/nvim/lua/vize/config.lua",
    extract: (text) => {
      const block = /local profiles = \{([\s\S]*?)\n\}/.exec(text);
      assert.ok(block, "neovim config.lua should declare a `local profiles` table");
      // Only top-level keys: exactly two leading spaces of indentation inside the
      // table (nested feature flags like `lint = true` sit at four spaces).
      const names = [...block[1].matchAll(/^ {2}([A-Za-z_][\w]*)\s*=/gm)].map((m) => m[1]);
      return names;
    },
  },
  // Emacs: `(defconst vize--profiles '((lint . ...) (off . nil) (recommended . ...)))`.
  {
    editor: "emacs",
    file: "npm/editor/emacs/vize.el",
    extract: (text) => {
      const block = /\(defconst vize--profiles\s*'\(([\s\S]*?)\)\s*"Vize/.exec(text);
      assert.ok(block, "emacs vize.el should declare a vize--profiles constant");
      const names = [...block[1].matchAll(/\(([A-Za-z][\w-]*)\s*\./g)].map((m) => m[1]);
      return names;
    },
  },
];

// Vim keeps its profile table in autoload/vize.vim (not ftdetect), so it also
// carries the profile invariant; include it explicitly here.
PROFILE_DECLARATIONS.push({
  editor: "vim",
  file: "npm/editor/vim/autoload/vize.vim",
  extract: (text) => {
    const block = /let s:profiles = \{([\s\S]*?)\n\s*\\ \}/.exec(text);
    assert.ok(block, "vim autoload/vize.vim should declare an s:profiles dictionary");
    // Only top-level keys: VimL line-continuation `\ 'name':` (single space after
    // the backslash); nested feature flags use `\   'name':` (three spaces).
    const names = [...block[1].matchAll(/^\s*\\ '([A-Za-z][\w-]*)'\s*:/gm)].map((m) => m[1]);
    return names;
  },
});

// Editors that legitimately declare NO profile table: VS Code (it exposes
// per-feature `*.enable` settings plus Recommended/Lint-only *commands*, not a
// canonical profile table), Zed and Helix (single `lint = true` config, no
// named profiles).
const PROFILE_LESS_EDITORS = new Set(["vscode", "zed", "helix"]);

test("every editor that declares Vue languages references both canonical ids", () => {
  const perEditor = LANGUAGE_DECLARATIONS.map((decl) => ({
    editor: decl.editor,
    ids: [...decl.extract(readText(decl.file))].sort(),
  }));

  // Every listed editor must declare BOTH "vue" and "art-vue".
  for (const { editor, ids } of perEditor) {
    assert.deepEqual(
      ids,
      CANONICAL_LANGUAGE_IDS,
      `${editor} should reference exactly the canonical ids vue & art-vue`,
    );
  }

  // Sanity: all six editors were checked.
  assert.equal(perEditor.length, 6);
  assert.deepEqual(perEditor.map((entry) => entry.editor).sort(), [
    "emacs",
    "helix",
    "neovim",
    "vim",
    "vscode",
    "zed",
  ]);
});

test("every editor that declares LSP profiles uses exactly lint/recommended/off", () => {
  const perEditor = PROFILE_DECLARATIONS.map((decl) => ({
    editor: decl.editor,
    file: decl.file,
    profiles: decl.extract(readText(decl.file)),
  }));

  // At least the editors that actually carry profile tables are present.
  assert.deepEqual(perEditor.map((entry) => entry.editor).sort(), ["emacs", "neovim", "vim"]);

  for (const { editor, profiles } of perEditor) {
    const profileSet = new Set(profiles);

    // No rogue / duplicate names: the parsed list has no duplicates...
    assert.equal(
      profiles.length,
      profileSet.size,
      `${editor} should not declare a duplicate profile name`,
    );

    // ...and every declared profile is one of the canonical three.
    for (const name of profileSet) {
      assert.ok(
        CANONICAL_PROFILES.has(name),
        `${editor} declares rogue profile "${name}" (allowed: lint/recommended/off)`,
      );
    }

    // The profile set is a subset of the canonical set...
    assert.ok(
      [...profileSet].every((name) => CANONICAL_PROFILES.has(name)),
      `${editor} profile set must be a subset of {lint, recommended, off}`,
    );

    // ...and must include at least "lint" (the default/baseline profile).
    assert.ok(profileSet.has("lint"), `${editor} must declare the baseline "lint" profile`);
  }

  // The three editors that DO declare profiles each declare all three canonical
  // names (current behavior: nvim/emacs/vim are fully aligned).
  for (const { editor, profiles } of perEditor) {
    assert.deepEqual(
      [...new Set(profiles)].sort(),
      ["lint", "off", "recommended"],
      `${editor} should declare all three canonical profiles`,
    );
  }
});

test("profile-less editors are intentionally excluded from the profile invariant", () => {
  // Guard against accidental drift: editors we excluded from the profile check
  // must NOT be ones that actually ship a profile table. We re-derive which
  // editors carry profiles and confirm the exclusion list is disjoint from them.
  const profileEditors = new Set(PROFILE_DECLARATIONS.map((decl) => decl.editor));
  for (const excluded of PROFILE_LESS_EDITORS) {
    assert.ok(
      !profileEditors.has(excluded),
      `${excluded} is excluded from profiles yet declares a profile table`,
    );
  }

  // VS Code exposes the profile *concept* as commands rather than a canonical
  // lint/recommended/off table, so it is excluded from the profile-name check
  // but the recommended & lint-only command surface still exists.
  const vscode = readText("npm/editor/vscode/package.json");
  assert.match(vscode, /"command":\s*"vize\.enableRecommendedProfile"/);
  assert.match(vscode, /"command":\s*"vize\.enableLintOnlyProfile"/);
  // VS Code has no "off" profile command — confirming why it is excluded from
  // the exact lint/recommended/off table invariant.
  assert.doesNotMatch(vscode, /"command":\s*"vize\.enableOffProfile"/);
});
