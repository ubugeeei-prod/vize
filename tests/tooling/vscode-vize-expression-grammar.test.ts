// Directive values are tokenized by a small expression grammar that must never
// run past its attribute. `name<` only starts type arguments when they close on
// the same line in front of a call; anything else is a comparison.
import assert from "node:assert/strict";
import { test } from "node:test";
import { loadVueTextMateGrammar, tokenizeLines } from "./support/vue-textmate.ts";

const { grammar } = await loadVueTextMateGrammar();
const expression = "meta.embedded.expression.vue";
const typeArguments = `${expression} meta.type.parameters.ts`;

/** Every non-blank token of `line`, inside a template, with its scopes below the root. */
function scopes(line: string): Array<[string, string]> {
  return tokenizeLines(grammar, ["<template>", line, "</template>"])
    .filter((token) => token.line === line && token.text.trim() !== "")
    .map((token) => [token.text, token.scopes.slice(1).join(" ")]);
}

const openTag: Array<[string, string]> = [
  ["<", "punctuation.definition.tag.begin.html"],
  ["i", "entity.name.tag.html"],
];
const afterValue: Array<[string, string]> = [
  ['"', "punctuation.definition.string.end.html"],
  ["x", "entity.other.attribute-name.html"],
  ["=", "punctuation.separator.key-value.html"],
  ['"', "string.quoted.double.html punctuation.definition.string.begin.html"],
  ["1", "string.quoted.double.html"],
  ['"', "string.quoted.double.html punctuation.definition.string.end.html"],
  [">", "punctuation.definition.tag.end.html"],
];
const vIf: Array<[string, string]> = [
  ["v-if", "keyword.control.directive.vue"],
  ["=", "punctuation.separator.key-value.html"],
  ['"', "punctuation.definition.string.begin.html"],
];
const bind: Array<[string, string]> = [
  [":", "punctuation.definition.directive.vue"],
  ["a", "entity.other.attribute-name.binding.vue"],
  ["=", "punctuation.separator.key-value.html"],
  ['"', "punctuation.definition.string.begin.html"],
];

test("a less-than comparison is an operator and the tag goes on after it", () => {
  assert.deepEqual(scopes('<i v-if="count < limit && ok(count)" x="1">'), [
    ...openTag,
    ...vIf,
    ["count", `${expression} variable.other.readwrite.ts`],
    ["<", `${expression} keyword.operator.ts`],
    ["limit", `${expression} variable.other.readwrite.ts`],
    ["&&", `${expression} keyword.operator.ts`],
    ["ok", `${expression} entity.name.function.ts`],
    ["(", `${expression} keyword.operator.ts`],
    ["count", `${expression} variable.other.readwrite.ts`],
    [")", `${expression} keyword.operator.ts`],
    ...afterValue,
  ]);
});

test("a comparison stays one when a later `>` is followed by a parenthesis", () => {
  assert.deepEqual(scopes('<i v-if="a < b" disabled>(text)</i>'), [
    ...openTag,
    ...vIf,
    ["a", `${expression} variable.other.readwrite.ts`],
    ["<", `${expression} keyword.operator.ts`],
    ["b", `${expression} variable.other.readwrite.ts`],
    ['"', "punctuation.definition.string.end.html"],
    [" disabled", ""],
    [">", "punctuation.definition.tag.end.html"],
    ["(text)", ""],
    ["</", "punctuation.definition.tag.begin.html"],
    ["i", "entity.name.tag.html"],
    [">", "punctuation.definition.tag.end.html"],
  ]);
});

test("type arguments may be string literal types", () => {
  assert.deepEqual(scopes(`<i :a="pick<'sm'>(v)" x="1">`), [
    ...openTag,
    ...bind,
    ["pick", `${typeArguments} entity.name.function.ts`],
    ["<", `${typeArguments} punctuation.definition.typeparameters.begin.ts`],
    ["'", `${typeArguments} string.quoted.single.ts`],
    ["sm", `${typeArguments} string.quoted.single.ts`],
    ["'", `${typeArguments} string.quoted.single.ts`],
    [">", `${typeArguments} punctuation.definition.typeparameters.end.ts`],
    ["(", `${expression} keyword.operator.ts`],
    ["v", `${expression} variable.other.readwrite.ts`],
    [")", `${expression} keyword.operator.ts`],
    ...afterValue,
  ]);
});

test("type arguments nest to any depth", () => {
  const nested = (depth: number) => `${expression}${" meta.type.parameters.ts".repeat(depth)}`;
  assert.deepEqual(scopes('<i :a="make<Foo<Bar<Baz>>>()" x="1">'), [
    ...openTag,
    ...bind,
    ["make", `${nested(1)} entity.name.function.ts`],
    ["<", `${nested(1)} punctuation.definition.typeparameters.begin.ts`],
    ["Foo", `${nested(2)} entity.name.type.ts`],
    ["<", `${nested(2)} punctuation.definition.typeparameters.begin.ts`],
    ["Bar", `${nested(3)} entity.name.type.ts`],
    ["<", `${nested(3)} punctuation.definition.typeparameters.begin.ts`],
    ["Baz", `${nested(3)} entity.name.type.ts`],
    [">", `${nested(3)} punctuation.definition.typeparameters.end.ts`],
    [">", `${nested(2)} punctuation.definition.typeparameters.end.ts`],
    [">", `${nested(1)} punctuation.definition.typeparameters.end.ts`],
    ["()", `${expression} keyword.operator.ts`],
    ...afterValue,
  ]);
});
