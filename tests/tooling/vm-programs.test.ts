import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const entrypoints = new Set([
  "Script",
  "compileFunction",
  "runInContext",
  "runInNewContext",
  "runInThisContext",
]);

function literalProgram(node: ts.Expression): boolean {
  if (ts.isParenthesizedExpression(node)) return literalProgram(node.expression);
  if (ts.isStringLiteralLike(node)) return true;
  return (
    ts.isBinaryExpression(node) &&
    node.operatorToken.kind === ts.SyntaxKind.PlusToken &&
    literalProgram(node.left) &&
    literalProgram(node.right)
  );
}

function vmEntrypoints(source: ts.SourceFile) {
  const namespaces = new Set<string>();
  const names = new Set<string>();
  for (const statement of source.statements) {
    if (
      ts.isImportDeclaration(statement) &&
      ts.isStringLiteral(statement.moduleSpecifier) &&
      statement.moduleSpecifier.text === "node:vm"
    ) {
      const clause = statement.importClause;
      if (clause?.name) namespaces.add(clause.name.text);
      const bindings = clause?.namedBindings;
      if (!bindings) continue;
      if (ts.isNamespaceImport(bindings)) {
        namespaces.add(bindings.name.text);
      } else {
        for (const entry of bindings.elements) {
          if (entrypoints.has((entry.propertyName ?? entry.name).text)) names.add(entry.name.text);
        }
      }
    }
    if (!ts.isVariableStatement(statement)) continue;
    for (const declaration of statement.declarationList.declarations) {
      const load = declaration.initializer;
      if (
        !load ||
        !ts.isCallExpression(load) ||
        !ts.isIdentifier(load.expression) ||
        load.expression.text !== "require" ||
        !load.arguments[0] ||
        !ts.isStringLiteral(load.arguments[0]) ||
        load.arguments[0].text !== "node:vm"
      )
        continue;
      if (ts.isIdentifier(declaration.name)) namespaces.add(declaration.name.text);
      if (!ts.isObjectBindingPattern(declaration.name)) continue;
      for (const entry of declaration.name.elements) {
        const original = entry.propertyName ?? entry.name;
        if (
          ts.isIdentifier(original) &&
          ts.isIdentifier(entry.name) &&
          entrypoints.has(original.text)
        ) {
          names.add(entry.name.text);
        }
      }
    }
  }
  return (method: ts.Expression): boolean =>
    ts.isIdentifier(method)
      ? names.has(method.text)
      : ts.isPropertyAccessExpression(method) &&
        ts.isIdentifier(method.expression) &&
        namespaces.has(method.expression.text) &&
        entrypoints.has(method.name.text);
}

function inlineVmPrograms(file: string, text: string): number[] {
  const source = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true);
  const isVmEntrypoint = vmEntrypoints(source);
  const lines: number[] = [];
  function visit(node: ts.Node): void {
    if (ts.isCallExpression(node) || ts.isNewExpression(node)) {
      const argument = node.arguments?.[0];
      if (argument && isVmEntrypoint(node.expression) && literalProgram(argument)) {
        lines.push(source.getLineAndCharacterOfPosition(argument.getStart(source)).line + 1);
      }
    }
    ts.forEachChild(node, visit);
  }
  visit(source);
  return lines;
}

test("VM drivers use named programs and accept source supplied by the compiler", () => {
  const source = [
    'import vm, { Script as Program, compileFunction } from "node:vm";',
    'new Program("install()");',
    'vm.runInNewContext(("prepare();" + "observe()"), scope);',
    "compileFunction(`return 42;`);",
    'vm.runInContext(readFileSync(fixture, "utf8"), context);',
    "new Program(compiled.code);",
    'browser.runInContext("case");',
    'const { Script: CommonJsProgram } = require("node:vm");',
    'new CommonJsProgram("install()");',
    'const runtime = require("node:vm");',
    'runtime.runInThisContext("observe()");',
  ].join("\n");
  assert.deepEqual(inlineVmPrograms("controls.ts", source), [2, 3, 4, 9, 11]);
});

test("repository VM entrypoints load their authored programs from files", () => {
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const files = execFileSync(
    "git",
    [
      "grep",
      "-Ilz",
      "node:vm",
      "--",
      "*.ts",
      "*.tsx",
      "*.mts",
      "*.cts",
      "*.js",
      "*.jsx",
      "*.cjs",
      "*.mjs",
      ":(exclude)upstream/**",
      ":(exclude)**/upstream/**",
    ],
    { cwd: root, encoding: "utf8" },
  )
    .split("\0")
    .filter(Boolean);
  assert.ok(files.includes("tests/tooling/docs-capture-packet-proof.test.ts"));
  const inline = files.flatMap((file) =>
    inlineVmPrograms(file, readFileSync(join(root, file), "utf8")).map((line) => `${file}:${line}`),
  );
  assert.deepEqual(inline, [], "move an authored VM program to a named file");
});
