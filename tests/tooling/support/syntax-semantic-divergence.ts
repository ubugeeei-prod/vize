import assert from "node:assert/strict";
import { createHash } from "node:crypto";

import { canonicalJson } from "./syntax-evidence.ts";
import type { TextMateGrammar } from "./vue-textmate.ts";

export { canonicalJson, sha256 } from "./syntax-evidence.ts";

export const semanticLineTimeoutMs = 1_000;

export const semanticCategories = Object.freeze([
  "attribute",
  "comment",
  "invalid",
  "tag",
] as const);

export const semanticNormalization = Object.freeze({
  version: 3,
  categories: semanticCategories,
  coordinates: "1-based UTF-16 columns with an exclusive end",
  omittedCategories: Object.freeze([
    "function",
    "keyword",
    "label",
    "literal",
    "number",
    "operator",
    "property",
    "selector",
    "string",
    "type",
    "variable",
  ]),
  ignoredScopeFamilies: Object.freeze([
    "case-clause",
    "cast",
    "expression",
    "fenced_code",
    "heading",
    "html",
    "inline",
    "markup",
    "meta",
    "new",
    "punctuation",
    "self-closing-tag",
    "source",
    "switch-block",
    "switch-expression",
    "switch-statement",
    "text",
    "vue",
  ]),
  rationale:
    "Compare Vue shell roles. Shiki 4.0.2 leaves interpolations unscoped and treats many directives as HTML attributes, so directive names normalize to attributes while embedded-language and string scopes are omitted.",
});

export type SemanticSpan = {
  categories: string[];
  endColumn: number;
  line: number;
  startColumn: number;
};

export function tokenizeSemanticSource(
  grammar: TextMateGrammar,
  source: string,
  rootScope: string,
  label: string,
  checkDeadline: () => void = () => {},
): { lineCount: number; semanticSpans: SemanticSpan[]; tokenCount: number; tokenSha256: string } {
  const lines = source.split("\n");
  const semanticSpans: SemanticSpan[] = [];
  const tokenDigest = createHash("sha256");
  let ruleStack: unknown = null;
  let tokenCount = 0;
  let nonRootTokenCount = 0;

  for (const [lineIndex, line] of lines.entries()) {
    checkDeadline();
    const result = grammar.tokenizeLine(line, ruleStack, semanticLineTimeoutMs);
    checkDeadline();
    if (result.stoppedEarly) {
      throw new Error(`${label}:${lineIndex + 1}: exceeded ${semanticLineTimeoutMs}ms`);
    }
    if (result.tokens.length === 0) throw new Error(`${label}:${lineIndex + 1}: no tokens`);
    let end = 0;
    for (const [tokenIndex, token] of result.tokens.entries()) {
      validateToken(token, end, line.length, rootScope, `${label}:${lineIndex + 1}:${tokenIndex}`);
      const categories = normalizeScopes(token.scopes, `${label}:${lineIndex + 1}:${tokenIndex}`);
      const authoredEnd = Math.min(token.endIndex, line.length);
      if (authoredEnd > token.startIndex) {
        appendSpan(semanticSpans, {
          categories,
          line: lineIndex + 1,
          startColumn: token.startIndex + 1,
          endColumn: authoredEnd + 1,
        });
      }
      if (token.scopes.some((scope) => scope !== rootScope)) nonRootTokenCount += 1;
      tokenDigest.update(
        `${JSON.stringify([lineIndex, token.startIndex, token.endIndex, token.scopes])}\n`,
      );
      end = token.endIndex;
      tokenCount += 1;
    }
    if (end !== line.length && end !== line.length + 1) {
      throw new Error(`${label}:${lineIndex + 1}: stopped at ${end}/${line.length}`);
    }
    ruleStack = result.ruleStack;
    checkDeadline();
  }
  if (source.trim().length > 0 && nonRootTokenCount === 0) {
    throw new Error(`${label}: non-empty source produced only the root scope`);
  }
  return {
    lineCount: lines.length,
    semanticSpans,
    tokenCount,
    tokenSha256: tokenDigest.digest("hex"),
  };
}

