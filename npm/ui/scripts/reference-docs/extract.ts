/**
 * Static API extraction for the generated reference docs.
 *
 * Reads `<script setup>` blocks and TypeScript modules with the TypeScript
 * parser (no type checker, no emit) and collects props, emits, slots, and
 * exposed members together with their TSDoc summaries and `@default` tags.
 */
import { readFileSync } from "node:fs";
import path from "node:path";

import ts from "typescript";

import type {
  ApiMember,
  ComponentApi,
  ModuleExport,
  ModuleInterface,
  ModuleTypeAlias,
} from "./api-types.ts";
import { emitMembers, namedTypeMembers, slotMembers, typeMembers } from "./members.ts";
import { TypeScope, commentText, memberName, oneLine, parse, readDoc } from "./ts-scope.ts";

export type {
  ApiMember,
  ComponentApi,
  ModuleExport,
  ModuleInterface,
  ModuleTypeAlias,
} from "./api-types.ts";

function scriptBlocks(source: string): { setup: string; generic: string | null; other: string } {
  let setup = "";
  let other = "";
  let generic: string | null = null;
  for (const match of source.matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script>/g)) {
    const attributes = match[1] ?? "";
    if (/\bsetup\b/.test(attributes)) {
      setup += match[2] ?? "";
      generic = /\bgeneric="([^"]*)"/.exec(attributes)?.[1] ?? generic;
    } else {
      other += match[2] ?? "";
    }
  }
  return { setup, generic, other };
}

/**
 * Resolves type names used by one SFC: local declarations first, then
 * relative imports (followed through `export … from` re-exports).
 */

function visit(node: ts.Node, callback: (node: ts.Node) => void): void {
  callback(node);
  node.forEachChild((child) => visit(child, callback));
}

function destructuredDefaults(setup: ts.SourceFile): Map<string, string> {
  const defaults = new Map<string, string>();
  visit(setup, (node) => {
    if (
      ts.isVariableDeclaration(node) &&
      ts.isObjectBindingPattern(node.name) &&
      node.initializer != null &&
      /\bdefineProps\b/.test(node.initializer.getText())
    ) {
      for (const element of node.name.elements) {
        const key = element.propertyName ?? element.name;
        if (element.initializer != null && ts.isIdentifier(key)) {
          defaults.set(key.text, oneLine(element.initializer.getText()));
        }
      }
    }
  });
  return defaults;
}

function exposeMembers(
  setup: ts.SourceFile,
  argument: ts.Expression | undefined,
  scope: TypeScope,
  exportName: string,
): readonly ApiMember[] {
  let literal: ts.ObjectLiteralExpression | undefined;
  if (argument != null && ts.isObjectLiteralExpression(argument)) literal = argument;
  if (argument != null && ts.isIdentifier(argument)) {
    visit(setup, (node) => {
      if (
        ts.isVariableDeclaration(node) &&
        ts.isIdentifier(node.name) &&
        node.name.text === argument.text &&
        node.initializer != null
      ) {
        let initializer: ts.Expression = node.initializer;
        while (ts.isSatisfiesExpression(initializer) || ts.isAsExpression(initializer)) {
          initializer = initializer.expression;
        }
        if (ts.isObjectLiteralExpression(initializer)) literal = initializer;
      }
    });
  }
  const publicMembers = namedTypeMembers(`${exportName}Expose`, scope) ?? [];
  const names =
    literal?.properties.map((property) => memberName(property.name)).filter(Boolean) ?? [];
  const extras = names
    .filter((name) => !publicMembers.some((member) => member.name === name))
    .map((name) => ({
      name,
      type: name === "element" ? "Readonly<ShallowRef<HTMLElement | null>>" : "—",
      optional: false,
      description: name === "element" ? "Template ref to the rendered root element." : "",
      defaultValue: null,
      deprecated: false,
    }));
  return [...publicMembers, ...extras];
}

/** Extract props, emits, slots, and exposed members from one SFC. */
export function extractComponentApi(file: string, exportName: string): ComponentApi {
  const source = readFileSync(file, "utf8");
  const blocks = scriptBlocks(source);
  const setup = parse(`${file}.setup.ts`, blocks.setup);
  const other = parse(`${file}.script.ts`, blocks.other);
  const scope = new TypeScope([setup, other], path.dirname(file));
  const defaults = destructuredDefaults(setup);
  let props: readonly ApiMember[] = [];
  let emits: readonly ApiMember[] = [];
  let slots: readonly ApiMember[] = [];
  let expose: readonly ApiMember[] = [];
  visit(setup, (node) => {
    if (!ts.isCallExpression(node) || !ts.isIdentifier(node.expression)) return;
    const typeArgument = node.typeArguments?.[0];
    switch (node.expression.text) {
      case "defineProps":
        props = typeMembers(typeArgument, scope, defaults);
        break;
      case "defineEmits":
        emits = emitMembers(typeArgument, scope);
        break;
      case "defineSlots":
        slots = slotMembers(typeArgument, scope);
        break;
      case "defineExpose":
        expose = exposeMembers(setup, node.arguments[0], scope, exportName);
        break;
    }
  });
  return { file, generic: blocks.generic, props, emits, slots, expose };
}

