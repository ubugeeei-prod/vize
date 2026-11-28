/** Match documented component inputs to their actual Vue runtime prop declarations. */
import ts from "typescript";
import { memberName, readDoc } from "./ts-scope.ts";

/** Default evidence for one optional public component input. */
export interface ComponentPropDefault {
  /** Public input name, including event-listener spelling. */
  readonly name: string;
  /** Documented literal, or null when the field has no default tag. */
  readonly documented: string | null;
  /** Canonical JavaScript literal or undefined, derived from the Vue declaration. */
  readonly runtime: string;
}

function unwrap(node: ts.Expression): ts.Expression {
  while (
    ts.isAsExpression(node) ||
    ts.isTypeAssertionExpression(node) ||
    ts.isParenthesizedExpression(node) ||
    ts.isSatisfiesExpression(node)
  )
    node = node.expression;
  return node;
}

function properties(node: ts.Expression): Map<string, ts.Expression> {
  node = unwrap(node);
  if (!ts.isObjectLiteralExpression(node))
    throw new Error("Expected literal Vue prop declarations");
  const entries = new Map<string, ts.Expression>();
  for (const property of node.properties) {
    if (!ts.isPropertyAssignment(property) || ts.isComputedPropertyName(property.name))
      throw new Error(
        "Computed or spread Vue prop declarations require an explicit source contract",
      );
    const name = memberName(property.name);
    if (entries.has(name)) throw new Error(`Duplicate Vue prop declaration: ${name}`);
    entries.set(name, property.initializer);
  }
  return entries;
}

function literalDefault(node: ts.Expression): string {
  node = unwrap(node);
  if (ts.isStringLiteralLike(node)) return JSON.stringify(node.text);
  if (ts.isNumericLiteral(node)) return String(Number(node.text));
  if (ts.isIdentifier(node) && node.text === "undefined") return "undefined";
  if (node.kind === ts.SyntaxKind.TrueKeyword) return "true";
  if (node.kind === ts.SyntaxKind.FalseKeyword) return "false";
  if (node.kind === ts.SyntaxKind.NullKeyword) return "null";
  throw new Error(`Nonliteral Vue default requires an explicit source contract: ${node.getText()}`);
}

function constructorDefault(node: ts.Expression | undefined): string {
  if (node === undefined) return "undefined";
  node = unwrap(node);
  if (ts.isIdentifier(node)) {
    if (node.text === "Boolean") return "false";
    if (
      [
        "String",
        "Number",
        "Function",
        "Object",
        "Array",
        "Date",
        "Symbol",
        "BigInt",
        "undefined",
      ].includes(node.text)
    )
      return "undefined";
  }
  if (node.kind === ts.SyntaxKind.NullKeyword) return "undefined";
  if (ts.isArrayLiteralExpression(node)) {
    const defaults = node.elements.map(constructorDefault);
    return defaults.includes("false") ? "false" : "undefined";
  }
  throw new Error(`Computed Vue prop type requires an explicit source contract: ${node.getText()}`);
}

function runtimeDefault(node: ts.Expression): string {
  node = unwrap(node);
  if (ts.isObjectLiteralExpression(node)) {
    const descriptor = properties(node);
    const value = descriptor.get("default");
    if (value !== undefined) return literalDefault(value);
    return constructorDefault(descriptor.get("type"));
  }
  return constructorDefault(node);
}

/**
 * Read optional input defaults from a declared interface and its real component.
 *
 * Literal defaults and Vue's absent-Boolean rule are checked without evaluating
 * application code. Explicit `default: undefined` takes precedence over Boolean
 * casting. Undeclared runtime props and unsupported declaration shapes fail.
 * Optional event-listener fields default to absence; event delivery is a separate
 * runtime contract.
 */
export function componentPropDefaults(
  filename: string,
  sourceText: string,
  interfaceName: string,
  componentName: string,
): readonly ComponentPropDefault[] {
  const source = ts.createSourceFile(filename, sourceText, ts.ScriptTarget.Latest, true);
  const vueFactory = source.statements.some((statement) => {
    if (
      !ts.isImportDeclaration(statement) ||
      !ts.isStringLiteral(statement.moduleSpecifier) ||
      statement.moduleSpecifier.text !== "vue" ||
      statement.importClause?.isTypeOnly
    )
      return false;
    const bindings = statement.importClause?.namedBindings;
    return (
      bindings !== undefined &&
      ts.isNamedImports(bindings) &&
      bindings.elements.some(
        (binding) =>
          !binding.isTypeOnly &&
          binding.name.text === "defineComponent" &&
          (binding.propertyName?.text ?? binding.name.text) === "defineComponent",
      )
    );
  });
  if (!vueFactory) throw new Error("Expected defineComponent imported from Vue");
  const contracts = source.statements.filter(
    (statement): statement is ts.InterfaceDeclaration =>
      ts.isInterfaceDeclaration(statement) && statement.name.text === interfaceName,
  );
  const contract = contracts[0];
  if (contracts.length !== 1 || !contract || contract.heritageClauses?.length)
    throw new Error(`Expected a complete local prop interface: ${interfaceName}`);
  const declaration = source.statements
    .filter(ts.isVariableStatement)
    .flatMap((statement) => [...statement.declarationList.declarations])
    .find((declaration) => declaration.name.getText(source) === componentName);
  const initializer = declaration?.initializer && unwrap(declaration.initializer);
  if (
    !initializer ||
    !ts.isCallExpression(initializer) ||
    !ts.isIdentifier(initializer.expression) ||
    initializer.expression.text !== "defineComponent" ||
    initializer.arguments.length !== 1
  )
    throw new Error(`Expected a literal Vue component: ${componentName}`);
  const options = unwrap(initializer.arguments[0]!);
  if (
    !ts.isObjectLiteralExpression(options) ||
    options.properties.some(
      (property) => ts.isSpreadAssignment(property) || ts.isComputedPropertyName(property.name),
    )
  )
    throw new Error(`Expected explicit Vue component options: ${componentName}`);
  if (
    options.properties.some(
      (property) =>
        !ts.isSpreadAssignment(property) &&
        ["mixins", "extends"].includes(memberName(property.name)),
    )
  )
    throw new Error(`Inherited Vue props require an explicit source contract: ${componentName}`);
  const declarations = options.properties.filter(
    (property) => !ts.isSpreadAssignment(property) && memberName(property.name) === "props",
  );
  const props = declarations[0];
  if (declarations.length !== 1 || !props || !ts.isPropertyAssignment(props))
    throw new Error(`Expected one Vue prop declaration: ${componentName}`);
  const runtime = properties(props.initializer);
  const members = new Map<string, ts.PropertySignature>();
  for (const member of contract.members) {
    if (!ts.isPropertySignature(member) || ts.isComputedPropertyName(member.name))
      throw new Error(`Expected named public props: ${interfaceName}`);
    const name = memberName(member.name);
    if (members.has(name)) throw new Error(`Duplicate public prop: ${interfaceName}.${name}`);
    members.set(name, member);
  }
  for (const name of runtime.keys())
    if (!members.has(name)) throw new Error(`Undocumented runtime prop: ${componentName}.${name}`);
  return [...members].flatMap(([name, member]) => {
    if (!member.questionToken) return [];
    const implementation = runtime.get(name);
    if (!implementation && !/^on[A-Z]/u.test(name))
      throw new Error(`Missing runtime prop declaration: ${componentName}.${name}`);
    return [
      {
        name,
        documented: readDoc(member).defaultValue,
        runtime: implementation ? runtimeDefault(implementation) : "undefined",
      },
    ];
  });
}
