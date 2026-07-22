// Babel AST signatures for the glyph parse-preservation property
// (tests/tooling/support/sfc-equivalence.ts). Glyph legitimately reprints
// template expressions through oxc, so the signature erases print-only facts
// (positions, quote style, redundant parens, separator semicolons) while
// preserving everything semantic. Each normalization below is matched to a
// concrete reprint the corpus exercises; anything unlisted must compare equal.

/** Vue SimpleExpression-ish node: raw content plus an optional Babel AST. */
export type ExpressionNode = { content: string; isStatic?: boolean; ast?: unknown };

/**
 * Expression equivalence: prefer the Babel AST the Vue parser attaches.
 * Static args compare as text; unparsed expressions fall back to text
 * stripped of whitespace, which cannot distinguish whitespace inside string
 * literals but never rejects a legitimate reprint.
 */
export function expressionSignature(expression: ExpressionNode): unknown {
  if (expression.isStatic === true) return expression.content;
  if (expression.ast != null && typeof expression.ast === "object") {
    return babelSignature(unwrapSingleExpressionProgram(expression.ast));
  }
  return expression.content.replace(/\s+/g, "");
}

/**
 * Vue parses inline handlers as a bare expression until the source contains a
 * `;`, at which point it parses a statement list instead. A single-expression
 * program is the same handler, so unwrap it — a formatter-added semicolon
 * inside an arrow body must not read as an AST change.
 */
function unwrapSingleExpressionProgram(ast: unknown): unknown {
  const node = ast as {
    type?: string;
    body?: Array<{ type?: string; expression?: unknown }>;
    directives?: unknown[];
  };
  if (
    node.type === "Program" &&
    Array.isArray(node.body) &&
    node.body.length === 1 &&
    node.body[0].type === "ExpressionStatement" &&
    (node.directives == null || node.directives.length === 0)
  ) {
    return node.body[0].expression;
  }
  return ast;
}

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
  // Cosmetic property-printing flag: `{ a: a }` vs `{ a }`. Key and value are
  // still compared, so ignoring it cannot hide a rename.
  "shorthand",
]);

function babelSignature(node: unknown): unknown {
  if (Array.isArray(node)) {
    // Reprinting drops separator-only statements (`;(foo(), bar())` guards).
    return node
      .filter((entry) => (entry as { type?: string } | null)?.type !== "EmptyStatement")
      .map(babelSignature);
  }
  if (node == null || typeof node !== "object") return node;
  const record = normalizePropertyKeyQuoting(node as Record<string, unknown>);
  if (record.type === "LogicalExpression") {
    // `a && (b && c)` and `(a && b) && c` evaluate identically; the formatter
    // drops the redundant parens, so compare chains as flat operand lists.
    return ["logical-chain", record.operator, flattenLogicalChain(record)];
  }
  if (record.type === "TSParenthesizedType") {
    // `x as (T)` and `x as T` are the same type; parens are print-only.
    return babelSignature(record.typeAnnotation);
  }
  const signature: Record<string, unknown> = {};
  for (const key of Object.keys(record).sort()) {
    if (ignoredBabelKeys.has(key)) continue;
    signature[key] = babelSignature(record[key]);
  }
  return signature;
}

function flattenLogicalChain(node: Record<string, unknown>): unknown[] {
  const operands: unknown[] = [];
  for (const side of [node.left, node.right]) {
    const child = side as Record<string, unknown> | null;
    if (child?.type === "LogicalExpression" && child.operator === node.operator) {
      operands.push(...flattenLogicalChain(child));
    } else {
      operands.push(babelSignature(side));
    }
  }
  return operands;
}

/**
 * `{ 'disabled': x }` and `{ disabled: x }` are the same object literal; the
 * formatter unquotes identifier-safe keys, so compare non-computed string
 * keys as identifiers.
 */
function normalizePropertyKeyQuoting(record: Record<string, unknown>): Record<string, unknown> {
  const key = record.key as { type?: string; value?: unknown } | undefined;
  if (
    (record.type === "ObjectProperty" ||
      record.type === "ObjectMethod" ||
      record.type === "Property") &&
    record.computed === false &&
    key?.type === "StringLiteral" &&
    typeof key.value === "string"
  ) {
    return { ...record, key: { type: "Identifier", name: key.value } };
  }
  return record;
}
