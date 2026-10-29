"use strict";

const path = require("node:path");
const { isRelativeSpecifier } = require("./module-resolution.cjs");
const { authoredSpan, generatedPositions } = require("./language-service.cjs");
const { isSfc } = require("./virtual-modules.cjs");

// TypeScript owns both the current source tree and lexical symbol identity.
// A same-spelled parameter/property is never an imported component binding.
function resolveVueImportDefinition(ts, support, fileName, position, service) {
  const program = service.getProgram?.();
  const sourceFile = program?.getSourceFile(fileName);
  if (!sourceFile) return undefined;
  const checker = program.getTypeChecker();
  const feature = support.virtual?.engine?.LanguageFeature?.Definition;
  for (const offset of generatedPositions(support.virtual, fileName, position, feature)) {
    const token = tokenAt(sourceFile, offset);
    if (!token) continue;
    const symbol = ts.isIdentifier(token)
      ? token.parent && ts.isShorthandPropertyAssignment(token.parent)
        ? checker.getShorthandAssignmentValueSymbol(token.parent)
        : checker.getSymbolAtLocation(token)
      : undefined;
    for (const statement of sourceFile.statements) {
      if (!ts.isImportDeclaration(statement) || !ts.isStringLiteral(statement.moduleSpecifier))
        continue;
      const specifier = statement.moduleSpecifier;
      const bindings = importBindings(ts, statement);
      const selected =
        symbol && bindings.find((binding) => checker.getSymbolAtLocation(binding.node) === symbol);
      const onSpecifier = token === specifier;
      if (!selected && !onSpecifier) continue;
      const binding = selected || bindings[0];
      const vuePath = componentModule(
        ts,
        support,
        program,
        checker,
        sourceFile.fileName,
        specifier.text,
        binding?.importedName,
        new Set(),
      );
      if (!vuePath) continue;
      const generated = onSpecifier
        ? { start: specifier.getStart(sourceFile) + 1, length: specifier.getWidth(sourceFile) - 2 }
        : { start: token.getStart(sourceFile), length: token.getWidth(sourceFile) };
      const textSpan = authoredSpan(support.virtual, fileName, generated, feature);
      if (!textSpan) continue;
      return {
        definition: {
          fileName: vuePath,
          kind: ts.ScriptElementKind.scriptElement,
          name: path.basename(vuePath),
          textSpan: { start: 0, length: 0 },
        },
        localName: selected?.node.text || binding?.node.text,
        textSpan,
      };
    }
  }
  return undefined;
}

function importBindings(ts, statement) {
  const clause = statement.importClause;
  if (!clause) return [];
  const bindings = clause.name ? [{ node: clause.name, importedName: "default" }] : [];
  if (clause.namedBindings) {
    if (ts.isNamespaceImport(clause.namedBindings))
      bindings.push({ node: clause.namedBindings.name });
    else
      for (const element of clause.namedBindings.elements)
        bindings.push({
          node: element.name,
          importedName: element.propertyName?.text || element.name.text,
        });
  }
  return bindings;
}

function componentModule(
  ts,
  support,
  program,
  checker,
  containingFile,
  specifier,
  importedName,
  seen,
) {
  const resolved = support.virtual?.resolve(specifier, containingFile);
  const fileName = resolved?.originalPath || resolved?.resolvedFileName;
  // Respect TypeScript's actual declaration/module choice, including .d.vue.ts.
  if (isSfc(fileName))
    return importedName === undefined || importedName === "default" ? fileName : undefined;
  if (!fileName || seen.has(fileName) || !importedName) return undefined;
  seen.add(fileName);
  const sourceFile =
    program.getSourceFile(fileName) || program.getSourceFile(resolved.resolvedFileName);
  const moduleSymbol = sourceFile && checker.getSymbolAtLocation(sourceFile);
  const exported =
    moduleSymbol &&
    checker.getExportsOfModule(moduleSymbol).find((symbol) => symbol.name === importedName);
  if (!exported) return undefined;
  for (const declaration of exported.declarations || []) {
    if (!ts.isExportSpecifier(declaration)) continue;
    const statement = declaration.parent.parent;
    if (
      !ts.isExportDeclaration(statement) ||
      !statement.moduleSpecifier ||
      !ts.isStringLiteral(statement.moduleSpecifier)
    )
      continue;
    const target = componentModule(
      ts,
      support,
      program,
      checker,
      fileName,
      statement.moduleSpecifier.text,
      declaration.propertyName?.text || declaration.name.text,
      seen,
    );
    if (target) return target;
  }
  return undefined;
}

function tokenAt(sourceFile, position) {
  let current = sourceFile;
  for (;;) {
    let child;
    current.forEachChild((node) => {
      if (position >= node.getStart(sourceFile) && position <= node.getEnd()) {
        child = node;
        return true;
      }
    });
    if (!child) return current;
    current = child;
  }
}

// Kept for the existing explicit-path utility contract. Navigation above uses
// the host's real TypeScript resolver and current Program instead.
function resolveExistingVueSpecifier(containingFile, specifier, fileExists) {
  if (
    typeof containingFile !== "string" ||
    typeof specifier !== "string" ||
    !specifier.endsWith(".vue") ||
    !isRelativeSpecifier(specifier)
  )
    return undefined;
  const vuePath = path.resolve(path.dirname(containingFile), specifier);
  return fileExists(vuePath) ? vuePath : undefined;
}
module.exports = { resolveExistingVueSpecifier, resolveVueImportDefinition };
