"use strict";

const path = require("node:path");
const { vueComponentDisplayParts } = require("./component-contracts.cjs");
const {
  resolveExistingVueSpecifier,
  resolveVueImportDefinition,
} = require("./import-resolution.cjs");
const { installVueVirtualModules } = require("./virtual-modules.cjs");

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

function bind(fn, thisArg) {
  return typeof fn === "function" ? fn.bind(thisArg) : undefined;
}

module.exports = init;
