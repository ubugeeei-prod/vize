import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const root = fileURLToPath(new URL("../../", import.meta.url));

function parse(file: string) {
  return ts.createSourceFile(file, fs.readFileSync(file, "utf8"), ts.ScriptTarget.Latest, true);
}

function workerDependencies(file: string, visited = new Set<string>()): Set<string> {
  if (visited.has(file)) return new Set();
  visited.add(file);
  const dependencies = new Set<string>();
  for (const statement of parse(file).statements) {
    if (
      !ts.isImportDeclaration(statement) ||
      statement.importClause?.isTypeOnly ||
      !ts.isStringLiteral(statement.moduleSpecifier)
    )
      continue;
    const specifier = statement.moduleSpecifier.text;
    if (specifier.startsWith(".")) {
      for (const dependency of workerDependencies(
        path.resolve(path.dirname(file), `${specifier}.ts`),
        visited,
      )) {
        dependencies.add(dependency);
      }
    } else {
      dependencies.add(specifier);
    }
  }
  return dependencies;
}

function optimizerIncludes(file: string): Set<string> {
  const dependencies = new Set<string>();
  function visit(node: ts.Node) {
    if (ts.isPropertyAssignment(node) && node.name.getText() === "optimizeDeps") {
      assert.ok(ts.isObjectLiteralExpression(node.initializer));
      for (const property of node.initializer.properties) {
        if (!ts.isPropertyAssignment(property) || property.name.getText() !== "include") continue;
        assert.ok(ts.isArrayLiteralExpression(property.initializer));
        for (const dependency of property.initializer.elements) {
          assert.ok(ts.isStringLiteral(dependency));
          dependencies.add(dependency.text);
        }
      }
    }
    ts.forEachChild(node, visit);
  }
  visit(parse(file));
  return dependencies;
}

void test("browser and app optimizers prepare the real highlighter worker dependencies before rendering", () => {
  const dependencies = workerDependencies(
    path.join(root, "playground/src/shared/codeHighlighting.worker.ts"),
  );
  assert.deepEqual([...dependencies], ["shiki"]);
  for (const config of ["vite.app.config.ts", "vite.test.config.ts"]) {
    const included = optimizerIncludes(path.join(root, "playground", config));
    for (const dependency of dependencies) {
      assert.ok(
        included.has(dependency),
        `${config} must prebundle worker dependency ${dependency}`,
      );
    }
  }
});
