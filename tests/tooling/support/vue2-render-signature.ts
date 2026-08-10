import { parse } from "@babel/parser";

import { sortPurePropertyRuns } from "./vue2-render-purity.ts";

type AstRecord = Record<string, unknown> & { type?: string };

const ignoredBabelKeys = new Set([
  "start",
  "end",
  "loc",
  "range",
  "extra",
  "errors",
  "comments",
  "leadingComments",
  "trailingComments",
  "innerComments",
  "shorthand",
]);

const whitespacePreservingTags = new Set(["pre", "textarea", "listing"]);

// Only these direct maps are produced from independently reorderable template
// attributes. Their property values remain ordinary user expressions and are
// deliberately not recursively sorted.
const compilerDataMaps = new Set(["attrs", "domProps", "on", "nativeOn"]);

/**
 * Canonicalize one Vue 2 generated render body as a Babel AST signature.
 *
 * This is intentionally narrower than a general JavaScript normalizer:
 * compiler-owned `_c` data buckets may reorder, and compiler-owned `_v` text
 * outside whitespace-preserving elements may gain layout whitespace. User
 * object literals, helper nesting, handler bodies, and `<pre>`-like text keep
 * their authored order and bytes.
 */
export function vue2RenderFunctionSignature(code: string): unknown {
  const program = parse(`function __vize_render__(){${code}\n}`, {
    sourceType: "script",
  });
  return normalizeNode(program.program.body[0], false);
}

/** Canonicalize the complete output of Vue 2's template compiler. */
export function vue2RenderSignature(render: string, staticRenderFns: string[]): unknown {
  return {
    render: vue2RenderFunctionSignature(render),
    staticRenderFns: staticRenderFns.map(vue2RenderFunctionSignature),
  };
}

/** Canonicalize the module-shaped code emitted by Vue 2.7's `compileTemplate`. */
export function vue27RenderCodeSignature(code: string): unknown {
  const program = parse(code, { sourceType: "script" });
  return normalizeNode(program.program, false);
}

function normalizeNode(node: unknown, preserveWhitespace: boolean): unknown {
  if (Array.isArray(node)) {
    return node
      .filter((entry) => asRecord(entry)?.type !== "EmptyStatement")
      .map((entry) => normalizeNode(entry, preserveWhitespace));
  }
  const record = asRecord(node);
  if (record == null) return node;

  if (record.type === "CallExpression") {
    const callee = vueHelperName(record.callee);
    if (callee === "_v") return normalizeVueTextCall(record, preserveWhitespace);
    if (callee === "_c") return normalizeCreateElementCall(record, preserveWhitespace);
    if (callee === "_b" || callee === "_g") {
      return normalizeDataHelperCall(record, preserveWhitespace);
    }
  }
  if (record.type === "LogicalExpression") {
    return ["logical-chain", record.operator, flattenLogicalChain(record, preserveWhitespace)];
  }
  if (record.type === "TSParenthesizedType") {
    return normalizeNode(record.typeAnnotation, preserveWhitespace);
  }
  if (record.type === "ObjectExpression") {
    return normalizeOrdinaryObject(record, preserveWhitespace);
  }
  return normalizeRecord(record, preserveWhitespace);
}

function normalizeCreateElementCall(record: AstRecord, preserveWhitespace: boolean): unknown {
  const args = arrayField(record, "arguments");
  const tag = stringLiteralValue(args[0]);
  const childPreservesWhitespace =
    preserveWhitespace ||
    (tag != null && whitespacePreservingTags.has(tag)) ||
    hasVPreDataArgument(args[1]);
  // Vue 2 may emit `_c(tag, children, normalizationType)` with no data object.
  // Argument count therefore cannot identify the data slot; only the compiler
  // data shapes can do so without treating children as ordinary expressions.
  const hasDataArgument = isObjectExpression(args[1]) || isDataHelperCall(args[1]);
  const childIndex = hasDataArgument ? 2 : 1;

  return normalizeCall(record, (argument, index) => {
    if (index === 1 && hasDataArgument) {
      return normalizeCompilerDataArgument(argument, false);
    }
    if (index === childIndex) {
      return normalizeNode(argument, childPreservesWhitespace);
    }
    return normalizeNode(argument, preserveWhitespace);
  });
}

