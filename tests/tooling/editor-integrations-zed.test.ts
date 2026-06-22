import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(relativePath: string): string {
  return fs.readFileSync(path.join(root, relativePath), "utf-8");
}

function workspaceVersion(): string {
  const version = readRepoFile("Cargo.toml").match(/^version = "(.+)"$/m)?.[1];
  assert.ok(version);
  return version;
}

test("zed extension.toml declares vize server, language ids, grammar pin and version", () => {
  const manifest = readRepoFile("editors/zed/extension.toml");

  // Top-level extension identity and schema.
  assert.match(manifest, /^id = "vize"$/m);
  assert.match(manifest, /^schema_version = 1$/m);

  // Language server registration for both Vue dialects.
  assert.match(manifest, /^\[language_servers\.vize\]$/m);
  assert.match(manifest, /^languages = \["Vue", "Art Vue"\]$/m);

  // Display-name -> language-id mapping.
  assert.match(manifest, /^\[language_servers\.vize\.language_ids\]$/m);
  assert.match(manifest, /^"Vue" = "vue"$/m);
  assert.match(manifest, /^"Art Vue" = "art-vue"$/m);

  // art-vue grammar pinned to a tree-sitter repo at a full 40-char commit sha.
  assert.match(manifest, /^\[grammars\.art-vue\]$/m);
  const grammarRepository = manifest.match(
    /^\[grammars\.art-vue\]\n(?:.*\n)*?repository = "([^"]+)"$/m,
  )?.[1];
  assert.equal(grammarRepository, "https://github.com/tree-sitter-grammars/tree-sitter-vue");
  const grammarCommit = manifest.match(/^commit = "([0-9a-f]{40})"$/m)?.[1];
  assert.ok(grammarCommit, "art-vue grammar must pin a full 40-char commit sha");

  // Extension version tracks the workspace Cargo version.
  const declaredVersion = manifest.match(/^version = "(.+)"$/m)?.[1];
  assert.equal(declaredVersion, workspaceVersion());
});

test("zed art-vue config.toml declares comments, brackets, autoclose and tailwind opt-in", () => {
  const config = readRepoFile("editors/zed/languages/art-vue/config.toml");

  // Identity already covered by editor-integrations; assert the new behavioral knobs.
  assert.match(config, /^block_comment = \["<!-- ", " -->"\]$/m);
  assert.match(config, /^autoclose_before = ";:\.,=\}\]\)>"$/m);
  assert.match(config, /^code_fence_block_name = "vue"$/m);
  assert.match(config, /^word_characters = \["-"\]$/m);

  // Bracket pairs: braces/brackets/parens auto-close and add a newline.
  for (const [open, close] of [
    ["{", "}"],
    ["[", "]"],
    ["(", ")"],
  ] as const) {
    const escapedOpen = open.replace(/[[\](){}]/g, "\\$&");
    const escapedClose = close.replace(/[[\](){}]/g, "\\$&");
    assert.match(
      config,
      new RegExp(
        `\\{ start = "${escapedOpen}", end = "${escapedClose}", close = true, newline = true \\}`,
      ),
      `bracket pair ${open}${close} should auto-close with newline`,
    );
  }

  // Angle bracket pair must NOT auto-close (close = false) and is guarded by not_in.
  assert.match(
    config,
    /\{ start = "<", end = ">", close = false, newline = true, not_in = \[\n\s*"string",\n\s*"comment",\n\s*\] \}/,
  );

  // Three quote pairs declared with auto-close and a not_in guard, none add a newline.
  // In TOML the double quote is itself escaped as \" inside the quoted string.
  const quotePairs = [
    ['\\\\"', "double"],
    ["'", "single"],
    ["`", "backtick"],
  ] as const;
  for (const [quote, label] of quotePairs) {
    assert.match(
      config,
      new RegExp(`\\{ start = "${quote}", end = "${quote}", close = true, newline = false`),
      `${label} quote pair should auto-close without newline`,
    );
  }

  // Tailwind language server is opted into at the top level.
  assert.match(config, /^scope_opt_in_language_servers = \["tailwindcss-language-server"\]$/m);

  // JSX tag auto-close wiring for the tree-sitter-vue node names.
  assert.match(config, /^\[jsx_tag_auto_close\]$/m);
  assert.match(config, /^open_tag_node_name = "start_tag"$/m);
  assert.match(config, /^close_tag_node_name = "end_tag"$/m);
});

test("zed art-vue injections.scm routes every embedded region to the right language", () => {
  const injections = readRepoFile("editors/zed/languages/art-vue/injections.scm");

  // <script lang="ts"> -> typescript.
  assert.match(injections, /#eq\? @_ts "ts"[\s\S]*"language" "typescript"/);

  // {{ }} interpolation -> typescript.
  assert.match(injections, /interpolation[\s\S]*typescript/);

  // v- directive attribute values -> typescript.
  assert.match(injections, /directive_attribute[\s\S]*typescript/);

  // <style> -> css.
  assert.match(injections, /style_element[\s\S]*"css"/);

  // <template lang="pug"> -> pug.
  assert.match(injections, /template_element[\s\S]*"pug"/);

  // HTML comments highlighted as html.
  assert.match(injections, /\(comment\)[\s\S]*"html"/);

  // <script lang="tsx"/"jsx"> handled via Zed's built-in tsx grammar.
  assert.match(injections, /#any-of\? @language "tsx" "jsx"/);
});
