"use strict";

const path = require("node:path");

const vueExtension = ".vue";
const virtualDtsSuffix = ".d.ts";
const installedHosts = new WeakSet();

function init({ typescript: ts }) {
  return {
    create(info) {
      if (!info?.languageService) {
        return info?.languageService;
      }
      try {
        return installVueImportResolver(ts, info)
          ? createLanguageServiceProxy(ts, info.languageService)
          : info.languageService;
      } catch (error) {
        logPluginError(info, "create", error);
        return info.languageService;
      }
    },
  };
}

function installVueImportResolver(ts, info) {
  const host = info.languageServiceHost;
  if (!host) {
    return false;
  }
  if (installedHosts.has(host)) {
    return true;
  }

  const serverHost = info.serverHost || ts.sys;
  const originalFileExists = bind(host.fileExists, host) || bind(serverHost.fileExists, serverHost);
  const originalReadFile = bind(host.readFile, host) || bind(serverHost.readFile, serverHost);
  const originalGetScriptSnapshot = bind(host.getScriptSnapshot, host);
  const originalGetScriptKind = bind(host.getScriptKind, host);
  const originalResolveModuleNameLiterals = bind(host.resolveModuleNameLiterals, host);
  const originalResolveModuleNames = bind(host.resolveModuleNames, host);

  if (!originalFileExists || !originalReadFile || !originalGetScriptSnapshot) {
    return false;
  }

  const replacements = {
    fileExists(fileName) {
      const vuePath = realVuePathFromVirtual(fileName);
      if (vuePath) {
        return originalFileExists(vuePath);
      }
      return originalFileExists(fileName);
    },
    readFile(fileName) {
      const vuePath = realVuePathFromVirtual(fileName);
      if (vuePath && originalFileExists(vuePath)) {
        return virtualVueDts();
      }
      return originalReadFile(fileName);
    },
    getScriptSnapshot(fileName) {
      const vuePath = realVuePathFromVirtual(fileName);
      if (vuePath && originalFileExists(vuePath)) {
        return ts.ScriptSnapshot.fromString(virtualVueDts());
      }
      return originalGetScriptSnapshot(fileName);
    },
    getScriptKind(fileName) {
      if (realVuePathFromVirtual(fileName)) {
        return ts.ScriptKind.TS;
      }
      return originalGetScriptKind ? originalGetScriptKind(fileName) : ts.ScriptKind.Unknown;
    },
    resolveModuleNameLiterals(...args) {
      const [moduleLiterals, containingFile] = args;
      const previous = toArray(originalResolveModuleNameLiterals?.(...args));

      if (!Array.isArray(moduleLiterals)) {
        return previous;
      }

      return moduleLiterals.map((literal, index) => {
        if (previous[index] && previous[index].resolvedModule) {
          return previous[index];
        }

        const resolvedModule = resolveVueModule(
          ts,
          literal?.text,
          containingFile,
          originalFileExists,
        );
        return resolvedModule
          ? { resolvedModule }
          : previous[index] || { resolvedModule: undefined };
      });
    },
    resolveModuleNames(...args) {
      const [moduleNames, containingFile] = args;
      const previous = toArray(originalResolveModuleNames?.(...args));

      if (!Array.isArray(moduleNames)) {
        return previous;
      }

      return moduleNames.map((moduleName, index) => {
        if (previous[index]) {
          return previous[index];
        }
        return resolveVueModule(ts, moduleName, containingFile, originalFileExists);
      });
    },
  };

  const replacementEntries = Object.entries(replacements);
  if (!replacementEntries.every(([name]) => canAssignProperty(host, name))) {
    return false;
  }

  const originals = new Map(replacementEntries.map(([name]) => [name, host[name]]));
  try {
    for (const [name, replacement] of replacementEntries) {
      host[name] = replacement;
    }
  } catch (error) {
    for (const [name, original] of originals) {
      try {
        host[name] = original;
      } catch {
        // Best effort rollback only; create() will return the unproxied service.
      }
    }
    logPluginError(info, "install", error);
    return false;
  }

  installedHosts.add(host);
  return true;
}

function resolveVueModule(ts, specifier, containingFile, fileExists) {
  if (!isRelativeVueSpecifier(specifier) || typeof containingFile !== "string") {
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
      safeCall(() => languageService.getDefinitionAtPosition(fileName, position), undefined),
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

function canAssignProperty(target, key) {
  const descriptor = findPropertyDescriptor(target, key);
  if (!descriptor) {
    return isExtensible(target);
  }
  if ("writable" in descriptor) {
    return descriptor.writable;
  }
  return typeof descriptor.set === "function";
}

function findPropertyDescriptor(target, key) {
  let current = target;
  while (current) {
    const descriptor = Object.getOwnPropertyDescriptor(current, key);
    if (descriptor) {
      return descriptor;
    }
    current = Object.getPrototypeOf(current);
  }
  return undefined;
}

function isExtensible(target) {
  try {
    return Object.isExtensible(target);
  } catch {
    return false;
  }
}

function safeCall(fn, fallback) {
  try {
    return fn();
  } catch {
    return fallback;
  }
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

function toArray(value) {
  return Array.isArray(value) ? value : [];
}

function isRelativeVueSpecifier(specifier) {
  return (
    typeof specifier === "string" &&
    specifier.endsWith(vueExtension) &&
    (specifier.startsWith("./") || specifier.startsWith("../"))
  );
}

function realVuePathFromVirtual(fileName) {
  return typeof fileName === "string" && fileName.endsWith(`${vueExtension}${virtualDtsSuffix}`)
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
