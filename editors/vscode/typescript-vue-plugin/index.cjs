"use strict";

const path = require("node:path");

const vueExtension = ".vue";
const virtualDtsSuffix = ".d.ts";

function init({ typescript: ts }) {
  return {
    create(info) {
      installVueImportResolver(ts, info);
      return info.languageService;
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
