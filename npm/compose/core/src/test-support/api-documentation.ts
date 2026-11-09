import { existsSync, readdirSync, readFileSync } from "node:fs";
import { isAbsolute, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";

import ts from "typescript";

export interface DocumentationAudit {
  readonly declarations: number;
  readonly optionalMembers: number;
  readonly problems: readonly string[];
}

function hasExportModifier(node: ts.Node): boolean {
  return ts.canHaveModifiers(node)
    ? (ts.getModifiers(node)?.some((modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword) ??
        false)
    : false;
}

function directDocumentation(node: ts.Node, source: ts.SourceFile): ts.JSDoc | undefined {
  const blocks = (node as ts.Node & { readonly jsDoc?: ts.NodeArray<ts.JSDoc> }).jsDoc;
  const documentation = blocks?.at(-1);
  if (!documentation) return undefined;

  const documentationLine = source.getLineAndCharacterOfPosition(documentation.end).line;
  const declarationLine = source.getLineAndCharacterOfPosition(node.getStart(source)).line;
  return declarationLine === documentationLine + 1 ? documentation : undefined;
}

function lineNumber(source: ts.SourceFile, node: ts.Node): number {
  return source.getLineAndCharacterOfPosition(node.getStart(source)).line + 1;
}

function declarationNames(statement: ts.Statement): readonly string[] | undefined {
  if (ts.isVariableStatement(statement)) {
    const names: string[] = [];
    const collect = (binding: ts.BindingName): void => {
      if (ts.isIdentifier(binding)) {
        names.push(binding.text);
        return;
      }
      for (const element of binding.elements) {
        if (!ts.isOmittedExpression(element)) collect(element.name);
      }
    };
    for (const declaration of statement.declarationList.declarations) collect(declaration.name);
    return names;
  }
  if (
    ts.isFunctionDeclaration(statement) ||
    ts.isClassDeclaration(statement) ||
    ts.isInterfaceDeclaration(statement) ||
    ts.isTypeAliasDeclaration(statement) ||
    ts.isEnumDeclaration(statement)
  ) {
    return [statement.name?.text ?? "default"];
  }
  return undefined;
}

function isValueDeclaration(statement: ts.Statement): boolean {
  return (
    ts.isVariableStatement(statement) ||
    ts.isFunctionDeclaration(statement) ||
    ts.isClassDeclaration(statement) ||
    ts.isEnumDeclaration(statement)
  );
}

function propertyName(member: ts.TypeElement, source: ts.SourceFile): string {
  if (!("name" in member) || !member.name) return "unknown";
  return member.name.getText(source);
}

function optionsMembers(statement: ts.Statement): readonly ts.TypeElement[] | undefined {
  if (ts.isInterfaceDeclaration(statement) && statement.name.text.endsWith("Options")) {
    return statement.members;
  }
  if (
    ts.isTypeAliasDeclaration(statement) &&
    statement.name.text.endsWith("Options") &&
    ts.isTypeLiteralNode(statement.type)
  ) {
    return statement.type.members;
  }
  return undefined;
}

export function auditDocumentation(file: string, sourceText: string): DocumentationAudit {
  const source = ts.createSourceFile(
    file,
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  const problems: string[] = [];
  const seenFunctionNames = new Set<string>();
  let declarations = 0;
  let optionalMembers = 0;

  const valueDeclarationsByName = new Map<string, ts.Statement>();
  for (const statement of source.statements) {
    if (!isValueDeclaration(statement)) continue;
    for (const name of declarationNames(statement) ?? []) {
      if (name !== "default" && !valueDeclarationsByName.has(name)) {
        valueDeclarationsByName.set(name, statement);
      }
    }
  }

  for (const statement of source.statements) {
    if (!hasExportModifier(statement)) continue;

    const names = declarationNames(statement);
    if (names) {
      declarations += names.length;
      const namesNeedingDocumentation = ts.isFunctionDeclaration(statement)
        ? names.filter((name) => !seenFunctionNames.has(name))
        : names;
      if (ts.isFunctionDeclaration(statement)) {
        for (const name of names) seenFunctionNames.add(name);
      }
      if (
        namesNeedingDocumentation.length > 0 &&
        directDocumentation(statement, source) === undefined
      ) {
        for (const name of namesNeedingDocumentation) {
          problems.push(
            `${file}:${lineNumber(source, statement)} export "${name}" has no JSDoc block directly above`,
          );
        }
      }
    }

    const members = optionsMembers(statement);
    if (!members) continue;
    for (const member of members) {
      if (!("questionToken" in member) || !member.questionToken) continue;
      optionalMembers += 1;
      const documentation = directDocumentation(member, source);
      if (!documentation?.getText(source).includes("@default")) {
        problems.push(
          `${file}:${lineNumber(source, member)} optional option "${propertyName(member, source)}" must document @default`,
        );
      }
    }
  }

  for (const statement of source.statements) {
    if (!ts.isExportAssignment(statement) || !ts.isIdentifier(statement.expression)) continue;
    const declaration = valueDeclarationsByName.get(statement.expression.text);
    if (declaration == null || hasExportModifier(declaration)) continue;

    declarations += 1;
    if (directDocumentation(declaration, source) === undefined) {
      problems.push(
        `${file}:${lineNumber(source, declaration)} export "${statement.expression.text}" has no JSDoc block directly above`,
      );
    }
  }

  return { declarations, optionalMembers, problems };
}

export function auditSourceDirectory(sourceDirectory: URL): DocumentationAudit {
  const totals = { declarations: 0, optionalMembers: 0, problems: [] as string[] };
  for (const file of readdirSync(sourceDirectory).sort()) {
    if (!file.endsWith(".ts") || file.endsWith(".test.ts") || file.endsWith(".test-d.ts")) {
      continue;
    }
    const audit = auditDocumentation(file, readFileSync(new URL(file, sourceDirectory), "utf8"));
    totals.declarations += audit.declarations;
    totals.optionalMembers += audit.optionalMembers;
    totals.problems.push(...audit.problems);
  }
  return totals;
}

function relativeReexports(file: URL, sourceText: string): readonly URL[] {
  const source = ts.createSourceFile(
    fileURLToPath(file),
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  const reexports: URL[] = [];

  for (const statement of source.statements) {
    if (!ts.isExportDeclaration(statement) || !statement.moduleSpecifier) continue;
    if (!ts.isStringLiteral(statement.moduleSpecifier)) continue;
    const specifier = statement.moduleSpecifier.text;
    if (!specifier.startsWith(".")) continue;

    const target = new URL(specifier, file);
    const candidates =
      specifier.endsWith(".d.ts") || specifier.endsWith(".ts")
        ? [target]
        : specifier.endsWith(".js")
          ? [
              new URL(`${specifier.slice(0, -3)}.ts`, file),
              new URL(`${specifier.slice(0, -3)}.d.ts`, file),
            ]
          : [
              new URL(`${specifier}.ts`, file),
              new URL(`${specifier}.d.ts`, file),
              new URL(`${specifier}/index.ts`, file),
              new URL(`${specifier}/index.d.ts`, file),
            ];
    const resolved = candidates.find((candidate) => existsSync(candidate));
    if (resolved) reexports.push(resolved);
  }

  return reexports;
}

/** Audits every TypeScript module reachable through relative public re-exports. */
export function auditSourceGraph(entrypoints: readonly URL[], sourceRoot: URL): DocumentationAudit {
  const totals = { declarations: 0, optionalMembers: 0, problems: [] as string[] };
  const pending = [...entrypoints];
  const visited = new Set<string>();

  while (pending.length > 0) {
    const file = pending.pop();
    if (!file) break;
    const absolutePath = fileURLToPath(file);
    if (visited.has(absolutePath)) continue;
    visited.add(absolutePath);

    const relativePath = relative(fileURLToPath(sourceRoot), absolutePath);
    if (relativePath === "" || relativePath === ".." || relativePath.startsWith(`..${sep}`)) {
      throw new Error(`public source graph escaped its source root: ${absolutePath}`);
    }
    if (isAbsolute(relativePath)) {
      throw new Error(`public source graph resolved an absolute relative path: ${absolutePath}`);
    }
    const fileName = relativePath.replaceAll(sep, "/");
    const sourceText = readFileSync(file, "utf8");
    const audit = auditDocumentation(fileName, sourceText);
    totals.declarations += audit.declarations;
    totals.optionalMembers += audit.optionalMembers;
    totals.problems.push(...audit.problems);
    pending.push(...relativeReexports(file, sourceText));
  }

  totals.problems.sort();
  return totals;
}
