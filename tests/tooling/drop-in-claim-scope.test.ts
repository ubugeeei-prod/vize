/**
 * Keeps the `@vizejs/vite-plugin` drop-in claim scoped (#3227).
 *
 * "Drop-in replacement for `@vitejs/plugin-vue`" is true for Vue 3 SFCs and not
 * for the legacy dialects, the non-Vite bundlers, or the full plugin-option
 * surface. These tests pin the boundary in the English docs (locale pages are
 * machine-generated from them by `docs/scripts/i18n/generate.mjs`) and in the
 * release notes, so the claim cannot widen silently.
 */

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const generatedLocaleDirs = new Set(["fr", "i18n", "ja", "pt-BR", "zh-CN"]);
const dropInClaim = "drop-in replacement for `@vitejs/plugin-vue`";
const scopeQualifier = " on Vue 3 SFCs";
const trackingIssue = "https://github.com/ubugeeei-prod/vize/issues/3227";

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}

/** Every English (source-of-truth) markdown page, plus the repo entry point. */
function englishMarkdownFiles(): string[] {
  const files = [path.join(root, "README.md")];
  const walk = (dir: string) => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const entryPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        if (dir === path.join(root, "docs", "content") && generatedLocaleDirs.has(entry.name)) {
          continue;
        }
        walk(entryPath);
      } else if (entry.name.endsWith(".md")) {
        files.push(entryPath);
      }
    }
  };
  walk(path.join(root, "docs", "content"));
  walk(path.join(root, "docs", "release"));
  return files;
}

test("every English drop-in claim is scoped to Vue 3 SFCs", () => {
  const unscoped: string[] = [];

  for (const file of englishMarkdownFiles()) {
    const content = fs.readFileSync(file, "utf8");
    let index = content.indexOf(dropInClaim);
    while (index !== -1) {
      const after = content.slice(index + dropInClaim.length);
      if (!after.startsWith(scopeQualifier)) {
        unscoped.push(`${path.relative(root, file)}: ${JSON.stringify(after.slice(0, 40))}`);
      }
      index = content.indexOf(dropInClaim, index + 1);
    }

    assert.doesNotMatch(
      content,
      /100\s*%\s*(?:drop-in|compatible)/iu,
      `${path.relative(root, file)} must not claim unqualified 100% compatibility`,
    );
  }

  assert.deepEqual(
    unscoped,
    [],
    `each drop-in claim must be immediately qualified with "${scopeQualifier.trim()}"`,
  );
});

test("the Vite plugin guide documents the drop-in boundary", () => {
  const guide = readRepoFile("docs", "content", "guide", "vite-plugin.md");
  const section = guide.split(/^## /mu).find((block) => block.startsWith("Drop-in Scope"));
  assert.ok(section, "the guide must carry a `## Drop-in Scope` section");
  // Line wrapping is the formatter's business, so assert on prose, not layout.
  const prose = section.replace(/\s+/gu, " ");

  // In scope: Vue 3 SFCs authored either way.
  assert.match(prose, /Vue 3 single-file components/u);
  assert.match(prose, /Options API/u);

  // Incubating: the legacy dialects stay opt-in and non-invasive.
  assert.match(prose, /\*\*Incubating\*\*/u);
  assert.match(prose, /`vue\.version: "2"` \/ `"2\.7"`/u);
  assert.match(prose, /does not compile `\.vue` files in those modes/u);

  // Out of scope: the non-Vite bundlers, pointed at their own experimental page.
  assert.match(prose, /\*\*Out of scope\*\*/u);
  assert.match(prose, /webpack/u);
  assert.match(prose, /\]\(\.\/unplugin\.md\)/u);

  // Incomplete: plugin-option parity, tracked by its own issue.
  assert.match(prose, /`include`, `exclude`, and `isProduction` are honored today/u);
  assert.ok(prose.includes(trackingIssue), "the option-parity gap must link its tracking issue");
});

test("the philosophy page defers to the documented drop-in boundary", () => {
  const philosophy = readRepoFile("docs", "content", "philosophy.md");

  assert.match(philosophy, /Vue 2 and 2\.7 \(`vue\.version`\) are incubating and opt-in/u);
  assert.match(philosophy, /webpack is not part of the drop-in claim/u);
  assert.match(philosophy, /\]\(\.\/guide\/vite-plugin\.md#drop-in-scope\)/u);
});

test("release notes lead with the drop-in scope", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const createRelease = workflow.slice(workflow.indexOf("name: Create Release"));
  assert.notEqual(createRelease, "", "the release workflow must create a GitHub release");

  const inputs = createRelease.slice(0, createRelease.indexOf("files:"));
  const bodyPath = /body_path: (\S+)/u.exec(inputs);
  assert.ok(bodyPath, "the release body must be authored, not only generated");
  assert.match(
    inputs,
    /generate_release_notes: true/u,
    "the authored body prepends the generated notes",
  );

  const body = readRepoFile(...bodyPath[1].split("/"));
  assert.match(body, /drop-in replacement for `@vitejs\/plugin-vue` on \*\*Vue 3 SFCs\*\*/u);
  assert.match(body, /incubating and opt-in/u);
  assert.match(body, /outside the drop-in claim/u);
  assert.ok(body.includes(trackingIssue), "the release body must link the parity issue");
});
