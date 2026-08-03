import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { test } from "node:test";

import ts from "typescript";

const sourceDirectory = new URL(".", import.meta.url);

interface DocumentationAudit {
  readonly declarations: number;
  readonly optionalMembers: number;
  readonly problems: readonly string[];
}

function listSourceFiles(): readonly string[] {
  return readdirSync(sourceDirectory)
    .filter((name) => name.endsWith(".ts") && !name.endsWith(".test.ts"))
    .sort();
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

function auditDocumentation(file: string, sourceText: string): DocumentationAudit {
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

  return { declarations, optionalMembers, problems };
}

void test("every exported declaration and optional option carries complete documentation", () => {
  const problems: string[] = [];
  let declarations = 0;
  let optionalMembers = 0;

  for (const file of listSourceFiles()) {
    const audit = auditDocumentation(file, readFileSync(new URL(file, sourceDirectory), "utf8"));
    declarations += audit.declarations;
    optionalMembers += audit.optionalMembers;
    problems.push(...audit.problems);
  }

  // Guard the parser itself: if source discovery rots, the suite must fail
  // loudly instead of silently checking nothing.
  assert.ok(declarations >= 20, `expected >= 20 exported declarations, found ${declarations}`);
  assert.ok(optionalMembers >= 15, `expected >= 15 optional options, found ${optionalMembers}`);
  assert.deepEqual(problems, []);
});

void test("formatting cannot hide exported declarations or optional options", () => {
  const audit = auditDocumentation(
    "fixture.ts",
    [
      "/** Documented despite the multiline declaration. */",
      "export",
      "interface MultilineOptions {",
      "  hidden?: boolean;",
      "}",
      "",
      "/** Type-alias options are part of the same contract. */",
      "export type AliasOptions = {",
      "  /** @default false */",
      "  documented?: boolean;",
      "  missing?: string;",
      "};",
      "",
      "  export async function undocumented(): Promise<void> {}",
    ].join("\n"),
  );

  assert.equal(audit.declarations, 3);
  assert.equal(audit.optionalMembers, 3);
  assert.deepEqual(audit.problems, [
    'fixture.ts:4 optional option "hidden" must document @default',
    'fixture.ts:11 optional option "missing" must document @default',
    'fixture.ts:14 export "undocumented" has no JSDoc block directly above',
  ]);
});

void test("re-exports are exempt and an overload group is documented once", () => {
  const audit = auditDocumentation(
    "fixture.ts",
    [
      'export * from "./dependency.ts";',
      'export { dependency } from "./dependency.ts";',
      "/** Documented overload group. */",
      "export function parse(value: string): string;",
      "export function parse(value: number): string;",
      "export function parse(value: string | number): string { return String(value); }",
    ].join("\n"),
  );

  assert.equal(audit.declarations, 3);
  assert.equal(audit.optionalMembers, 0);
  assert.deepEqual(audit.problems, []);
});
