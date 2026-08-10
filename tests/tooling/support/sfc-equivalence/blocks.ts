// SFC envelope comparison: block presence, kind, and attrs. Split out of
// sfc-equivalence.ts so both the full equivalence check and the envelope-only
// check (used for non-HTML template languages) share one implementation.
import type { ExpressionNode } from "../babel-expression-signature.ts";

export type SfcBlock = {
  type: string;
  lang?: string;
  attrs: Record<string, string | true>;
  content: string;
};
export type SfcDescriptor = {
  template: (SfcBlock & { ast?: TemplateNode }) | null;
  script: SfcBlock | null;
  scriptSetup: SfcBlock | null;
  styles: SfcBlock[];
  customBlocks: SfcBlock[];
};
export type TemplateNode = {
  type: number;
  tag?: string;
  ns?: number;
  tagType?: number;
  props?: TemplateProp[];
  children?: TemplateNode[];
  content?: string | { content: string };
};
export type TemplateProp = {
  type: number;
  name: string;
  value?: { content: string } | null;
  arg?: ExpressionNode | null;
  exp?: ExpressionNode | null;
  modifiers?: Array<{ content: string }>;
};

function codePointCompare(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

export function parseErrorSignatures(errors: Array<{ code?: number; message: string }>): string[] {
  return errors.map((error) => String(error.code ?? error.message)).sort(codePointCompare);
}

export function condense(value: string): string {
  return value.replace(/\s+/g, " ").trim();
}

export function compareBlocks(
  before: SfcDescriptor,
  after: SfcDescriptor,
  differences: string[],
): void {
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
  return Object.entries(attrs).sort(([left], [right]) => codePointCompare(left, right));
}
