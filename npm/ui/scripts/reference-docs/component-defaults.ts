/** Match documented component inputs to their actual Vue runtime prop declarations. */
import ts from "typescript";
import { commentText, memberName } from "./ts-scope.ts";

/** Default evidence for one optional public component input. */
export interface ComponentPropDefault {
  /** Public input name, including event-listener spelling. */
  readonly name: string;
  /** Documented literal, or null when the field has no default tag. */
  readonly documented: string | null;
  /** Canonical JavaScript literal or undefined, derived from the Vue declaration. */
  readonly runtime: string;
}

const propBuiltins = new Set([
  "String",
  "Number",
  "Boolean",
  "Function",
  "Object",
  "Array",
  "Date",
  "Symbol",
  "BigInt",
  "undefined",
]);

function bindingNames(name: ts.BindingName): readonly string[] {
  return ts.isIdentifier(name)
    ? [name.text]
    : name.elements.flatMap((element) =>
        ts.isOmittedExpression(element) ? [] : bindingNames(element.name),
      );
}

function requireGlobalPropBuiltins(source: ts.SourceFile): void {
  const names = source.statements.flatMap((statement): readonly string[] => {
    if (ts.isVariableStatement(statement))
      return statement.declarationList.declarations.flatMap((declaration) =>
        bindingNames(declaration.name),
      );
    if (ts.isImportDeclaration(statement)) {
      const clause = statement.importClause;
      if (!clause || clause.isTypeOnly) return [];
      const bindings = clause.namedBindings;
      return [
        ...(clause.name ? [clause.name.text] : []),
        ...(bindings && ts.isNamespaceImport(bindings) ? [bindings.name.text] : []),
        ...(bindings && ts.isNamedImports(bindings)
          ? bindings.elements
              .filter((binding) => !binding.isTypeOnly)
              .map((binding) => binding.name.text)
          : []),
      ];
    }
    if (
      ts.isFunctionDeclaration(statement) ||
      ts.isClassDeclaration(statement) ||
      ts.isEnumDeclaration(statement) ||
      ts.isModuleDeclaration(statement) ||
      ts.isImportEqualsDeclaration(statement)
    )
      return statement.name ? [statement.name.text] : [];
    return [];
  });
  const collectHoisted = (node: ts.Node): void => {
    if (
      ts.isFunctionLike(node) ||
      ts.isClassDeclaration(node) ||
      ts.isClassExpression(node) ||
      ts.isModuleDeclaration(node)
    )
      return;
    if (ts.isVariableDeclarationList(node) && !(node.flags & ts.NodeFlags.BlockScoped))
      for (const declaration of node.declarations) names.push(...bindingNames(declaration.name));
    ts.forEachChild(node, collectHoisted);
  };
  collectHoisted(source);
  for (const name of names)
    if (propBuiltins.has(name)) throw new Error(`Shadowed Vue prop builtin: ${name}`);
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
    if (propBuiltins.has(node.text)) return "undefined";
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
    if (value !== undefined) {
      const expression = unwrap(value);
      if (ts.isArrowFunction(expression)) {
        const type = descriptor.get("type");
        const constructor = type && unwrap(type);
        const result = ts.isBlock(expression.body) ? undefined : unwrap(expression.body);
        if (
          expression.parameters.length === 0 &&
          !expression.modifiers?.length &&
          !descriptor.has("skipFactory") &&
          constructor &&
          ts.isIdentifier(constructor) &&
          result
        ) {
          if (
            constructor.text === "Array" &&
            ts.isArrayLiteralExpression(result) &&
            result.elements.length === 0
          )
            return "[]";
          if (
            constructor.text === "Object" &&
            ts.isObjectLiteralExpression(result) &&
            result.properties.length === 0
          )
            return "{}";
        }
        throw new Error(`Unsupported Vue default factory: ${expression.getText()}`);
      }
      return literalDefault(expression);
    }
    return constructorDefault(descriptor.get("type"));
  }
  return constructorDefault(node);
}

function documentedDefault(member: ts.PropertySignature): string | null {
  const tags = ts
    .getJSDocTags(member)
    .filter((tag) => tag.tagName.text === "default" || tag.tagName.text === "defaultValue");
  if (tags.length > 1) throw new Error(`Duplicate public prop default: ${memberName(member.name)}`);
  // Preserve spaces within quoted literals, including a two-space empty indicator.
  return tags[0] ? commentText(tags[0].comment).trim() : null;
}

/**
 * Read optional input defaults from a declared interface and its real component.
 *
 * Literal defaults, fresh empty Array/Object factories, and Vue's absent-Boolean
 * rule are checked without evaluating application code. Explicit `default:
 * undefined` takes precedence over Boolean casting. Undeclared runtime props and
 * unsupported declaration shapes fail.
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
  requireGlobalPropBuiltins(source);
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
        documented: documentedDefault(member),
        runtime: implementation ? runtimeDefault(implementation) : "undefined",
      },
    ];
  });
}
