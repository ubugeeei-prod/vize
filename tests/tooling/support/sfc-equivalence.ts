// Vue 3 semantic signatures keep structure and spread/pre boundaries exact.
// Script/style bodies use separate compile/lint oracles; this comparator keeps
// their block identity and attrs because text alone cannot separate reprints.
import { createRequire } from "node:module";

import { expressionSignature } from "./babel-expression-signature.ts";
import type { ExpressionNode } from "./babel-expression-signature.ts";
import {
  compareBlocks,
  parseErrorSignatures,
  sfcEnvelopeSemanticSignature,
} from "./sfc-equivalence/blocks.ts";
import type { SfcDescriptor, TemplateNode, TemplateProp } from "./sfc-equivalence/blocks.ts";
import { findSfcOpeningTagEnd } from "./sfc-opening-tag.ts";

export type { TemplateNode } from "./sfc-equivalence/blocks.ts";

const require = createRequire(import.meta.url);
// Resolved from tests/node_modules (pinned devDependency of @vize/tests).
const { parse } = require("@vue/compiler-sfc") as {
  parse: (
    source: string,
    options: { filename: string; sourceMap: boolean },
  ) => {
    descriptor: SfcDescriptor;
    errors: Array<{ code?: number; message: string }>;
  };
};

/** Compare original and formatted SFC sources; returns human-readable diffs. */
export function compareSfcEquivalence(
  original: string,
  formatted: string,
  filename: string,
): string[] {
  const before = parse(original, { filename, sourceMap: false });
  const after = parse(formatted, { filename, sourceMap: false });
  const differences: string[] = [];

  const beforeErrors = parseErrorSignatures(before.errors);
  const afterErrors = parseErrorSignatures(after.errors);
  if (JSON.stringify(beforeErrors) !== JSON.stringify(afterErrors)) {
    differences.push(
      `parse errors changed: [${beforeErrors.join(", ")}] -> [${afterErrors.join(", ")}]`,
    );
    return differences;
  }

  compareBlocks(before.descriptor, after.descriptor, differences);
  const beforeAst = before.descriptor.template?.ast;
  const afterAst = after.descriptor.template?.ast;
  if (beforeAst != null && afterAst != null) {
    compareChildren(beforeAst, afterAst, "template", false, differences);
  }
  return differences;
}

/** Compare only the SFC envelope; a non-HTML language owns its template AST. */
export function compareSfcBlockStructure(
  original: string,
  formatted: string,
  filename: string,
): string[] {
  const before = parse(original, { filename, sourceMap: false });
  const after = parse(formatted, { filename, sourceMap: false });
  const differences: string[] = [];
  const beforeErrors = parseErrorSignatures(before.errors);
  const afterErrors = parseErrorSignatures(after.errors);
  if (JSON.stringify(beforeErrors) !== JSON.stringify(afterErrors)) {
    differences.push(
      `parse errors changed: [${beforeErrors.join(", ")}] -> [${afterErrors.join(", ")}]`,
    );
    return differences;
  }
  compareBlocks(before.descriptor, after.descriptor, differences);
  return differences;
}

/** Canonical Vue 3 semantic signature used by dialect evidence hashes. */
export function sfcSemanticSignature(source: string, filename: string): string {
  const parsed = parse(source, { filename, sourceMap: false });
  return JSON.stringify([
    parseErrorSignatures(parsed.errors),
    sfcEnvelopeSemanticSignature(parsed.descriptor),
    parsed.descriptor.template?.ast == null
      ? null
      : semanticTree(parsed.descriptor.template.ast, false),
  ]);
}

/** Compare template ASTs already produced by one pinned compiler baseline. */
export function compareTemplateAstEquivalence(before: TemplateNode, after: TemplateNode): string[] {
  const differences: string[] = [];
  compareChildren(before, after, "template", false, differences);
  return differences;
}

/** Stable semantic signature used only for evidence hashes, never source maps. */
export function templateAstSemanticSignature(root: TemplateNode): string {
  return JSON.stringify(semanticTree(root, false));
}

const ELEMENT = 1;
const TEXT = 2;
const COMMENT = 3;
const INTERPOLATION = 5;
const ATTRIBUTE = 6;

