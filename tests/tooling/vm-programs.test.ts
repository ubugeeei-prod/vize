import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";
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

const vmModule = (name: ts.Node | undefined) =>
  name !== undefined &&
  ts.isStringLiteralLike(name) &&
  (name.text === "node:vm" || name.text === "vm");

function literalProgram(
  node: ts.Expression,
  checker: ts.TypeChecker,
  seen = new Set<ts.Declaration>(),
): boolean {
  if (ts.isStringLiteralLike(node)) return true;
  if (ts.isParenthesizedExpression(node)) return literalProgram(node.expression, checker, seen);
  if (ts.isBinaryExpression(node) && node.operatorToken.kind === ts.SyntaxKind.PlusToken) {
    return literalProgram(node.left, checker, seen) && literalProgram(node.right, checker, seen);
  }
  if (!ts.isIdentifier(node)) return false;
  const declaration = checker.getSymbolAtLocation(node)?.valueDeclaration;
  if (
    !declaration ||
    seen.has(declaration) ||
    !ts.isVariableDeclaration(declaration) ||
    !(declaration.parent.flags & ts.NodeFlags.Const) ||
    !declaration.initializer
  )
    return false;
  return literalProgram(declaration.initializer, checker, new Set(seen).add(declaration));
}

function loadsVm(node: ts.Expression | undefined, checker: ts.TypeChecker): boolean {
  if (
    !node ||
    !ts.isCallExpression(node) ||
    !ts.isIdentifier(node.expression) ||
    node.expression.text !== "require" ||
    !vmModule(node.arguments[0])
  )
    return false;
  return !checker.getSymbolAtLocation(node.expression)?.declarations?.length;
}

function vmBinding(node: ts.Expression, checker: ts.TypeChecker): string | undefined {
  if (loadsVm(node, checker)) return "*";
  if (!ts.isIdentifier(node)) return;
  const symbol = checker.getSymbolAtLocation(node);
  for (const declaration of symbol?.declarations ?? []) {
    const imported = ts.findAncestor(declaration, ts.isImportDeclaration);
    if (imported && vmModule(imported.moduleSpecifier)) {
      if (ts.isImportClause(declaration) || ts.isNamespaceImport(declaration)) return "*";
      if (ts.isImportSpecifier(declaration)) {
        const name = (declaration.propertyName ?? declaration.name).text;
        return name === "default" ? "*" : name;
      }
    }
    if (ts.isVariableDeclaration(declaration) && loadsVm(declaration.initializer, checker))
      return "*";
    if (ts.isBindingElement(declaration) && ts.isObjectBindingPattern(declaration.parent)) {
      const owner = declaration.parent.parent;
      const name = declaration.propertyName ?? declaration.name;
      if (
        ts.isVariableDeclaration(owner) &&
        loadsVm(owner.initializer, checker) &&
        (ts.isIdentifier(name) || ts.isStringLiteralLike(name))
      )
        return name.text;
    }
  }
}

function inlineVmPrograms(file: string, text: string): number[] {
  const filename = resolve(file);
  const source = ts.createSourceFile(filename, text, ts.ScriptTarget.Latest, true);
  const options: ts.CompilerOptions = { allowJs: true, noResolve: true, noLib: true, types: [] };
  const host = ts.createCompilerHost(options);
  host.getSourceFile = (name) => (name === filename ? source : undefined);
  const checker = ts.createProgram([filename], options, host).getTypeChecker();
  const lines: number[] = [];
  function visit(node: ts.Node): void {
    if (ts.isCallExpression(node) || ts.isNewExpression(node)) {
      const argument = node.arguments?.[0];
      const target = node.expression;
      const method =
        ts.isPropertyAccessExpression(target) && vmBinding(target.expression, checker) === "*"
          ? target.name.text
          : vmBinding(target, checker);
      if (argument && method && entrypoints.has(method) && literalProgram(argument, checker)) {
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

test("VM imports resolve to their declaration through local scopes", () => {
  const source = [
    'import vm, { runInContext } from "node:vm";',
    'function local() { const { runInContext: run } = require("vm"); run("install()", context); }',
    'function shadow(vm, runInContext) { vm.runInContext("case"); runInContext("case"); }',
    'function localName() { const vm = browser; vm.runInContext("case"); }',
    'function alias(require) { const vm = require("node:vm"); vm.runInContext("case"); }',
    'function actual() { const body = "install()"; vm.runInContext(body, context); }',
    'runInContext("observe()", context);',
    'require("node:vm").runInThisContext("observe()");',
  ].join("\n");
  assert.deepEqual(inlineVmPrograms("nested.ts", source), [2, 6, 7, 8]);
});

test("repository VM entrypoints load their authored programs from files", () => {
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const files = execFileSync(
    "git",
    [
      "grep",
      "-IlzE",
      `['"](node:)?vm['"]`,
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
