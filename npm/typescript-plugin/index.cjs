"use strict";

const path = require("node:path");
const { vueComponentDisplayParts } = require("./component-contracts.cjs");
const { resolveVueImportDefinition } = require("./import-resolution.cjs");
const { createMappedLanguageService } = require("./language-service.cjs");
const { installVueVirtualModules, isSfc } = require("./virtual-modules.cjs");
const liveProjects = new WeakMap();
const externalFiles = new WeakMap();
const inactiveProjects = new WeakSet();

function init({ typescript: ts }) {
  return {
    getExternalFiles(project, updateLevel = 0) {
      if (inactiveProjects.has(project)) return [];
      const configFile = project.getCompilerOptions?.().configFilePath;
      if (typeof configFile !== "string") return [];
      // ProgramUpdateLevel.Update keeps the root names and options unchanged.
      const cached = externalFiles.get(project);
      if (updateLevel === 0 && cached?.configFile === configFile) return cached.files;
      const files = discoverSfcFiles(ts, configFile);
      externalFiles.set(project, { configFile, files });
      return files;
    },
    create(info) {
      if (info?.config?.enabled === false || !info?.languageService) {
        return info?.languageService;
      }
      // TypeScript re-enables configured plugins on the current service after
      // reloadProjects. That service can already contain this mapper, followed
      // by another plugin's wrapper. Preserve that complete live chain.
      if (liveProjects.has(info.languageServiceHost)) return info.languageService;
      try {
        const support = createVueImportSupport(ts, info);
        // Without an installed provider (the universal VSIX), leave tsserver untouched.
        if (support?.virtual?.engine?.unavailable) {
          if (info.project) inactiveProjects.add(info.project);
          logPluginError(info, "native provider", support.virtual.engine.error);
          return info.languageService;
        }
        const service = support
          ? createLanguageServiceProxy(
              ts,
              createMappedLanguageService(ts, info.languageService, support.virtual),
              support,
            )
          : info.languageService;
        if (support) {
          const dispose = service.dispose?.bind(service);
          service.dispose = () => {
            liveProjects.delete(info.languageServiceHost);
            dispose?.();
          };
          liveProjects.set(info.languageServiceHost, service);
        }
        return service;
      } catch (error) {
        logPluginError(info, "create", error);
        return info.languageService;
      }
    },
  };
}

function discoverSfcFiles(ts, configFile) {
  const config = ts.readConfigFile(configFile, ts.sys.readFile);
  if (config.error) return [];
  const parsed = ts.parseJsonConfigFileContent(
    config.config,
    ts.sys,
    path.dirname(configFile),
    undefined,
    configFile,
    undefined,
    [".vue", ".art"].map((extension) => ({
      extension,
      isMixedContent: true,
      scriptKind: ts.ScriptKind.Deferred,
    })),
  );
  // Parsed options include plugin entries inherited through `extends`.
  const plugin = parsed.options.plugins?.find(
    (entry) => entry.name === "@vizejs/typescript-vue-plugin",
  );
  if (plugin?.enabled === false) return [];
  return parsed.fileNames.filter((file) => /\.(?:vue|art)$/i.test(file));
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

  const originalSnapshot = bind(host.getScriptSnapshot, host);
  const virtual = installVueVirtualModules(ts, info);
  const readFile = (fileName) => {
    if (isSfc(fileName)) return virtual?.readSource(fileName) ?? originalReadFile(fileName);
    const snapshot = originalSnapshot?.(fileName);
    return snapshot?.getText(0, snapshot.getLength()) ?? originalReadFile(fileName);
  };

  return {
    virtual,
    fileExists: originalFileExists,
    readFile,
  };
}

function createLanguageServiceProxy(ts, languageService, support) {
  const proxy = Object.create(null);
  for (const key of Object.keys(languageService)) {
    const value = languageService[key];
    proxy[key] = typeof value === "function" ? value.bind(languageService) : value;
  }

  proxy.getDefinitionAtPosition = (fileName, position) => {
    const vueDefinition = resolveVueImportDefinition(
      ts,
      support,
      fileName,
      position,
      languageService,
    );
    return vueDefinition
      ? [vueDefinition.definition]
      : languageService.getDefinitionAtPosition(fileName, position);
  };

  proxy.getDefinitionAndBoundSpan = (fileName, position) => {
    const vueDefinition = resolveVueImportDefinition(
      ts,
      support,
      fileName,
      position,
      languageService,
    );
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
    const vueDefinition = resolveVueImportDefinition(
      ts,
      support,
      fileName,
      position,
      languageService,
    );
    return vueDefinition
      ? [vueDefinition.definition]
      : languageService.getTypeDefinitionAtPosition(fileName, position);
  };

  proxy.getQuickInfoAtPosition = (fileName, position) => {
    const vueDefinition = resolveVueImportDefinition(
      ts,
      support,
      fileName,
      position,
      languageService,
    );
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
      ...quickInfo,
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

function bind(fn, thisArg) {
  return typeof fn === "function" ? fn.bind(thisArg) : undefined;
}

module.exports = init;
