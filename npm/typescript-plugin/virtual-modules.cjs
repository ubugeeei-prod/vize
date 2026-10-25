"use strict";

const crypto = require("node:crypto");
const path = require("node:path");
const { createRequire } = require("node:module");
const installedHosts = new WeakMap();

// A source snapshot is the sole invalidation authority, including unsaved edits.
function installVueVirtualModules(ts, info) {
  const host = info.languageServiceHost;
  if (!host) return undefined;
  if (installedHosts.has(host)) return installedHosts.get(host);
  const serverHost = info.serverHost || ts.sys;
  const bind = (name) =>
    typeof host[name] === "function" ? host[name].bind(host) : serverHost[name]?.bind(serverHost);
  const readFile = bind("readFile");
  const fileExists = bind("fileExists");
  const getSnapshot = bind("getScriptSnapshot");
  if (!readFile || !fileExists || !getSnapshot) return undefined;
  const root = host.getCurrentDirectory?.() || info.project?.getCurrentDirectory?.();
  const engine = loadNative(root);
  const snapshots = new Map();
  const get = (fileName) => {
    if (!isSfc(fileName)) return undefined;
    const raw = getSnapshot(fileName);
    const source = raw?.getText(0, raw.getLength()) ?? readFile(fileName);
    if (typeof source !== "string") return undefined;
    const settings = host.getCompilationSettings?.() || {};
    const configSource =
      typeof settings.configFilePath === "string" ? readFile(settings.configFilePath) : "";
    const configuration = JSON.stringify([settings.checkJs, settings.configFilePath, configSource]);
    const cached = snapshots.get(fileName);
    if (cached?.source === source && cached.configuration === configuration) return cached;
    const version = crypto
      .createHash("sha256")
      .update(source)
      .update("\0")
      .update(configuration)
      .digest("hex");
    let projection, error;
    try {
      if (!engine)
        throw new Error("A matching @vizejs/native with generateVueLanguageCode is required");
      projection = engine.toUtf16LanguageProjection(
        source,
        engine.native.generateVueLanguageCode(source, {
          filename: fileName,
          configFileName: settings.configFilePath,
          checkJavaScript: settings.checkJs === true,
          preserveUnusedDiagnostics: true,
        }),
      );
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
    // Failed transforms remain explicit errors, never a permissive component type.
    const snapshot = ts.ScriptSnapshot.fromString(projection?.text ?? "export {};\n");
    const value = { source, configuration, version, projection, error, snapshot };
    snapshots.set(fileName, value);
    return value;
  };
  const originalOptions = bind("getCompilationSettings");
  const options = () => ({ ...originalOptions?.(), allowNonTsExtensions: true });
  const resolverHost = {
    ...serverHost,
    ...host,
    readFile,
    fileExists(candidate) {
      return (
        fileExists(candidate) ||
        (/\.(?:vue|art)\.tsx?$/.test(candidate) && fileExists(candidate.replace(/\.tsx?$/, "")))
      );
    },
  };
  function resolve(name, containingFile, redirectedReference, resolutionMode) {
    if (typeof name !== "string") return undefined;
    const result = ts.resolveModuleName(
      name,
      containingFile,
      options(),
      resolverHost,
      undefined,
      redirectedReference,
      resolutionMode,
    ).resolvedModule;
    const resolvedFileName = result?.resolvedFileName.replace(/(\.(?:vue|art))\.tsx?$/, "$1");
    if (!isSfc(name)) return result;
    if (!resolvedFileName || !isSfc(resolvedFileName) || !fileExists(resolvedFileName))
      return undefined;
    return { ...result, resolvedFileName, extension: ts.Extension.Ts };
  }
  const previousLiterals = bind("resolveModuleNameLiterals");
  const previousNames = bind("resolveModuleNames");
  const previousKind = bind("getScriptKind");
  const previousVersion = bind("getScriptVersion");
  const replacements = {
    getCompilationSettings: options,
    getScriptSnapshot: (name) => (isSfc(name) ? get(name)?.snapshot : getSnapshot(name)),
    getScriptKind: (name) =>
      isSfc(name) ? ts.ScriptKind.TSX : (previousKind?.(name) ?? ts.ScriptKind.Unknown),
    getScriptVersion: (name) =>
      isSfc(name) ? (get(name)?.version ?? "0") : (previousVersion?.(name) ?? "0"),
    resolveModuleNameLiterals(...args) {
      const [literals, containingFile, redirectedReference] = args;
      const prior = previousLiterals?.(...args) || [];
      return literals.map((literal, index) => {
        const resolvedModule = resolve(
          literal.text,
          containingFile,
          redirectedReference,
          literal.resolutionMode,
        );
        return resolvedModule ? { resolvedModule } : prior[index] || { resolvedModule: undefined };
      });
    },
    resolveModuleNames(...args) {
      const [names, containingFile, , redirectedReference] = args;
      const prior = previousNames?.(...args) || [];
      return names.map(
        (name, index) => resolve(name, containingFile, redirectedReference) || prior[index],
      );
    },
  };
  const support = { get, engine, resolve, fileExists, readFile, snapshots };
  if (!installAtomically(host, replacements)) return undefined;
  installedHosts.set(host, support);
  return support;
}

function loadNative(projectRoot) {
  const roots = [];
  if (typeof projectRoot === "string") {
    for (const anchor of [
      "@vizejs/vite-plugin",
      "@vizejs/unplugin",
      "@vizejs/rspack-plugin",
      "@vizejs/vite-plugin-musea",
      "vize",
    ]) {
      try {
        roots.push(createRequire(require.resolve(anchor, { paths: [projectRoot] })));
      } catch {}
    }
    roots.push(createRequire(path.join(projectRoot, "package.json")));
  }
  roots.push(require);
  for (const loader of roots) {
    try {
      const entry = loader.resolve("@vizejs/native");
      const native = loader(entry);
      const helper = loader(path.join(path.dirname(entry), "language-projection.js"));
      if (
        typeof native.generateVueLanguageCode === "function" &&
        typeof helper.toUtf16LanguageProjection === "function" &&
        typeof helper.applyDiagnosticDirectives === "function"
      )
        return { native, ...helper };
    } catch {
      /* Resolve the engine and its helper from one physical package. */
    }
  }
  return undefined;
}

function installAtomically(host, replacements) {
  const originals = Object.entries(replacements).map(([name]) => [
    name,
    Object.getOwnPropertyDescriptor(host, name),
  ]);
  try {
    for (const [name, value] of Object.entries(replacements)) {
      if (!Reflect.set(host, name, value) || host[name] !== value)
        throw new Error("Host is read-only");
    }
    return true;
  } catch {
    for (const [name, descriptor] of originals) {
      try {
        if (descriptor) Object.defineProperty(host, name, descriptor);
        else delete host[name];
      } catch {}
    }
    return false;
  }
}

function isSfc(fileName) {
  return typeof fileName === "string" && /\.(?:vue|art)$/i.test(fileName);
}
module.exports = { installVueVirtualModules, isSfc };
