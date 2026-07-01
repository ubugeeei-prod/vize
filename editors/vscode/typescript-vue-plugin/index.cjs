"use strict";

const path = require("node:path");

const vueExtension = ".vue";
const virtualDtsSuffix = ".d.ts";

function init({ typescript: ts }) {
  return {
    create(info) {
      installVueImportResolver(ts, info);
      return createLanguageServiceProxy(ts, info.languageService);
    },
  };
}

function installVueImportResolver(ts, info) {
  const host = info.languageServiceHost;
  if (!host || host.__vizeVueImportResolverInstalled) {
    return;
  }
  Object.defineProperty(host, "__vizeVueImportResolverInstalled", {
    value: true,
  });

  const serverHost = info.serverHost || ts.sys;
  const originalFileExists = bind(host.fileExists, host) || bind(serverHost.fileExists, serverHost);
  const originalReadFile = bind(host.readFile, host) || bind(serverHost.readFile, serverHost);
  const originalGetScriptSnapshot = bind(host.getScriptSnapshot, host);
  const originalGetScriptKind = bind(host.getScriptKind, host);
  const originalResolveModuleNameLiterals = bind(host.resolveModuleNameLiterals, host);
  const originalResolveModuleNames = bind(host.resolveModuleNames, host);

  host.fileExists = (fileName) => {
    const vuePath = realVuePathFromVirtual(fileName);
    if (vuePath) {
      return originalFileExists(vuePath);
    }
    return originalFileExists(fileName);
  };

  host.readFile = (fileName) => {
    const vuePath = realVuePathFromVirtual(fileName);
    if (vuePath && originalFileExists(vuePath)) {
      return virtualVueDts();
    }
    return originalReadFile(fileName);
  };

  host.getScriptSnapshot = (fileName) => {
    const vuePath = realVuePathFromVirtual(fileName);
    if (vuePath && originalFileExists(vuePath)) {
      return ts.ScriptSnapshot.fromString(virtualVueDts());
    }
    return originalGetScriptSnapshot(fileName);
  };

  host.getScriptKind = (fileName) => {
    if (realVuePathFromVirtual(fileName)) {
      return ts.ScriptKind.TS;
    }
    return originalGetScriptKind ? originalGetScriptKind(fileName) : ts.ScriptKind.Unknown;
  };

  host.resolveModuleNameLiterals = (...args) => {
    const [moduleLiterals, containingFile] = args;
    const previous = originalResolveModuleNameLiterals
      ? originalResolveModuleNameLiterals(...args)
      : [];

    return moduleLiterals.map((literal, index) => {
      if (previous[index] && previous[index].resolvedModule) {
        return previous[index];
      }

      const resolvedModule = resolveVueModule(ts, literal.text, containingFile, originalFileExists);
      return resolvedModule ? { resolvedModule } : previous[index] || { resolvedModule: undefined };
    });
  };

  host.resolveModuleNames = (...args) => {
    const [moduleNames, containingFile] = args;
    const previous = originalResolveModuleNames ? originalResolveModuleNames(...args) : [];

    return moduleNames.map((moduleName, index) => {
      if (previous[index]) {
        return previous[index];
      }
      return resolveVueModule(ts, moduleName, containingFile, originalFileExists);
    });
  };
}

function resolveVueModule(ts, specifier, containingFile, fileExists) {
  if (!isRelativeVueSpecifier(specifier)) {
    return undefined;
  }

  const vuePath = path.resolve(path.dirname(containingFile), specifier);
  if (!fileExists(vuePath)) {
    return undefined;
  }

  return {
    extension: ts.Extension.Dts,
    isExternalLibraryImport: false,
    resolvedFileName: virtualVueDtsPath(vuePath),
  };
}

function createLanguageServiceProxy(ts, languageService) {
  const proxy = Object.create(null);
  for (const key of Object.keys(languageService)) {
    const value = languageService[key];
    proxy[key] = typeof value === "function" ? value.bind(languageService) : value;
  }

  proxy.getDefinitionAtPosition = (fileName, position) =>
    remapVueVirtualDefinitions(ts, languageService.getDefinitionAtPosition(fileName, position));

  proxy.getDefinitionAndBoundSpan = (fileName, position) => {
    const result = languageService.getDefinitionAndBoundSpan(fileName, position);
    if (!result || !result.definitions) {
      return result;
    }
    return {
      ...result,
      definitions: remapVueVirtualDefinitions(ts, result.definitions),
    };
  };

  proxy.getTypeDefinitionAtPosition = (fileName, position) =>
    remapVueVirtualDefinitions(ts, languageService.getTypeDefinitionAtPosition(fileName, position));

  proxy.getQuickInfoAtPosition = (fileName, position) => {
    const quickInfo = languageService.getQuickInfoAtPosition(fileName, position);
    if (!quickInfo) {
      return quickInfo;
    }

    const vuePaths = vuePathsFromDefinitions(
      languageService.getDefinitionAtPosition(fileName, position),
    );
    if (vuePaths.length === 0) {
      return quickInfo;
    }

    return {
      ...quickInfo,
      documentation: [
        ...(quickInfo.documentation || []),
        {
          kind: "text",
          text: `Vue component: ${path.basename(vuePaths[0])}`,
        },
      ],
    };
  };

  return proxy;
}

function remapVueVirtualDefinitions(ts, definitions) {
  return definitions?.map((definition) => remapVueVirtualDefinition(ts, definition));
}

function vuePathsFromDefinitions(definitions) {
  return (
    definitions
      ?.map((definition) => realVuePathFromVirtual(definition.fileName))
      .filter((vuePath) => vuePath) || []
  );
}

function remapVueVirtualDefinition(ts, definition) {
  const vuePath = realVuePathFromVirtual(definition.fileName);
  if (!vuePath) {
    return definition;
  }

  return {
    ...definition,
    fileName: vuePath,
    textSpan: { start: 0, length: 0 },
    contextSpan: undefined,
    kind: ts.ScriptElementKind.scriptElement,
    name: path.basename(vuePath),
  };
}

function bind(fn, thisArg) {
  return typeof fn === "function" ? fn.bind(thisArg) : undefined;
}

function isRelativeVueSpecifier(specifier) {
  return (
    specifier.endsWith(vueExtension) && (specifier.startsWith("./") || specifier.startsWith("../"))
  );
}

function realVuePathFromVirtual(fileName) {
  return fileName.endsWith(`${vueExtension}${virtualDtsSuffix}`)
    ? fileName.slice(0, -virtualDtsSuffix.length)
    : undefined;
}

function virtualVueDtsPath(vuePath) {
  return `${vuePath}${virtualDtsSuffix}`;
}

function virtualVueDts() {
  return "declare const component: any;\nexport default component;\n";
}

module.exports = init;
