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

void test("the plain code worker has no browser highlighter dependency", () => {
  const dependencies = workerDependencies(
    path.join(root, "playground/src/shared/codeHighlighting.worker.ts"),
  );
  assert.deepEqual([...dependencies], []);
});
