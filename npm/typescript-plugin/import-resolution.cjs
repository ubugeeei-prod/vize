"use strict";

const path = require("node:path");
const { isRelativeSpecifier, resolveExistingModuleSpecifier } = require("./module-resolution.cjs");

const vueExtension = ".vue";

function resolveVueImportDefinition(ts, support, fileName, position) {
  if (typeof fileName !== "string") {
    return undefined;
  }

  const sourceText = support.readFile(fileName);
  if (typeof sourceText !== "string") {
    return undefined;
  }

  const directImport = findVueImportAtPosition(ts, sourceText, position);
  const directVuePath = directImport
    ? resolveExistingVueSpecifier(fileName, directImport.specifier, support.fileExists)
    : undefined;
  const vueImport = directVuePath
    ? { ...directImport, vuePath: directVuePath }
    : findVueReExportImportAtPosition(ts, support, fileName, sourceText, position);
  if (!vueImport?.vuePath) {
    return undefined;
  }

  return {
    definition: {
      fileName: vueImport.vuePath,
      kind: ts.ScriptElementKind.scriptElement,
      name: path.basename(vueImport.vuePath),
      textSpan: { start: 0, length: 0 },
    },
    localName: vueImport.localName,
    textSpan: vueImport.textSpan,
  };
}

