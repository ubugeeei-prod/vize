// Structural SFC equivalence for the glyph parse-preservation property.
//
// Both sides are parsed with @vue/compiler-sfc (the reference Vue parser, so
// the check is independent of vize's own parser) and compared on:
//   - parse error codes (formatting must not add or remove parse errors),
//   - the block multiset: kind, lang, and attrs (glyph reorders blocks into
//     canonical order by default, so order is compared per kind only),
//   - template AST shape: tags, structure, text (exact inside <pre>-like
//     elements, whitespace-condensed elsewhere), interpolations, comments,
//   - expressions by Babel AST signature (glyph legitimately reprints
//     expressions via oxc, so token spacing and quote style may change while
//     the parsed AST must not); expressions the Vue parser leaves unparsed
//     fall back to a whitespace-stripped text comparison,
//   - props as per-spread-segment multisets: glyph sorts attributes inside a
//     priority group by default, but moving a prop across a no-arg v-bind /
//     v-on spread changes merge semantics and is reported.
// Script content is intentionally not compared (glyph reprints scripts via
// oxc; text-level comparison cannot separate reformatting from corruption)
// and style content is intentionally not compared (glyph canonicalizes CSS
// values via lightningcss, which is not whitespace-only). Those blocks are
// still compared by kind and attrs, and script/style corruption stays visible
// through the compile-facing oracles and the lint-agreement property.
import { createRequire } from "node:module";

import { expressionSignature } from "./babel-expression-signature.ts";
import type { ExpressionNode } from "./babel-expression-signature.ts";

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

