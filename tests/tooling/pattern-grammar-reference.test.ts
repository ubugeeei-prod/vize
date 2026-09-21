// The TextMate contract of the patterned-template reference implementation
// (vuejs/language-tools#6207, `extensions/vscode/tests/grammar.spec.ts`), read
// from its pinned snapshot: every character of every `v-match` / `v-when`
// attribute must receive the scopes the reference grammar assigns. The two
// grammars name the attribute shell and the embedded expression differently,
// so those are compared by role; everything inside a pattern is compared by
// its exact scope names.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { patternRoot } from "./support/upstream/vue-language-tools.ts";
import { loadVueTextMateGrammar, type TextMateGrammar } from "./support/vue-textmate.ts";

const upstreamTests = path.join(patternRoot, "upstream/language-tools/extensions/vscode/tests");
const snapshot = fs.readFileSync(
  path.join(upstreamTests, "__snapshots__/grammar.spec.ts.snap"),
  "utf8",
);

type Token = { start: number; end: number; scopes: string[] };
type Line = { text: string; tokens: Token[] };
/** Consecutive characters that share a label, as `[text, label]`. */
type Runs = Array<[string, string]>;

/** `>source` lines followed by `#   ^^^ scope scope` token lines. */
function readEntry(name: string): Line[] {
  const header = `exports[\`${name} 1\`] = \`\n"`;
  const start = snapshot.indexOf(header);
  assert.notEqual(start, -1, `missing snapshot entry ${name}`);
  const body = snapshot.slice(start + header.length, snapshot.indexOf('"\n`;', start));
  const lines: Line[] = [];
  for (const raw of body.split("\n")) {
    if (raw.startsWith(">")) {
      lines.push({ text: raw.slice(1), tokens: [] });
      continue;
    }
    const token = /^#( *)(\^+) (.+)$/.exec(raw);
    assert.ok(token, `unreadable snapshot line in ${name}: ${raw}`);
    const startColumn = token[1].length;
    lines.at(-1)!.tokens.push({
      start: startColumn,
      end: startColumn + token[2].length,
      scopes: token[3].split(" "),
    });
  }
  return lines;
}

function tokenize(grammar: TextMateGrammar, texts: string[]): Line[] {
  let ruleStack: unknown = null;
  return texts.map((text) => {
    const result = grammar.tokenizeLine(text, ruleStack);
    ruleStack = result.ruleStack;
    return {
      text,
      tokens: result.tokens.map((token) => ({
        start: token.startIndex,
        end: Math.min(token.endIndex, text.length),
        scopes: token.scopes,
      })),
    };
  });
}

type Shell = {
  directive: string;
  separator: string;
  quoteBegin: string;
  quoteEnd: string;
  expression: string;
};
const referenceShell: Shell = {
  directive: "keyword.control.conditional.vue",
  separator: "punctuation.separator.key-value.html.vue",
  quoteBegin: "punctuation.definition.string.begin.html.vue",
  quoteEnd: "punctuation.definition.string.end.html.vue",
  expression: "source.ts.embedded.html.vue",
};
const vizeShell: Shell = {
  directive: "keyword.control.directive.vue",
  separator: "punctuation.separator.key-value.html",
  quoteBegin: "punctuation.definition.string.begin.html",
  quoteEnd: "punctuation.definition.string.end.html",
  expression: "meta.embedded.expression.vue",
};

function label(scopes: string[], shell: Shell): string {
  const pattern = scopes.indexOf("meta.pattern.vue");
  if (pattern !== -1) {
    const inside = scopes.slice(pattern + 1);
    if (inside.includes(shell.expression)) return "pattern expression";
    return ["pattern", ...inside].join(" ");
  }
  if (scopes.includes(shell.expression)) return "expression";
  const leaf = scopes.at(-1);
  if (leaf === shell.directive) return "directive";
  if (leaf === shell.separator) return "=";
  if (leaf === shell.quoteBegin) return "quote.begin";
  if (leaf === shell.quoteEnd) return "quote.end";
  return `unexpected ${scopes.join(" ")}`;
}

function runs(line: Line, from: number, to: number, shell: Shell): Runs {
  const result: Runs = [];
  for (let column = from; column < to; column++) {
    const token = line.tokens.find((token) => token.start <= column && column < token.end);
    assert.ok(token, `no token covers column ${column} of ${line.text}`);
    const current = label(token.scopes, shell);
    const last = result.at(-1);
    if (last?.[1] === current) last[0] += line.text[column];
    else result.push([line.text[column], current]);
  }
  return result;
}