/** `export { default as Name } from "./file.vue"` pairs of a family entry. */
export function componentExports(entryFile: string): readonly { name: string; file: string }[] {
  const source = parse(entryFile);
  const components: { name: string; file: string }[] = [];
  for (const statement of source.statements) {
    if (
      ts.isExportDeclaration(statement) &&
      statement.moduleSpecifier != null &&
      ts.isStringLiteral(statement.moduleSpecifier) &&
      statement.moduleSpecifier.text.endsWith(".vue") &&
      statement.exportClause != null &&
      ts.isNamedExports(statement.exportClause)
    ) {
      for (const element of statement.exportClause.elements) {
        if (element.propertyName?.text === "default") {
          components.push({
            name: element.name.text,
            file: path.resolve(path.dirname(entryFile), statement.moduleSpecifier.text),
          });
        }
      }
    }
  }
  return components;
}

/** Leading TSDoc of a module (its first statement's comment). */
export function moduleSummary(file: string): string {
  const source = parse(file);
  const first = source.statements[0];
  return first == null ? "" : readDoc(first).description;
}

function examples(node: ts.Node): readonly string[] {
  return ts
    .getJSDocTags(node)
    .filter((tag) => tag.tagName.text === "example")
    .map((tag) => commentText(tag.comment).trim())
    .filter((text) => text.length > 0);
}

/** Exported interfaces of a module with their documented members. */
export function moduleInterfaces(file: string): readonly ModuleInterface[] {
  const source = parse(file);
  const scope = new TypeScope([source], path.dirname(file));
  return source.statements.filter(ts.isInterfaceDeclaration).flatMap((statement) => {
    const exported = (ts.getModifiers(statement) ?? []).some(
      (modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword,
    );
    if (!exported) return [];
    return [
      {
        name: statement.name.text,
        description: readDoc(statement).description,
        members: namedTypeMembers(statement.name.text, scope) ?? [],
      },
    ];
  });
}

/** Exported aliases, needed by entries whose public surface is types only. */
export function moduleTypeAliases(file: string): readonly ModuleTypeAlias[] {
  const source = parse(file);
  return source.statements.filter(ts.isTypeAliasDeclaration).flatMap((statement) => {
    const exported = (ts.getModifiers(statement) ?? []).some(
      (modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword,
    );
    if (!exported) return [];
    return [
      {
        name: statement.name.text,
        signature: oneLine(statement.getText().replace(/^export\s+/, "")).replace(/;$/, ""),
        description: readDoc(statement).description,
      },
    ];
  });
}

/** Exported functions and constants of a module, with TSDoc summaries. */
export function moduleExports(file: string): readonly ModuleExport[] {
  const source = parse(file);
  const exports: ModuleExport[] = [];
  const exported = (node: ts.Node): boolean =>
    ts.canHaveModifiers(node) &&
    (ts.getModifiers(node) ?? []).some((modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword);
  for (const statement of source.statements) {
    if (ts.isFunctionDeclaration(statement) && statement.name != null && exported(statement)) {
      const header =
        statement.body == null
          ? statement.getText()
          : statement.getText().slice(0, statement.body.getStart() - statement.getStart());
      exports.push({
        name: statement.name.text,
        signature: oneLine(header.replace(/^export\s+/, "")).replace(/;$/, ""),
        description: readDoc(statement).description,
        examples: examples(statement),
      });
    } else if (ts.isVariableStatement(statement) && exported(statement)) {
      for (const declaration of statement.declarationList.declarations) {
        if (!ts.isIdentifier(declaration.name)) continue;
        exports.push({
          name: declaration.name.text,
          signature: `const ${declaration.name.text}${
            declaration.type == null ? "" : `: ${oneLine(declaration.type.getText())}`
          }`,
          description: readDoc(statement).description,
          examples: examples(statement),
        });
      }
    } else if (ts.isClassDeclaration(statement) && statement.name != null && exported(statement)) {
      exports.push({
        name: statement.name.text,
        signature: `class ${statement.name.text}`,
        description: readDoc(statement).description,
        examples: examples(statement),
      });
    }
  }
  return exports;
}
