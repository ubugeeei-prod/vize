"use strict";

const crypto = require("node:crypto");
const { loadNative } = require("./provider.cjs");
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
  const engine = loadNative(root, info.config?.nativePath);
  try {
    info.project?.projectService?.logger?.info?.(
      `[vize] TypeScript native provider ${JSON.stringify({ origin: engine.origin, path: engine.path, version: engine.version, source: engine.source, target: engine.target, nativeBuildToken: engine.nativeBuildToken, error: engine.error, unavailable: engine.unavailable })}`,
    );
  } catch {
    /* Logging cannot affect the host. */
  }
  const snapshots = new Map();
  const projectVersion = bind("getProjectVersion");
  let config;
  // Every SFC query shares one config read per project version.
  const configText = (fileName) => {
    if (typeof fileName !== "string") return "";
    const version = projectVersion?.();
    if (version === undefined || config?.fileName !== fileName || config.version !== version)
      config = { fileName, version, text: readFile(fileName) };
    return config.text;
  };
  const get = (fileName) => {
    if (!isSfc(fileName)) return undefined;
    const originalSnapshot = getSnapshot(fileName);
    const scriptInfo = info.project?.projectService?.getScriptInfo?.(fileName);
    // tsserver deliberately exposes an empty snapshot for closed mixed files.
    // An open, unsaved empty file is authoritative and must remain empty.
    const closedMixed = scriptInfo?.hasMixedContent && scriptInfo.isScriptOpen?.() === false;
    const raw = closedMixed ? undefined : originalSnapshot;
    const source = raw?.getText(0, raw.getLength()) ?? readFile(fileName);
    if (typeof source !== "string") return undefined;
    if (closedMixed && originalSnapshot?.getText(0, originalSnapshot.getLength()) !== source) {
      // Reference/rename protocol responses use ScriptInfo's own line map and
      // snapshot, rather than LanguageService.toLineColumnOffset. Populate the
      // authored disk text through its public API; open buffers are untouched.
      scriptInfo.reloadFromFile?.(fileName);
    }
    const settings = host.getCompilationSettings?.() || {};
    const configSource = configText(settings.configFilePath);
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
      if (!engine.native) throw new Error(`Native projection unavailable: ${engine.error}`);
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
  const options = () => {
    const original = originalOptions?.() || {};
    // TypeScript stores the parsed configFile as a non-enumerable option.
    // Losing it crashes configured-project reload when another SFC opens.
    return Object.create(Object.getPrototypeOf(original), {
      ...Object.getOwnPropertyDescriptors(original),
      allowNonTsExtensions: { value: true, enumerable: true, configurable: true, writable: true },
    });
  };
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
  // A spread omits Project prototype methods, such as its own current directory.
  for (const name of ["getCurrentDirectory", "directoryExists", "getDirectories", "realpath"]) {
    const method = bind(name);
    if (method) resolverHost[name] = method;
  }
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
    // Respect a real declaration or TypeScript module selected by TypeScript,
    // such as Card.d.vue.ts. Only our nonexistent .vue.ts probe is virtual.
    if (result && !isSfc(result.resolvedFileName) && fileExists(result.resolvedFileName))
      return result;
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
      const [literals, containingFile, redirectedReference, compilerOptions, sourceFile] = args;
      const prior = previousLiterals?.(...args) || [];
      return literals.map((literal, index) => {
        // Ordinary imports keep the host's own result and resolution cache.
        if (previousLiterals && !isSfc(literal.text)) return prior[index];
        // String literals carry no mode; NodeNext must select import or require conditions.
        const mode =
          sourceFile && ts.getModeForUsageLocation
            ? ts.getModeForUsageLocation(sourceFile, literal, compilerOptions || options())
            : undefined;
        // ESM performs no extension probing, so the virtual .vue.ts probe keeps the
        // mode-less resolver when the selected import conditions find nothing.
        const resolvedModule =
          (mode !== undefined &&
            resolve(literal.text, containingFile, redirectedReference, mode)) ||
          resolve(literal.text, containingFile, redirectedReference);
        return resolvedModule ? { resolvedModule } : prior[index] || { resolvedModule: undefined };
      });
    },
    resolveModuleNames(...args) {
      const [names, containingFile, , redirectedReference] = args;
      const prior = previousNames?.(...args) || [];
      return names.map((name, index) =>
        previousNames && !isSfc(name)
          ? prior[index]
          : resolve(name, containingFile, redirectedReference) || prior[index],
      );
    },
  };
  const support = { get, engine, resolve, fileExists, readFile, snapshots };
  if (!engine.native) return support;
  if (!installAtomically(host, replacements)) return undefined;
  installedHosts.set(host, support);
  return support;
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
