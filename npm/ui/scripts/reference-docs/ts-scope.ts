/** TypeScript parsing, TSDoc reading, and cross-file type-name resolution. */
import { readFileSync } from "node:fs";
import path from "node:path";

import ts from "typescript";

export type Declaration = ts.InterfaceDeclaration | ts.TypeAliasDeclaration;

const sourceCache = new Map<string, ts.SourceFile>();

export function parse(file: string, text?: string): ts.SourceFile {
  const cached = text == null ? sourceCache.get(file) : undefined;
  if (cached != null) return cached;
  const source = ts.createSourceFile(
    file,
    text ?? readFileSync(file, "utf8"),
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  if (text == null) sourceCache.set(file, source);
  return source;
}

export function oneLine(text: string): string {
  return text.replace(/\s+/g, " ").trim();
}

export function commentText(comment: string | ts.NodeArray<ts.JSDocComment> | undefined): string {
  if (comment == null) return "";
  if (typeof comment === "string") return comment;
  return comment
    .map((part) => {
      if (ts.isJSDocLink(part) || ts.isJSDocLinkCode(part) || ts.isJSDocLinkPlain(part)) {
        const target = part.name == null ? "" : part.name.getText();
        const label = part.text.trim();
        return `\`${label === "" ? target : label}\``;
      }
      return part.text;
    })
    .join("");
}

/** TSDoc summary, `@default`, and `@deprecated` of a node. */
export function readDoc(node: ts.Node): {
  description: string;
  defaultValue: string | null;
  deprecated: boolean;
} {
  let description = "";
  let defaultValue: string | null = null;
  let deprecated = false;
  for (const doc of ts.getJSDocCommentsAndTags(node)) {
    if (ts.isJSDoc(doc)) {
      description = oneLine(commentText(doc.comment));
      for (const tag of doc.tags ?? []) {
        const name = tag.tagName.text;
        if (name === "default" || name === "defaultValue") {
          defaultValue = oneLine(commentText(tag.comment));
        } else if (name === "deprecated") {
          deprecated = true;
        }
      }
    }
  }
  return { description, defaultValue, deprecated };
}

export function memberName(name: ts.PropertyName | undefined): string {
  if (name == null) return "";
  if (ts.isIdentifier(name) || ts.isStringLiteral(name) || ts.isNumericLiteral(name)) {
    return name.text;
  }
  return name.getText();
}

export class TypeScope {
  readonly #locals = new Map<string, Declaration>();
  readonly #imports = new Map<string, { file: string; name: string }>();

  constructor(sources: readonly ts.SourceFile[], directory: string) {
    for (const source of sources) {
      for (const statement of source.statements) {
        if (ts.isInterfaceDeclaration(statement) || ts.isTypeAliasDeclaration(statement)) {
          this.#locals.set(statement.name.text, statement);
        }
        if (
          ts.isImportDeclaration(statement) &&
          ts.isStringLiteral(statement.moduleSpecifier) &&
          statement.moduleSpecifier.text.startsWith(".")
        ) {
          const file = path.resolve(directory, statement.moduleSpecifier.text);
          const bindings = statement.importClause?.namedBindings;
          if (bindings != null && ts.isNamedImports(bindings)) {
            for (const element of bindings.elements) {
              this.#imports.set(element.name.text, {
                file,
                name: element.propertyName?.text ?? element.name.text,
              });
            }
          }
        }
      }
    }
  }

  resolve(name: string): Declaration | undefined {
    return this.resolveWithScope(name)?.declaration;
  }

  /** The declaration plus the scope its own type references resolve in. */
  resolveWithScope(name: string): { declaration: Declaration; scope: TypeScope } | undefined {
    const local = this.#locals.get(name);
    if (local != null) return { declaration: local, scope: this };
    const imported = this.#imports.get(name);
    const declaration =
      imported == null ? undefined : findExported(imported.file, imported.name, 0);
    return declaration == null
      ? undefined
      : { declaration, scope: scopeForFile(declaration.getSourceFile().fileName) };
  }
}

const fileScopes = new Map<string, TypeScope>();

export function scopeForFile(file: string): TypeScope {
  let scope = fileScopes.get(file);
  if (scope == null) {
    scope = new TypeScope([parse(file)], path.dirname(file));
    fileScopes.set(file, scope);
  }
  return scope;
}

export function findExported(file: string, name: string, depth: number): Declaration | undefined {
  if (depth > 4 || !file.endsWith(".ts")) return undefined;
  let source: ts.SourceFile;
  try {
    source = parse(file);
  } catch {
    return undefined;
  }
  for (const statement of source.statements) {
    if (
      (ts.isInterfaceDeclaration(statement) || ts.isTypeAliasDeclaration(statement)) &&
      statement.name.text === name
    ) {
      return statement;
    }
    if (
      ts.isExportDeclaration(statement) &&
      statement.moduleSpecifier != null &&
      ts.isStringLiteral(statement.moduleSpecifier) &&
      statement.exportClause != null &&
      ts.isNamedExports(statement.exportClause)
    ) {
      const element = statement.exportClause.elements.find(
        (candidate) => candidate.name.text === name,
      );
      if (element != null) {
        const target = path.resolve(path.dirname(file), statement.moduleSpecifier.text);
        return findExported(target, element.propertyName?.text ?? name, depth + 1);
      }
    }
    if (ts.isImportDeclaration(statement) && ts.isStringLiteral(statement.moduleSpecifier)) {
      const bindings = statement.importClause?.namedBindings;
      if (bindings != null && ts.isNamedImports(bindings)) {
        const element = bindings.elements.find((candidate) => candidate.name.text === name);
        if (element != null && statement.moduleSpecifier.text.startsWith(".")) {
          const target = path.resolve(path.dirname(file), statement.moduleSpecifier.text);
          return findExported(target, element.propertyName?.text ?? name, depth + 1);
        }
      }
    }
  }
  return undefined;
}
