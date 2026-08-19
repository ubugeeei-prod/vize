const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vscode = require("vscode");
const { extensionId } = require("./extension-host-fixtures.cjs");

/**
 * The TextMate grammar contributions, asserted from the installed extension.
 *
 * This suite is purely static — it never starts a language server — and it is
 * the longest single scenario in the extension-host run because every embedded
 * language, directive capture, and custom block gets its own assertion. Keeping
 * it beside the lifecycle suites rather than inside them is what lets both stay
 * readable, and the grammars are read from `extension.extensionPath` so the
 * assertions describe what was packaged rather than what is in the source tree.
 */

exports.runSyntaxHighlightContributionSmoke = async function runSyntaxHighlightContributionSmoke() {
  const extension = vscode.extensions.getExtension(extensionId);
  assert.ok(extension, `missing extension: ${extensionId}`);

  const grammars = extension.packageJSON.contributes?.grammars ?? [];
  const vueGrammarContribution = grammars.find((grammar) => grammar.language === "vue");
  const artVueGrammarContribution = grammars.find((grammar) => grammar.language === "art-vue");
  assert.ok(vueGrammarContribution, "missing vue grammar contribution");
  assert.ok(artVueGrammarContribution, "missing art-vue grammar contribution");

  assert.equal(vueGrammarContribution.scopeName, "source.vue");
  assert.deepEqual(vueGrammarContribution.embeddedLanguages, {
    "source.css": "css",
    "source.css.less": "less",
    "source.css.scss": "scss",
    "source.graphql": "graphql",
    "source.js": "javascript",
    "source.js.jsx": "javascriptreact",
    "source.json": "json",
    "source.postcss": "postcss",
    "source.sass": "sass",
    "source.stylus": "stylus",
    "source.ts": "typescript",
    "source.toml": "toml",
    "source.tsx": "typescriptreact",
    "source.yaml": "yaml",
    "text.html.basic": "html",
    "text.pug": "pug",
  });
  assert.equal(artVueGrammarContribution.scopeName, "source.art-vue");
  assert.deepEqual(
    artVueGrammarContribution.embeddedLanguages,
    vueGrammarContribution.embeddedLanguages,
  );

  const vueGrammar = readGrammar(extension, vueGrammarContribution.path);
  const artVueGrammar = readGrammar(extension, artVueGrammarContribution.path);

  assert.equal(vueGrammar.scopeName, "source.vue");
  assert.deepEqual(vueGrammar.patterns, [
    { include: "#vue-comments" },
    { include: "#vue-template-pug" },
    { include: "#vue-template" },
    { include: "source.vue.script" },
    { include: "#vue-style" },
    { include: "#vue-custom-block-json" },
    { include: "#vue-custom-block-yaml" },
    { include: "#vue-custom-block-toml" },
    { include: "#vue-custom-block-graphql" },
    { include: "#vue-custom-block" },
  ]);
  assert.equal(
    vueGrammar.repository["vue-template"].beginCaptures["2"].name,
    "entity.name.tag.template.html",
  );
  assert.equal(vueGrammar.repository["vue-interpolation"].name, "meta.embedded.expression.vue");
  assert.equal(
    vueGrammar.repository["vue-template-pug"].patterns[1].contentName,
    "meta.embedded.block.pug",
  );
  assert.equal(
    vueGrammar.repository["vue-script-tsx"].patterns[1].contentName,
    "meta.embedded.block.tsx",
  );
  assert.equal(
    vueGrammar.repository["vue-script-ts"].patterns[1].contentName,
    "meta.embedded.block.typescript",
  );
  assert.equal(
    vueGrammar.repository["vue-script-jsx"].patterns[1].contentName,
    "meta.embedded.block.jsx",
  );
  assert.equal(
    vueGrammar.repository["vue-style-scss"].patterns[1].contentName,
    "meta.embedded.block.scss",
  );
  assert.equal(
    vueGrammar.repository["vue-style-less"].patterns[1].contentName,
    "meta.embedded.block.less",
  );
  assert.equal(
    vueGrammar.repository["vue-style-sass"].patterns[1].contentName,
    "meta.embedded.block.sass",
  );
  assert.equal(
    vueGrammar.repository["vue-style-stylus"].patterns[1].contentName,
    "meta.embedded.block.stylus",
  );
  assert.equal(
    vueGrammar.repository["vue-style-postcss"].patterns[1].contentName,
    "meta.embedded.block.postcss",
  );
  assert.equal(
    vueGrammar.repository["vue-style-css"].patterns[1].contentName,
    "meta.embedded.block.css",
  );
  assert.equal(
    vueGrammar.repository["vue-custom-block-json"].contentName,
    "meta.embedded.block.json",
  );
  assert.equal(
    vueGrammar.repository["vue-custom-block-yaml"].contentName,
    "meta.embedded.block.yaml",
  );
  assert.equal(
    vueGrammar.repository["vue-custom-block-toml"].contentName,
    "meta.embedded.block.toml",
  );
  assert.equal(
    vueGrammar.repository["vue-custom-block-graphql"].contentName,
    "meta.embedded.block.graphql",
  );
  assert.equal(
    vueGrammar.repository["vue-generic-attribute"].patterns[0].contentName,
    "meta.embedded.type.typescript",
  );
  assert.equal(
    vueGrammar.repository["vue-generic-attribute"].patterns[0].patterns[0].include,
    "#vue-ts-type",
  );
  assert.equal(
    vueGrammar.repository["vue-interpolation"].patterns[0].include,
    "source.ts#expression",
  );
  assert.equal(
    vueGrammar.repository["vue-directive-attributes"].patterns[0].patterns[0].include,
    "#vue-ts-expression",
  );
  assert.equal(
    vueGrammar.repository["vue-directive-attributes"].patterns[0].beginCaptures["5"].patterns[0]
      .include,
    "#vue-ts-expression",
  );
  assert.equal(
    vueGrammar.repository["vue-directive-attributes"].patterns[0].beginCaptures["1"].name,
    "keyword.control.directive.vue",
  );
  assert.equal(
    vueGrammar.repository["vue-directive-attributes"].patterns[1].beginCaptures["2"].name,
    "entity.other.attribute-name.binding.vue",
  );
  assert.equal(
    vueGrammar.repository["vue-directive-attributes"].patterns[2].beginCaptures["2"].name,
    "entity.other.attribute-name.event.vue",
  );
  assert.equal(
    vueGrammar.repository["vue-css-vbind"].beginCaptures["1"].name,
    "support.function.vue",
  );
  assert.equal(vueGrammar.repository["vue-css-vbind"].patterns[0].name, "variable.other.vue");
  assert.equal(artVueGrammar.scopeName, "source.art-vue");
  assert.deepEqual(artVueGrammar.patterns, [
    { include: "#art-comments" },
    { include: "#art-block" },
    { include: "source.vue" },
  ]);
  assert.equal(
    artVueGrammar.repository["art-block"].beginCaptures["2"].name,
    "entity.name.tag.art.vue",
  );
  assert.equal(
    artVueGrammar.repository["variant-block"].beginCaptures["2"].name,
    "entity.name.tag.variant.vue",
  );
  assert.ok(
    artVueGrammar.repository["art-template-content"].patterns.some(
      (pattern) => pattern.include === "source.vue#html-tags",
    ),
    "art template content should reuse Vue HTML tag highlighting",
  );
  assert.equal(
    artVueGrammar.repository["variant-json-attribute-values"].patterns[0].contentName,
    "meta.embedded.block.json",
  );
};

function readGrammar(extension, grammarPath) {
  return JSON.parse(fs.readFileSync(path.join(extension.extensionPath, grammarPath), "utf-8"));
}