function normalizeScopes(scopes: string[], label: string): string[] {
  assert.ok(Array.isArray(scopes) && scopes.length > 0, `${label}: missing scopes`);
  const categories = new Set<string>();
  for (const scope of scopes) {
    assert.equal(typeof scope, "string", `${label}: scope must be a string`);
    if (scope.includes("invalid.unresolved-oracle-grammar")) {
      throw new Error(`${label}: activated unresolved oracle grammar ${scope}`);
    }
    if (/^punctuation\.(?:definition\.directive|attribute-shorthand\.[^.]+)\..*vue$/.test(scope)) {
      categories.add("attribute");
      continue;
    }
    const topLevel = scope.split(".", 1)[0];
    if (semanticNormalization.ignoredScopeFamilies.includes(topLevel)) continue;
    const category = semanticCategory(scope);
    if (category == null) throw new Error(`${label}: unknown semantic scope ${scope}`);
    if (semanticCategories.includes(category as (typeof semanticCategories)[number])) {
      categories.add(category);
    } else {
      assert.ok(
        semanticNormalization.omittedCategories.includes(category),
        `${label}: ${category}`,
      );
    }
  }
  return [...categories].sort();
}

function semanticCategory(scope: string): string | null {
  if (scope.startsWith("comment")) return "comment";
  if (/^attribute_value\d*$/u.test(scope)) return "string";
  if (scope.startsWith("string")) return "string";
  if (scope.startsWith("constant.numeric")) return "number";
  if (scope.startsWith("constant") || scope.startsWith("support.constant")) return "literal";
  if (scope.startsWith("keyword.operator")) return "operator";
  if (scope.startsWith("keyword") && scope.endsWith(".vue")) return "attribute";
  if (scope.startsWith("keyword") || scope.startsWith("storage")) return "keyword";
  if (scope.startsWith("tag.")) return "tag";
  if (scope.startsWith("entity.name.tag")) return "tag";
  if (scope.startsWith("entity.name.label")) return "label";
  if (scope.startsWith("entity.name.section")) return "label";
  if (scope.startsWith("entity.name.constant")) return "literal";
  if (scope.startsWith("entity.other.keyframe-offset")) return "literal";
  if (scope.startsWith("entity.other.counter-name")) return "literal";
  if (scope.startsWith("entity.other.attribute-selector")) return "selector";
  if (scope.startsWith("entity.other.attribute-name")) return "attribute";
  if (scope.startsWith("entity.name.function") || scope.startsWith("support.function")) {
    return "function";
  }
  if (scope === "name.generic.filter.pug") return "function";
  if (
    /^(?:entity\.name\.(?:class|enum|interface|struct|type)|support\.(?:class|type))/.test(scope)
  ) {
    return "type";
  }
  if (scope.startsWith("entity.other.inherited-class")) return "type";
  if (
    scope.startsWith("variable.other.property") ||
    scope.startsWith("support.variable.property")
  ) {
    return "property";
  }
  if (scope.startsWith("support.other.variable")) return "variable";
  if (scope.startsWith("variable") || scope.startsWith("support.variable")) return "variable";
  if (scope.startsWith("invalid")) return "invalid";
  return null;
}

function validateToken(
  token: { endIndex: number; scopes: string[]; startIndex: number },
  expectedStart: number,
  lineLength: number,
  rootScope: string,
  label: string,
): void {
  assert.ok(Number.isSafeInteger(token.startIndex) && Number.isSafeInteger(token.endIndex));
  assert.ok(token.startIndex === expectedStart && token.endIndex > token.startIndex, label);
  assert.ok(token.endIndex <= lineLength + 1, `${label}: token exceeds line`);
  assert.equal(token.scopes[0], rootScope, `${label}: lost root scope ${rootScope}`);
}

function appendSpan(records: SemanticSpan[], record: SemanticSpan): void {
  const previous = records.at(-1);
  if (
    previous?.line === record.line &&
    previous.endColumn === record.startColumn &&
    canonicalJson(previous.categories) === canonicalJson(record.categories)
  ) {
    previous.endColumn = record.endColumn;
  } else records.push(record);
}
