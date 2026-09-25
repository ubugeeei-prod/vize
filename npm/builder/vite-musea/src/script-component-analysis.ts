/** Read a script component's runtime options without executing project code. */

import ts from "typescript";

export interface ScriptComponentAnalysis {
  props: Array<{ name: string; type: string; required: boolean; default_value?: unknown }>;
  emits: string[];
}

function scriptKind(filename: string): ts.ScriptKind {
  if (filename.endsWith(".tsx")) return ts.ScriptKind.TSX;
  if (filename.endsWith(".jsx")) return ts.ScriptKind.JSX;
  if (filename.endsWith(".js")) return ts.ScriptKind.JS;
  return ts.ScriptKind.TS;
}

function unwrap(expression: ts.Expression): ts.Expression {
  while (
    ts.isParenthesizedExpression(expression) ||
    ts.isAsExpression(expression) ||
    ts.isSatisfiesExpression(expression) ||
    ts.isNonNullExpression(expression)
  ) {
    expression = expression.expression;
  }
  return expression;
}

function nameOf(name: ts.PropertyName): string | undefined {
  if (ts.isIdentifier(name) || ts.isStringLiteral(name) || ts.isNumericLiteral(name)) {
    return name.text;
  }
  if (ts.isComputedPropertyName(name) && ts.isStringLiteral(name.expression)) {
    return name.expression.text;
  }
  return undefined;
}

function variable(sourceFile: ts.SourceFile, name: string): ts.Expression | undefined {
  for (const statement of sourceFile.statements) {
    if (!ts.isVariableStatement(statement)) continue;
    for (const declaration of statement.declarationList.declarations) {
      if (ts.isIdentifier(declaration.name) && declaration.name.text === name) {
        return declaration.initializer;
      }
    }
  }
  return undefined;
}

function resolved(expression: ts.Expression, sourceFile: ts.SourceFile): ts.Expression {
  expression = unwrap(expression);
  if (ts.isIdentifier(expression)) {
    const initializer = variable(sourceFile, expression.text);
    if (initializer) return unwrap(initializer);
  }
  return expression;
}

function option(
  object: ts.ObjectLiteralExpression,
  name: string,
  sourceFile: ts.SourceFile,
): ts.Expression | undefined {
  for (const property of object.properties) {
    if (ts.isPropertyAssignment(property) && nameOf(property.name) === name) {
      return resolved(property.initializer, sourceFile);
    }
    if (ts.isShorthandPropertyAssignment(property) && property.name.text === name) {
      const initializer = variable(sourceFile, name);
      if (initializer) return resolved(initializer, sourceFile);
    }
  }
  return undefined;
}

function componentOptions(sourceFile: ts.SourceFile): ts.ObjectLiteralExpression | undefined {
  const assignment = sourceFile.statements.find((statement): statement is ts.ExportAssignment =>
    ts.isExportAssignment(statement),
  );
  if (!assignment || assignment.isExportEquals) return undefined;

  let expression = resolved(assignment.expression, sourceFile);
  if (ts.isCallExpression(expression)) {
    const callee = expression.expression;
    const isDefineComponent =
      (ts.isIdentifier(callee) && callee.text === "defineComponent") ||
      (ts.isPropertyAccessExpression(callee) && callee.name.text === "defineComponent");
    if (!isDefineComponent || !expression.arguments[0]) return undefined;
    expression = resolved(expression.arguments[0], sourceFile);
  }
  return ts.isObjectLiteralExpression(expression) ? expression : undefined;
}

function literal(expression: ts.Expression): unknown {
  expression = unwrap(expression);
  if (ts.isStringLiteral(expression) || ts.isNoSubstitutionTemplateLiteral(expression)) {
    return expression.text;
  }
  if (ts.isNumericLiteral(expression)) return Number(expression.text);
  if (expression.kind === ts.SyntaxKind.TrueKeyword) return true;
  if (expression.kind === ts.SyntaxKind.FalseKeyword) return false;
  if (expression.kind === ts.SyntaxKind.NullKeyword) return null;
  if (ts.isPrefixUnaryExpression(expression) && ts.isNumericLiteral(expression.operand)) {
    const value = Number(expression.operand.text);
    if (expression.operator === ts.SyntaxKind.MinusToken) return -value;
    if (expression.operator === ts.SyntaxKind.PlusToken) return value;
  }
  return undefined;
}

function propType(expression: ts.Expression | undefined): string {
  if (!expression) return "unknown";
  expression = unwrap(expression);
  if (ts.isArrayLiteralExpression(expression)) {
    return expression.elements.map((element) => propType(element)).join(" | ") || "unknown";
  }
  const name = ts.isIdentifier(expression)
    ? expression.text
    : ts.isPropertyAccessExpression(expression)
      ? expression.name.text
      : undefined;
  switch (name) {
    case "String":
      return "string";
    case "Number":
      return "number";
    case "Boolean":
      return "boolean";
    case "Array":
      return "unknown[]";
    case "Object":
      return "Record<string, unknown>";
    default:
      return "unknown";
  }
}

function analyzeProps(
  expression: ts.Expression | undefined,
  sourceFile: ts.SourceFile,
): ScriptComponentAnalysis["props"] {
  if (!expression) return [];
  if (ts.isArrayLiteralExpression(expression)) {
    return expression.elements.flatMap((element) =>
      ts.isStringLiteral(element) ? [{ name: element.text, type: "unknown", required: false }] : [],
    );
  }
  if (!ts.isObjectLiteralExpression(expression)) return [];

  const props: ScriptComponentAnalysis["props"] = [];
  for (const property of expression.properties) {
    if (!ts.isPropertyAssignment(property)) continue;
    const name = nameOf(property.name);
    if (!name) continue;
    const value = resolved(property.initializer, sourceFile);
    if (ts.isObjectLiteralExpression(value)) {
      const type = propType(option(value, "type", sourceFile));
      const required = option(value, "required", sourceFile)?.kind === ts.SyntaxKind.TrueKeyword;
      const defaultExpression = option(value, "default", sourceFile);
      const defaultValue = defaultExpression ? literal(defaultExpression) : undefined;
      props.push({
        name,
        type,
        required,
        ...(defaultValue !== undefined ? { default_value: defaultValue } : {}),
      });
    } else {
      props.push({ name, type: propType(value), required: false });
    }
  }
  return props;
}

function analyzeEmits(expression: ts.Expression | undefined): string[] {
  if (!expression) return [];
  if (ts.isArrayLiteralExpression(expression)) {
    return expression.elements.flatMap((element) =>
      ts.isStringLiteral(element) ? [element.text] : [],
    );
  }
  if (ts.isObjectLiteralExpression(expression)) {
    return expression.properties.flatMap((property) =>
      (ts.isPropertyAssignment(property) || ts.isMethodDeclaration(property)) &&
      nameOf(property.name)
        ? [nameOf(property.name)!]
        : [],
    );
  }
  return [];
}

export function analyzeScriptComponent(source: string, filename: string): ScriptComponentAnalysis {
  const sourceFile = ts.createSourceFile(
    filename,
    source,
    ts.ScriptTarget.Latest,
    true,
    scriptKind(filename),
  );
  const options = componentOptions(sourceFile);
  if (!options) return { props: [], emits: [] };
  return {
    props: analyzeProps(option(options, "props", sourceFile), sourceFile),
    emits: analyzeEmits(option(options, "emits", sourceFile)),
  };
}