function compareChildren(
  before: TemplateNode,
  after: TemplateNode,
  nodePath: string,
  preserveWhitespace: boolean,
  differences: string[],
): void {
  const baseline = differences.length;
  const beforeChildren = normalizedChildren(before, preserveWhitespace);
  const afterChildren = normalizedChildren(after, preserveWhitespace);
  const limit = Math.max(beforeChildren.length, afterChildren.length);
  for (let index = 0; index < limit; index += 1) {
    const left = beforeChildren[index];
    const right = afterChildren[index];
    if (left == null || right == null) {
      differences.push(
        `${nodePath}: child ${index} ${left == null ? "appeared" : "disappeared"} ` +
          `(${describeNode(left ?? right)})`,
      );
      return;
    }
    const childPath = `${nodePath} > ${describeNode(left)}[${index}]`;
    if (left.summary !== right.summary) {
      differences.push(`${childPath}: ${left.summary} -> ${right.summary}`);
      return;
    }
    if (left.node.type === ELEMENT) {
      compareChildren(
        left.node,
        right.node,
        childPath,
        preserveWhitespace || preservesAuthoredWhitespace(left.node),
        differences,
      );
      if (differences.length > baseline) return;
    }
  }
}

type NormalizedChild = { node: TemplateNode; summary: string };

function normalizedChildren(node: TemplateNode, preserveWhitespace: boolean): NormalizedChild[] {
  const children = node.children ?? [];
  const normalized: NormalizedChild[] = [];
  for (const child of children) {
    const summary = summarizeNode(child, preserveWhitespace);
    if (summary == null) continue;
    normalized.push({ node: child, summary });
  }
  return normalized;
}

function summarizeNode(node: TemplateNode, preserveWhitespace: boolean): string | null {
  if (node.type === TEXT) {
    const content = node.content as string;
    const authored = preserveWhitespace
      ? (node.loc?.source ?? content)
      : content.replace(/\s+/g, " ");
    return JSON.stringify(["text", authored]);
  }
  if (node.type === INTERPOLATION) {
    return JSON.stringify(["interpolation", expressionSignature(node.content as ExpressionNode)]);
  }
  if (node.type === COMMENT) {
    return JSON.stringify(["comment", node.content as string]);
  }
  if (node.type === ELEMENT) {
    return JSON.stringify([
      "element",
      node.tag,
      node.ns,
      node.tagType,
      normalizeProps(node.props ?? []),
    ]);
  }
  return JSON.stringify(["node", node.type]);
}

function semanticTree(node: TemplateNode, preserveWhitespace: boolean): unknown {
  const children = normalizedChildren(node, preserveWhitespace).map(({ node: child, summary }) => [
    summary,
    child.type === ELEMENT
      ? semanticTree(child, preserveWhitespace || preservesAuthoredWhitespace(child))
      : null,
  ]);
  return children;
}

/** Sort unique static runs; directives and duplicate literals stay fixed. */
function normalizeProps(props: TemplateProp[]): unknown[] {
  const normalized: unknown[] = [];
  let literals: TemplateProp[] = [];
  const flushLiterals = (): void => {
    if (literals.length === 0) return;
    const names = literals.map((prop) => prop.name.toLowerCase());
    const unique = new Set(names).size === names.length;
    const entries = literals.map((prop) => JSON.stringify(normalizeProp(prop)));
    normalized.push(...(unique ? entries.sort() : entries));
    literals = [];
  };
  for (const prop of props) {
    if (prop.type === ATTRIBUTE) {
      literals.push(prop);
      continue;
    }
    flushLiterals();
    normalized.push(JSON.stringify(normalizeProp(prop)));
  }
  flushLiterals();
  return normalized;
}

function normalizeProp(prop: TemplateProp): unknown {
  if (prop.type === ATTRIBUTE) {
    return ["attr", prop.name, prop.value?.content ?? null];
  }
  return [
    "dir",
    prop.name,
    prop.arg == null ? null : [expressionSignature(prop.arg), prop.arg.isStatic === true],
    prop.exp == null ? null : expressionSignature(prop.exp),
    (prop.modifiers ?? []).map((modifier) => modifier.content),
  ];
}

function describeNode(child: NormalizedChild | undefined): string {
  if (child == null) return "unknown";
  const node = child.node;
  if (node.type === ELEMENT) return `<${node.tag}>`;
  if (node.type === TEXT) return "#text";
  if (node.type === INTERPOLATION) return "#interpolation";
  if (node.type === COMMENT) return "#comment";
  return `#node${node.type}`;
}

function preservesAuthoredWhitespace(node: TemplateNode): boolean {
  if (node.tag === "pre" || node.tag === "textarea" || node.tag === "listing") {
    return true;
  }
  const source = node.loc?.source;
  if (source == null) return false;
  const end = findSfcOpeningTagEnd(source, 1);
  if (end < 0) return false;
  const openingTag = source.slice(0, end + 1);
  return /(?:^|\s)v-pre(?:\s|=|\/?>)/u.test(openingTag);
}
