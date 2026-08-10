import { codePointCompare } from "./sfc-equivalence/blocks.ts";

type AstRecord = Record<string, unknown> & { type?: string };

/** Sort only consecutive compiler-map properties proven reorder-safe. */
export function sortPurePropertyRuns(properties: unknown[], normalized: unknown[]): unknown[] {
  const result = [...normalized];
  let start = 0;
  while (start < properties.length) {
    if (!isReorderSafeProperty(properties[start])) {
      start += 1;
      continue;
    }
    let end = start + 1;
    while (end < properties.length && isReorderSafeProperty(properties[end])) end += 1;
    const names = properties
      .slice(start, end)
      .map((property) => staticPropertyName(asRecord(property) as AstRecord));
    if (new Set(names).size === names.length) {
      result.splice(
        start,
        end - start,
        ...result
          .slice(start, end)
          .sort((left, right) => codePointCompare(JSON.stringify(left), JSON.stringify(right))),
      );
    }
    start = end;
  }
  return result;
}

function isReorderSafeProperty(node: unknown): boolean {
  const record = asRecord(node);
  return (
    record?.type === "ObjectProperty" &&
    staticPropertyName(record) != null &&
    isPureExpression(record.value)
  );
}

function isPureExpression(node: unknown): boolean {
  const record = asRecord(node);
  if (record == null) return node == null;
  switch (record.type) {
    case "StringLiteral":
    case "NumericLiteral":
    case "BooleanLiteral":
    case "NullLiteral":
    case "BigIntLiteral":
    case "RegExpLiteral":
    case "ThisExpression":
    case "FunctionExpression":
    case "ArrowFunctionExpression":
      return true;
    case "UnaryExpression":
      return record.operator !== "delete" && isPureExpression(record.argument);
    case "BinaryExpression":
    case "LogicalExpression":
      return isPureExpression(record.left) && isPureExpression(record.right);
    case "ConditionalExpression":
      return (
        isPureExpression(record.test) &&
        isPureExpression(record.consequent) &&
        isPureExpression(record.alternate)
      );
    case "ArrayExpression":
      return arrayField(record, "elements").every(isPureExpression);
    case "ObjectExpression":
      return arrayField(record, "properties").every(isReorderSafeProperty);
    case "TemplateLiteral":
      return arrayField(record, "expressions").every(isPureExpression);
    case "ParenthesizedExpression":
    case "TSAsExpression":
    case "TSTypeAssertion":
    case "TSNonNullExpression":
      return isPureExpression(record.expression);
    default:
      // Identifiers/member reads can invoke Vue instance getters. Calls,
      // constructors, assignments, updates, and spreads are also effectful.
      return false;
  }
}

function staticPropertyName(record: AstRecord): string | null {
  if (record.computed === true) return null;
  const key = asRecord(record.key);
  if (key?.type === "Identifier" && typeof key.name === "string") return key.name;
  if (key?.type === "StringLiteral" && typeof key.value === "string") return key.value;
  if (key?.type === "NumericLiteral" && typeof key.value === "number") return String(key.value);
  return null;
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