type SfcBlock = {
  type: string;
  lang?: string;
  attrs: Record<string, string | true>;
  content: string;
};
type SfcDescriptor = {
  template: (SfcBlock & { ast?: TemplateNode }) | null;
  script: SfcBlock | null;
  scriptSetup: SfcBlock | null;
  styles: SfcBlock[];
  customBlocks: SfcBlock[];
};
type TemplateNode = {
  type: number;
  tag?: string;
  ns?: number;
  tagType?: number;
  props?: TemplateProp[];
  children?: TemplateNode[];
  content?: string | { content: string };
};
type TemplateProp = {
  type: number;
  name: string;
  value?: { content: string } | null;
  arg?: ExpressionNode | null;
  exp?: ExpressionNode | null;
  modifiers?: Array<{ content: string }>;
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

  const beforeErrors = before.errors
    .map((error) => String(error.code ?? error.message))
    .sort((left, right) => left.localeCompare(right));
  const afterErrors = after.errors
    .map((error) => String(error.code ?? error.message))
    .sort((left, right) => left.localeCompare(right));
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

function compareBlocks(before: SfcDescriptor, after: SfcDescriptor, differences: string[]): void {
  for (const kind of ["template", "script", "scriptSetup"] as const) {
    const beforeBlock = before[kind];
    const afterBlock = after[kind];
    if ((beforeBlock == null) !== (afterBlock == null)) {
      differences.push(`${kind} block ${beforeBlock == null ? "appeared" : "disappeared"}`);
    } else if (beforeBlock != null && afterBlock != null) {
      compareAttrs(kind, beforeBlock, afterBlock, differences);
    }
  }
  for (const kind of ["styles", "customBlocks"] as const) {
    const beforeBlocks = before[kind];
    const afterBlocks = after[kind];
    if (beforeBlocks.length !== afterBlocks.length) {
      differences.push(`${kind} count changed: ${beforeBlocks.length} -> ${afterBlocks.length}`);
      continue;
    }
    const signature = (block: SfcBlock): string =>
      JSON.stringify([
        block.type,
        semanticAttrEntries(kind === "styles" ? "style" : "customBlock", block),
        kind === "customBlocks" ? condense(block.content) : null,
      ]);
    const beforeSignatures = beforeBlocks.map(signature).sort();
    const afterSignatures = afterBlocks.map(signature).sort();
    for (let index = 0; index < beforeSignatures.length; index += 1) {
      if (beforeSignatures[index] !== afterSignatures[index]) {
        differences.push(
          `${kind} changed: ${beforeSignatures[index]} -> ${afterSignatures[index]}`,
        );
        break;
      }
    }
  }
}

function compareAttrs(
  label: "template" | "script" | "scriptSetup",
  before: SfcBlock,
  after: SfcBlock,
  differences: string[],
): void {
  const beforeEntries = JSON.stringify(semanticAttrEntries(label, before));
  const afterEntries = JSON.stringify(semanticAttrEntries(label, after));
  if (beforeEntries !== afterEntries) {
    differences.push(`${label} block attrs changed: ${beforeEntries} -> ${afterEntries}`);
  }
}

// These are the attributes compiler-sfc itself consumes by presence: each
// parser branch coerces the raw value to truthiness or assigns a descriptor
// slot/boolean. Keep this block-kind table closed so module, lang, src,
// generic, and custom attributes remain value-sensitive.
const compilerPresenceAttrs = {
  template: ["functional", "vapor"],
  script: [],
  scriptSetup: ["setup", "vapor"],
  style: ["scoped"],
  customBlock: [],
} as const;

function semanticAttrEntries(
  kind: keyof typeof compilerPresenceAttrs,
  block: SfcBlock,
): Array<[string, string | true]> {
  const presenceAttrs = compilerPresenceAttrs[kind];
  if (
    !presenceAttrs.some(
      (attribute) => Object.hasOwn(block.attrs, attribute) && block.attrs[attribute] !== true,
    )
  ) {
    return sortedAttrEntries(block.attrs);
  }
  const attrs = { ...block.attrs };
  for (const attribute of presenceAttrs) {
    if (Object.hasOwn(attrs, attribute)) attrs[attribute] = true;
  }
  return sortedAttrEntries(attrs);
}

function sortedAttrEntries(attrs: Record<string, string | true>): Array<[string, string | true]> {
  return Object.entries(attrs).sort(([left], [right]) => left.localeCompare(right));
}

const ELEMENT = 1;
const TEXT = 2;
const COMMENT = 3;
const INTERPOLATION = 5;
const ATTRIBUTE = 6;
const DIRECTIVE = 7;

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
        preserveWhitespace || isWhitespacePreservingTag(left.node.tag),
        differences,
      );
      // Report at most one template difference per file to keep output small.
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
    if (!preserveWhitespace && content.trim() === "") return null;
    return JSON.stringify(["text", preserveWhitespace ? content : condense(content)]);
  }
  if (node.type === INTERPOLATION) {
    return JSON.stringify(["interpolation", expressionSignature(node.content as ExpressionNode)]);
  }
  if (node.type === COMMENT) {
    return JSON.stringify(["comment", condense(node.content as string)]);
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

/**
 * Group props into segments split at no-arg v-bind / v-on spreads. Segments
 * are compared as sorted multisets (glyph sorts attributes by default) while
 * the segment boundaries pin every prop's position relative to each spread,
 * because crossing a spread changes runtime merge behavior.
 */
function normalizeProps(props: TemplateProp[]): unknown[] {
  const segments: string[][] = [[]];
  for (const prop of props) {
    const normalized = normalizeProp(prop);
    if (
      prop.type === DIRECTIVE &&
      prop.arg == null &&
      (prop.name === "bind" || prop.name === "on")
    ) {
      segments.push([JSON.stringify(normalized)], []);
    } else {
      segments[segments.length - 1].push(JSON.stringify(normalized));
    }
  }
  return segments.map((segment, index) => (index % 2 === 1 ? segment : [...segment].sort()));
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

function isWhitespacePreservingTag(tag: string | undefined): boolean {
  return tag === "pre" || tag === "textarea" || tag === "listing";
}

function condense(value: string): string {
  return value.replace(/\s+/g, " ").trim();
}
