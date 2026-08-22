"use strict";

const path = require("node:path");
const { vueComponentDisplayParts } = require("./component-contracts.cjs");
const { installVueVirtualModules } = require("./virtual-modules.cjs");

const vueExtension = ".vue";

function init({ typescript: ts }) {
  return {
    create(info) {
      if (!info?.languageService) {
        return info?.languageService;
      }
      try {
        const support = createVueImportSupport(ts, info);
        return support
          ? createLanguageServiceProxy(ts, info.languageService, support)
          : info.languageService;
      } catch (error) {
        logPluginError(info, "create", error);
        return info.languageService;
      }
    },
  };
}

function createVueImportSupport(ts, info) {
  const host = info.languageServiceHost;
  if (!host) {
    return undefined;
  }

  const serverHost = info.serverHost || ts.sys;
  const originalFileExists = bind(host.fileExists, host) || bind(serverHost.fileExists, serverHost);
  const originalReadFile = bind(host.readFile, host) || bind(serverHost.readFile, serverHost);

  if (!originalFileExists || !originalReadFile) {
    return undefined;
  }

  installVueVirtualModules(ts, info);

  return {
    fileExists: originalFileExists,
    readFile: originalReadFile,
  };
}

function createLanguageServiceProxy(ts, languageService, support) {
  const proxy = Object.create(null);
  for (const key of Object.keys(languageService)) {
    const value = languageService[key];
    proxy[key] = typeof value === "function" ? value.bind(languageService) : value;
  }

  proxy.getSemanticDiagnostics = (fileName) =>
    filterVueImportDiagnostics(ts, languageService.getSemanticDiagnostics(fileName), {
      fileExists: support.fileExists,
      fileName,
    });

  proxy.getDefinitionAtPosition = (fileName, position) => {
    const vueDefinition = resolveVueImportDefinition(ts, support, fileName, position);
    return vueDefinition
      ? [vueDefinition.definition]
      : languageService.getDefinitionAtPosition(fileName, position);
  };

  proxy.getDefinitionAndBoundSpan = (fileName, position) => {
    const vueDefinition = resolveVueImportDefinition(ts, support, fileName, position);
    if (vueDefinition) {
      return {
        definitions: [vueDefinition.definition],
        textSpan: vueDefinition.textSpan,
      };
    }

    const result = languageService.getDefinitionAndBoundSpan(fileName, position);
    return result;
  };

  proxy.getTypeDefinitionAtPosition = (fileName, position) => {
    const vueDefinition = resolveVueImportDefinition(ts, support, fileName, position);
    return vueDefinition
      ? [vueDefinition.definition]
      : languageService.getTypeDefinitionAtPosition(fileName, position);
  };

  proxy.getQuickInfoAtPosition = (fileName, position) => {
    const vueDefinition = resolveVueImportDefinition(ts, support, fileName, position);
    const quickInfo = languageService.getQuickInfoAtPosition(fileName, position);
    if (!vueDefinition) {
      return quickInfo;
    }
    const sourceText = support.readFile(vueDefinition.definition.fileName);
    const displayParts = vueComponentDisplayParts(ts, sourceText, vueDefinition.localName);

    return {
      kind: ts.ScriptElementKind.alias,
      kindModifiers: "",
      textSpan: vueDefinition.textSpan,
      ...(quickInfo || {}),
      ...(displayParts ? { displayParts } : {}),
      documentation: [
        ...(quickInfo?.documentation || []),
        {
          kind: "text",
          text: `Vue component: ${path.basename(vueDefinition.definition.fileName)}`,
        },
      ],
    };
  };

  return proxy;
}

function logPluginError(info, phase, error) {
  try {
    const message = error instanceof Error ? error.message : String(error);
    info?.project?.projectService?.logger?.info?.(
      `[vize] TypeScript Vue plugin ${phase} failed: ${message}`,
    );
  } catch {
    // Logging must never be the reason the TypeScript server exits.
  }
}

function filterVueImportDiagnostics(ts, diagnostics, { fileExists, fileName }) {
  return diagnostics.filter((diagnostic) => {
    if (diagnostic.code !== 2307) {
      return true;
    }

    const specifier = diagnosticVueSpecifier(ts, diagnostic);
    const containingFile = diagnostic.file?.fileName || fileName;
    return !resolveExistingVueSpecifier(containingFile, specifier, fileExists);
  });
}

function diagnosticVueSpecifier(ts, diagnostic) {
  const message = ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n");
  return message.match(/Cannot find module ['"]([^'"]+\.vue)['"]/)?.[1];
}

function resolveVueImportDefinition(ts, support, fileName, position) {
  if (typeof fileName !== "string") {
    return undefined;
  }

  const sourceText = support.readFile(fileName);
  if (typeof sourceText !== "string") {
    return undefined;
  }

  const vueImport = findVueImportAtPosition(ts, sourceText, position);
  const vuePath = resolveExistingVueSpecifier(fileName, vueImport?.specifier, support.fileExists);
  if (!vueImport || !vuePath) {
    return undefined;
  }

  return {
    definition: {
      fileName: vuePath,
      kind: ts.ScriptElementKind.scriptElement,
      name: path.basename(vuePath),
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

function bind(fn, thisArg) {
  return typeof fn === "function" ? fn.bind(thisArg) : undefined;
}

module.exports = init;