function hasVPreDataArgument(node: unknown): boolean {
  const record = asRecord(node);
  if (record?.type !== "ObjectExpression") return false;
  return arrayField(record, "properties").some((property) => {
    const candidate = asRecord(property);
    if (candidate?.type !== "ObjectProperty" || staticPropertyName(candidate) !== "pre") {
      return false;
    }
    const value = asRecord(candidate.value);
    return value?.type === "BooleanLiteral" && value.value === true;
  });
}

function normalizeDataHelperCall(record: AstRecord, preserveWhitespace: boolean): unknown {
  return normalizeCall(record, (argument, index) =>
    index === 0
      ? normalizeCompilerDataArgument(argument, false)
      : normalizeNode(argument, preserveWhitespace),
  );
}

function normalizeCompilerDataArgument(node: unknown, preserveWhitespace: boolean): unknown {
  const record = asRecord(node);
  if (record?.type === "ObjectExpression") {
    return normalizeCompilerDataObject(record, preserveWhitespace);
  }
  return normalizeNode(node, preserveWhitespace);
}

function normalizeCompilerDataObject(record: AstRecord, preserveWhitespace: boolean): unknown {
  const properties = arrayField(record, "properties");
  const normalized = properties.map((property) =>
    normalizeCompilerDataProperty(property, preserveWhitespace),
  );
  return normalizeRecord(record, preserveWhitespace, {
    properties: sortPurePropertyRuns(properties, normalized),
  });
}

function normalizeCompilerDataProperty(property: unknown, preserveWhitespace: boolean): unknown {
  const record = asRecord(property);
  if (record?.type !== "ObjectProperty") {
    return normalizeNode(property, preserveWhitespace);
  }
  const name = staticPropertyName(record);
  const value = asRecord(record.value);
  if (name != null && compilerDataMaps.has(name) && value?.type === "ObjectExpression") {
    return normalizeRecord(record, preserveWhitespace, {
      key: staticKeySignature(record),
      value: normalizeCompilerMapObject(value, preserveWhitespace),
    });
  }
  return normalizeRecord(record, preserveWhitespace, {
    key: staticKeySignature(record),
    value: normalizeNode(record.value, preserveWhitespace),
  });
}

function normalizeCompilerMapObject(record: AstRecord, preserveWhitespace: boolean): unknown {
  const properties = arrayField(record, "properties");
  const normalized = properties.map((property) => normalizeNode(property, preserveWhitespace));
  return normalizeRecord(record, preserveWhitespace, {
    properties: sortPurePropertyRuns(properties, normalized),
  });
}

function normalizeOrdinaryObject(record: AstRecord, preserveWhitespace: boolean): unknown {
  // ObjectExpression property order is observable through evaluation order.
  // Keep it exactly as Babel parsed it. Compiler-owned containers above sort
  // only consecutive, unique, provably side-effect-free properties.
  return normalizeRecord(record, preserveWhitespace, {
    properties: arrayField(record, "properties").map((property) =>
      normalizeNode(property, preserveWhitespace),
    ),
  });
}

function normalizeVueTextCall(record: AstRecord, preserveWhitespace: boolean): unknown {
  const args = arrayField(record, "arguments");
  if (preserveWhitespace || args.length !== 1) {
    return normalizeCall(record, (argument) => normalizeNode(argument, preserveWhitespace));
  }

  const rawSegments: Array<
    { kind: "text"; value: string } | { kind: "expression"; value: unknown }
  > = [];
  for (const segment of flattenStringConcatenation(args[0])) {
    const text = stringLiteralValue(segment);
    if (text != null) {
      const previous = rawSegments.at(-1);
      if (previous?.kind === "text") previous.value += text;
      else rawSegments.push({ kind: "text", value: text });
      continue;
    }
    rawSegments.push({ kind: "expression", value: segment });
  }
  const segments = rawSegments.flatMap((segment): unknown[] => {
    if (segment.kind === "expression") {
      return [["expression", normalizeNode(segment.value, false)]];
    }
    const text = segment.value.replace(/\s+/gu, " ");
    return text === "" ? [] : [["text", text]];
  });
  return ["vue-text", segments];
}