function findVueImportAtPosition(ts, sourceText, position) {
  const sourceFile = ts.createSourceFile(
    "vize-vue-import.ts",
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  const imports = [];

  visit(sourceFile, (node) => {
    if (!ts.isImportDeclaration(node) || !isStringLiteralLike(ts, node.moduleSpecifier)) {
      return;
    }

    const specifier = node.moduleSpecifier.text;
    if (!isRelativeVueSpecifier(specifier)) {
      return;
    }

    const localNames = new Set();
    const importClause = node.importClause;
    if (importClause?.name) {
      localNames.add(importClause.name.text);
    }
    const namedBindings = importClause?.namedBindings;
    if (namedBindings && ts.isNamespaceImport(namedBindings)) {
      localNames.add(namedBindings.name.text);
    } else if (namedBindings && ts.isNamedImports(namedBindings)) {
      for (const element of namedBindings.elements) {
        localNames.add(element.name.text);
      }
    }

    imports.push({
      localNames,
      specifier,
      specifierSpan: {
        length: specifier.length,
        start: node.moduleSpecifier.getStart(sourceFile) + 1,
      },
    });
  });

  for (const vueImport of imports) {
    if (containsPosition(vueImport.specifierSpan, position)) {
      return {
        localName: [...vueImport.localNames][0],
        specifier: vueImport.specifier,
        textSpan: vueImport.specifierSpan,
      };
    }
  }

  const identifier = identifierAtPosition(ts, sourceFile, position);
  if (!identifier) {
    return undefined;
  }

  const identifierText = identifier.text;
  const vueImport = imports.find((entry) => entry.localNames.has(identifierText));
  if (!vueImport) {
    return undefined;
  }

  return {
    localName: identifierText,
    specifier: vueImport.specifier,
    textSpan: {
      length: identifier.getEnd() - identifier.getStart(sourceFile),
      start: identifier.getStart(sourceFile),
    },
  };
}

function findVueReExportImportAtPosition(ts, support, fileName, sourceText, position) {
  const importBinding = findNamedImportBindingAtPosition(
    ts,
    sourceText,
    position,
    (specifier) => !isRelativeVueSpecifier(specifier),
  );
  if (!importBinding) {
    return undefined;
  }
  const vuePath = resolveVueReExport(
    ts,
    support,
    fileName,
    importBinding.specifier,
    importBinding.importedName,
  );
  return vuePath ? { ...importBinding, vuePath } : undefined;
}

function findNamedImportBindingAtPosition(ts, sourceText, position, acceptsSpecifier) {
  const sourceFile = ts.createSourceFile(
    "vize-vue-import.ts",
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  const imports = [];

  visit(sourceFile, (node) => {
    if (!ts.isImportDeclaration(node) || !isStringLiteralLike(ts, node.moduleSpecifier)) {
      return;
    }
    const specifier = node.moduleSpecifier.text;
    if (!acceptsSpecifier(specifier)) {
      return;
    }
    const importClause = node.importClause;
    const namedBindings = importClause?.namedBindings;
    if (namedBindings && ts.isNamedImports(namedBindings)) {
      for (const element of namedBindings.elements) {
        const localName = element.name.text;
        imports.push({
          importedName: element.propertyName?.text || localName,
          localName,
          specifier,
          textSpan: {
            length: element.name.getEnd() - element.name.getStart(sourceFile),
            start: element.name.getStart(sourceFile),
          },
        });
      }
    }
  });

  const direct = imports.find((entry) => containsPosition(entry.textSpan, position));
  if (direct) {
    return direct;
  }
  const identifier = identifierAtPosition(ts, sourceFile, position);
  return identifier ? imports.find((entry) => entry.localName === identifier.text) : undefined;
}

function resolveVueReExport(
  ts,
  support,
  containingFile,
  specifier,
  exportedName,
  seen = new Set(),
) {
  const barrelPath = resolveExistingModuleSpecifier(
    containingFile,
    specifier,
    support.fileExists,
    support.readFile,
  );
  if (!barrelPath || seen.has(barrelPath)) {
    return undefined;
  }
  seen.add(barrelPath);
  const sourceText = support.readFile(barrelPath);
  if (typeof sourceText !== "string") {
    return undefined;
  }
  const sourceFile = ts.createSourceFile(
    barrelPath,
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  let result;
  visit(sourceFile, (node) => {
    if (result || !ts.isExportDeclaration(node) || !isStringLiteralLike(ts, node.moduleSpecifier)) {
      return;
    }
    const exportClause = node.exportClause;
    if (!exportClause || !ts.isNamedExports(exportClause)) {
      return;
    }
    for (const element of exportClause.elements) {
      if (element.name.text !== exportedName) {
        continue;
      }
      const targetName = element.propertyName?.text || element.name.text;
      const targetSpecifier = node.moduleSpecifier.text;
      result = isRelativeVueSpecifier(targetSpecifier)
        ? resolveExistingVueSpecifier(barrelPath, targetSpecifier, support.fileExists)
        : resolveVueReExport(ts, support, barrelPath, targetSpecifier, targetName, seen);
      return;
    }
  });
  return result;
}

function identifierAtPosition(ts, sourceFile, position) {
  let result;
  visit(sourceFile, (node) => {
    if (
      result ||
      !ts.isIdentifier(node) ||
      position < node.getStart(sourceFile) ||
      position > node.getEnd()
    ) {
      return;
    }
    result = node;
  });
  return result;
}

function visit(node, callback) {
  callback(node);
  node.forEachChild((child) => visit(child, callback));
}

function isStringLiteralLike(ts, node) {
  return ts.isStringLiteral(node) || node.kind === ts.SyntaxKind.NoSubstitutionTemplateLiteral;
}

function containsPosition(span, position) {
  return position >= span.start && position <= span.start + span.length;
}

function resolveExistingVueSpecifier(containingFile, specifier, fileExists) {
  if (!isRelativeVueSpecifier(specifier) || typeof containingFile !== "string") {
    return undefined;
  }

  const vuePath = path.resolve(path.dirname(containingFile), specifier);
  return fileExists(vuePath) ? vuePath : undefined;
}

function isRelativeVueSpecifier(specifier) {
  return (
    typeof specifier === "string" &&
    specifier.endsWith(vueExtension) &&
    (specifier.startsWith("./") || specifier.startsWith("../"))
  );
}

module.exports = {
  resolveExistingVueSpecifier,
  resolveVueImportDefinition,
};