/** `[start, end)` of every pattern directive attribute, name through closing quote. */
function attributes(text: string): Array<[number, number]> {
  return [...text.matchAll(/\bv-(?:match|when)=(["'])/g)].map((match) => {
    const open = match.index + match[0].length;
    return [match.index, text.indexOf(match[1], open) + 1];
  });
}

const { grammar } = await loadVueTextMateGrammar();

// The reference also snapshots `patterned-templates.vue` without its embedded
// grammars; that run applies no pattern rule at all, so it carries no contract.
for (const [fixture, expectedAttributes] of [
  ["patterned-templates.vue", 7],
  ["root-patterned-template.vue", 3],
] as const) {
  const entry = `embedded grammar > ${fixture}`;
  test(`${fixture} scopes pattern attributes like the reference grammar`, () => {
    const reference = readEntry(entry);
    const source = reference.map((line) => line.text);
    // The snapshot is of exactly the pinned fixture.
    assert.equal(
      `${source.join("\n")}\n`.replace(/\n+$/, "\n"),
      fs.readFileSync(path.join(upstreamTests, "embeddedGrammarFixtures", fixture), "utf8"),
    );
    const actual = tokenize(grammar, source);
    let compared = 0;
    for (const [index, line] of reference.entries()) {
      for (const [from, to] of attributes(line.text)) {
        assert.deepEqual(
          runs(actual[index], from, to, vizeShell),
          runs(line, from, to, referenceShell),
          `${entry}:${index + 1}: ${line.text.slice(from, to)}`,
        );
        compared++;
      }
    }
    assert.equal(compared, expectedAttributes);
  });
}

/** The tokens of one attribute value with their scopes, without the root scope. */
function valueRuns(source: string): Runs {
  const [line] = tokenize(grammar, ["<template>", source]).slice(1);
  const [[from, to]] = attributes(source);
  const open = source.indexOf("=", from) + 2;
  const result: Runs = [];
  for (const token of line.tokens) {
    if (token.start < open || token.end > to - 1) continue;
    const text = source.slice(token.start, token.end);
    if (text.trim() === "") continue;
    result.push([text, token.scopes.slice(1).join(" ")]);
  }
  return result;
}

test("a guard keeps nested calls and comparisons inside its parentheses", () => {
  assert.deepEqual(valueRuns(`<p v-when="[const a, ...] if (a < max(1, 2))" class="x">`), [
    ["[", "meta.pattern.vue punctuation.definition.pattern.vue"],
    ["const", "meta.pattern.vue keyword.declaration.pattern.vue"],
    ["a", "meta.pattern.vue variable.other.constant.ts"],
    [",", "meta.pattern.vue punctuation.separator.pattern.vue"],
    ["...", "meta.pattern.vue keyword.operator.rest.vue"],
    ["]", "meta.pattern.vue punctuation.definition.pattern.vue"],
    ["if", "meta.pattern.vue keyword.control.conditional.vue"],
    ["(", "meta.pattern.vue punctuation.definition.parameters.begin.ts"],
    ["a", "meta.pattern.vue meta.embedded.expression.vue variable.other.readwrite.ts"],
    ["<", "meta.pattern.vue meta.embedded.expression.vue keyword.operator.ts"],
    ["max", "meta.pattern.vue meta.embedded.expression.vue entity.name.function.ts"],
    ["(", "meta.pattern.vue meta.embedded.expression.vue keyword.operator.ts"],
    ["1", "meta.pattern.vue meta.embedded.expression.vue constant.numeric.ts"],
    [",", "meta.pattern.vue meta.embedded.expression.vue keyword.operator.ts"],
    ["2", "meta.pattern.vue meta.embedded.expression.vue constant.numeric.ts"],
    [")", "meta.pattern.vue meta.embedded.expression.vue keyword.operator.ts"],
    [")", "meta.pattern.vue punctuation.definition.parameters.end.ts"],
  ]);
});

test("literals, wildcards, entities and rejected declarations have their own scopes", () => {
  assert.deepEqual(
    valueRuns(
      `<p v-when="{ a: -42 | 0xff | 1.5e2 | 123n, b: NaN, c: _, _d: null, let e, const 日本語 }&#32;">`,
    ),
    [
      ["{", "meta.pattern.vue punctuation.definition.pattern.vue"],
      ["a", "meta.pattern.vue variable.other.readwrite.ts"],
      [":", "meta.pattern.vue punctuation.separator.pattern.vue"],
      ["-42", "meta.pattern.vue constant.numeric.ts"],
      ["|", "meta.pattern.vue keyword.operator.pattern.vue"],
      ["0xff", "meta.pattern.vue constant.numeric.ts"],
      ["|", "meta.pattern.vue keyword.operator.pattern.vue"],
      ["1.5e2", "meta.pattern.vue constant.numeric.ts"],
      ["|", "meta.pattern.vue keyword.operator.pattern.vue"],
      ["123n", "meta.pattern.vue constant.numeric.ts"],
      [",", "meta.pattern.vue punctuation.separator.pattern.vue"],
      ["b", "meta.pattern.vue variable.other.readwrite.ts"],
      [":", "meta.pattern.vue punctuation.separator.pattern.vue"],
      ["NaN", "meta.pattern.vue constant.language.ts"],
      [",", "meta.pattern.vue punctuation.separator.pattern.vue"],
      ["c", "meta.pattern.vue variable.other.readwrite.ts"],
      [":", "meta.pattern.vue punctuation.separator.pattern.vue"],
      ["_", "meta.pattern.vue constant.language.wildcard.vue"],
      [",", "meta.pattern.vue punctuation.separator.pattern.vue"],
      ["_d", "meta.pattern.vue variable.other.readwrite.ts"],
      [":", "meta.pattern.vue punctuation.separator.pattern.vue"],
      ["null", "meta.pattern.vue constant.language.ts"],
      [",", "meta.pattern.vue punctuation.separator.pattern.vue"],
      ["let", "meta.pattern.vue invalid.illegal.pattern.vue"],
      ["e", "meta.pattern.vue variable.other.readwrite.ts"],
      [",", "meta.pattern.vue punctuation.separator.pattern.vue"],
      ["const", "meta.pattern.vue keyword.declaration.pattern.vue"],
      ["日本語", "meta.pattern.vue variable.other.constant.ts"],
      ["}", "meta.pattern.vue punctuation.definition.pattern.vue"],
      ["&#32;", "meta.pattern.vue constant.character.entity.html"],
    ],
  );
});

test("a <template> arm is a pattern too, in either quote style", () => {
  for (const quote of ['"', "'"]) {
    const [line] = tokenize(grammar, [
      "<template>",
      `<template v-when=${quote}{ const id }${quote} :key="id">`,
    ]).slice(1);
    assert.deepEqual(
      line.tokens
        .map((token): [string, string] => [
          line.text.slice(token.start, token.end),
          token.scopes.slice(1).join(" "),
        ])
        .filter(([text]) => text.trim() !== ""),
      [
        ["<", "punctuation.definition.tag.begin.html"],
        ["template", "entity.name.tag.template.html"],
        ["v-when", "keyword.control.directive.vue"],
        ["=", "punctuation.separator.key-value.html"],
        [quote, "punctuation.definition.string.begin.html"],
        ["{", "meta.pattern.vue punctuation.definition.pattern.vue"],
        ["const", "meta.pattern.vue keyword.declaration.pattern.vue"],
        ["id", "meta.pattern.vue variable.other.constant.ts"],
        ["}", "meta.pattern.vue punctuation.definition.pattern.vue"],
        [quote, "punctuation.definition.string.end.html"],
        [":", "punctuation.definition.directive.vue"],
        ["key", "entity.other.attribute-name.binding.vue"],
        ["=", "punctuation.separator.key-value.html"],
        ['"', "punctuation.definition.string.begin.html"],
        ["id", "meta.embedded.expression.vue variable.other.readwrite.ts"],
        ['"', "punctuation.definition.string.end.html"],
        [">", "punctuation.definition.tag.end.html"],
      ],
    );
  }
});

test("an unfinished guard ends with its attribute instead of swallowing the tag", () => {
  const [line] = tokenize(grammar, ["<template>", `<p v-when="_ if (a" class="x">text</p>`]).slice(
    1,
  );
  assert.deepEqual(
    line.tokens
      .filter((token) => token.start >= line.text.indexOf(" class"))
      .map((token) => [line.text.slice(token.start, token.end), token.scopes.slice(1).join(" ")]),
    [
      [" ", ""],
      ["class", "entity.other.attribute-name.html"],
      ["=", "punctuation.separator.key-value.html"],
      ['"', "string.quoted.double.html punctuation.definition.string.begin.html"],
      ["x", "string.quoted.double.html"],
      ['"', "string.quoted.double.html punctuation.definition.string.end.html"],
      [">", "punctuation.definition.tag.end.html"],
      ["text", ""],
      ["</", "punctuation.definition.tag.begin.html"],
      ["p", "entity.name.tag.html"],
      [">", "punctuation.definition.tag.end.html"],
    ],
  );
});
