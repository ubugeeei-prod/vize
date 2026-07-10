import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readJson<T>(relativePath: string): T {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf-8")) as T;
}

function readText(relativePath: string): string {
  return fs.readFileSync(path.join(root, relativePath), "utf-8");
}

type VscodeManifest = {
  contributes?: {
    grammars?: Array<{
      embeddedLanguages?: Record<string, string>;
      language?: string;
      path?: string;
      scopeName?: string;
    }>;
    languages?: Array<{ id?: string; aliases?: string[]; extensions?: string[] }>;
  };
};

// VS Code stores per-language display name as aliases[0] and exposes the file
// associations as `extensions` (leading dot); Zed stores the display name as a
// key in `language_ids` and the associations as `path_suffixes` (no leading
// dot). This reconciles both stores so the two editors agree on the identity of
// each Vue language, instead of re-checking either side's literal fields (those
// are already covered in editor-integrations.test.ts).
test("VS Code and Zed agree on language ids, display names, and extensions", () => {
  const vscode = readJson<VscodeManifest>("editors/vscode/package.json");
  const zedExtension = readText("editors/zed/extension.toml");
  const zedArtConfig = readText("editors/zed/languages/art-vue/config.toml");

  const vscodeLanguages = new Map(
    (vscode.contributes?.languages ?? []).map((language) => [language.id, language]),
  );

  // art-vue identity parity: VS Code id/alias/extension vs Zed id-map/path_suffix.
  const vscodeArt = vscodeLanguages.get("art-vue");
  assert.ok(vscodeArt, "vscode should declare the art-vue language");
  assert.ok(
    vscodeArt.aliases?.includes("Art Vue"),
    "vscode art-vue should expose the 'Art Vue' display alias",
  );
  assert.deepEqual(vscodeArt.extensions, [".art.vue"]);

  // Zed maps the display name 'Art Vue' to the id 'art-vue'.
  assert.match(zedExtension, /^"Art Vue" = "art-vue"$/m);
  // The VS Code `.art.vue` association equals the Zed `art.vue` suffix (Zed
  // omits the leading dot), so they describe the same file pattern.
  assert.equal(vscodeArt.extensions?.[0], ".art.vue");
  assert.match(zedArtConfig, /^path_suffixes = \["art\.vue"\]$/m);
  assert.equal(`.${"art.vue"}`, vscodeArt.extensions?.[0]);

  // Vue identity parity: VS Code id/alias vs Zed id-map.
  const vscodeVue = vscodeLanguages.get("vue");
  assert.ok(vscodeVue, "vscode should declare the vue language");
  assert.ok(vscodeVue.aliases?.includes("Vue"), "vscode vue should expose the 'Vue' display alias");
  assert.deepEqual(vscodeVue.extensions, [".vue"]);
  assert.match(zedExtension, /^"Vue" = "vue"$/m);

  // Zed formats art-vue with the shared Vue prettier parser, keeping formatting
  // behavior consistent with the VS Code extension's Vue handling.
  assert.match(zedArtConfig, /^prettier_parser_name = "vue"$/m);
});

// The Vue TextMate grammar embeds sub-languages by `include`-ing their root
// scope (source.* / text.*); VS Code only highlights an embedded region if the
// same scope is also declared in the grammar contribution's embeddedLanguages
// map. A scope that is included but unmapped (or mapped but never included)
// silently breaks highlighting, so the minimum agreed embed set must appear in
// BOTH. editor-integrations.test.ts checks individual embeddedLanguages values
// but never reconciles them against the grammar's include graph.
test("scopeName / TextMate embed scopes align with declared embeddedLanguages", () => {
  const vscode = readJson<VscodeManifest>("editors/vscode/package.json");
  const vueGrammarContribution = (vscode.contributes?.grammars ?? []).find(
    (grammar) => grammar.language === "vue",
  );
  assert.ok(vueGrammarContribution, "vscode should contribute a vue grammar");
  assert.equal(vueGrammarContribution.scopeName, "source.vue");

  const grammar = readJson<unknown>("editors/vscode/syntaxes/vue.tmLanguage.json");
  const grammarIncludes = new Set<string>();
  const walk = (node: unknown): void => {
    if (Array.isArray(node)) {
      for (const child of node) walk(child);
      return;
    }
    if (node != null && typeof node === "object") {
      const record = node as Record<string, unknown>;
      const include = record.include;
      // Strip any `#fragment` so `source.ts#expression` counts as `source.ts`.
      if (typeof include === "string" && /^(?:source|text)\./.test(include)) {
        grammarIncludes.add(include.split("#")[0]);
      }
      for (const value of Object.values(record)) walk(value);
    }
  };
  walk(grammar);

  const embeddedLanguages = vueGrammarContribution.embeddedLanguages ?? {};
  const requiredEmbedScopes = [
    "source.ts",
    "source.tsx",
    "source.css.scss",
    "source.json",
    "text.pug",
    "source.graphql",
  ].sort();

  const includedSubset = requiredEmbedScopes.filter((scope) => grammarIncludes.has(scope)).sort();
  const mappedSubset = requiredEmbedScopes.filter((scope) => scope in embeddedLanguages).sort();

  // Every required scope is reachable from the grammar's include graph AND has a
  // declared editor language mapping.
  assert.deepEqual(includedSubset, requiredEmbedScopes);
  assert.deepEqual(mappedSubset, requiredEmbedScopes);
});