function normalizeCall(
  record: AstRecord,
  normalizeArgument: (argument: unknown, index: number) => unknown,
): unknown {
  return normalizeRecord(record, false, {
    arguments: arrayField(record, "arguments").map(normalizeArgument),
  });
}

function normalizeRecord(
  record: AstRecord,
  preserveWhitespace: boolean,
  overrides: Record<string, unknown> = {},
): Record<string, unknown> {
  const signature: Record<string, unknown> = {};
  for (const key of Object.keys(record).sort()) {
    if (ignoredBabelKeys.has(key)) continue;
    if (Object.hasOwn(overrides, key)) {
      signature[key] = overrides[key];
    } else if (key === "key" && isStaticKeyOwner(record)) {
      signature[key] = staticKeySignature(record);
    } else {
      signature[key] = normalizeNode(record[key], preserveWhitespace);
    }
  }
  return signature;
}

function flattenStringConcatenation(node: unknown, output: unknown[] = []): unknown[] {
  const record = asRecord(node);
  if (record?.type === "BinaryExpression" && record.operator === "+") {
    flattenStringConcatenation(record.left, output);
    flattenStringConcatenation(record.right, output);
  } else {
    output.push(node);
  }
  return output;
}

function flattenLogicalChain(record: AstRecord, preserveWhitespace: boolean): unknown[] {
  const operands: unknown[] = [];
  for (const side of [record.left, record.right]) {
    const child = asRecord(side);
    if (child?.type === "LogicalExpression" && child.operator === record.operator) {
      operands.push(...flattenLogicalChain(child, preserveWhitespace));
    } else {
      operands.push(normalizeNode(side, preserveWhitespace));
    }
  }
  return operands;
}

function staticKeySignature(record: AstRecord): unknown {
  const name = staticPropertyName(record);
  return name == null ? normalizeNode(record.key, false) : ["static-key", name];
}

function staticPropertyName(record: AstRecord): string | null {
  if (record.computed === true) return null;
  const key = asRecord(record.key);
  if (key?.type === "Identifier" && typeof key.name === "string") return key.name;
  if (key?.type === "StringLiteral" && typeof key.value === "string") return key.value;
  if (key?.type === "NumericLiteral" && typeof key.value === "number") {
    return String(key.value);
  }
  return null;
}

function isStaticKeyOwner(record: AstRecord): boolean {
  return record.type === "ObjectProperty" || record.type === "ObjectMethod";
}

function isObjectExpression(node: unknown): boolean {
  return asRecord(node)?.type === "ObjectExpression";
}

function isDataHelperCall(node: unknown): boolean {
  const record = asRecord(node);
  if (record?.type !== "CallExpression") return false;
  const name = vueHelperName(record.callee);
  return name === "_b" || name === "_g";
}

function vueHelperName(node: unknown): string | null {
  const record = asRecord(node);
  if (record?.type === "Identifier" && typeof record.name === "string") return record.name;
  if (record?.type !== "MemberExpression" || record.computed === true) return null;
  if (identifierValue(record.object) !== "_vm") return null;
  return identifierValue(record.property);
}

function identifierValue(node: unknown): string | null {
  const record = asRecord(node);
  return record?.type === "Identifier" && typeof record.name === "string" ? record.name : null;
}

function stringLiteralValue(node: unknown): string | null {
  const record = asRecord(node);
  return record?.type === "StringLiteral" && typeof record.value === "string" ? record.value : null;
}

function arrayField(record: AstRecord, key: string): unknown[] {
  const value = record[key];
  return Array.isArray(value) ? value : [];
}

function asRecord(value: unknown): AstRecord | null {
  return value != null && typeof value === "object" && !Array.isArray(value)
    ? (value as AstRecord)
    : null;
}
