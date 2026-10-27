"use strict";

const crypto = require("node:crypto");
const path = require("node:path");
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
  const originalOptions = bind("getCompilationSettings");
  const originalProjectVersion = bind("getProjectVersion");
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
  const projectOptions = createProjectOptions(ts, info, originalOptions, readFile, fileExists);
  const projectConfiguration = createProjectConfiguration(
    engine,
    readFile,
    serverHost.watchFile?.bind(serverHost),
    originalProjectVersion,
    () => info.project?.refreshDiagnostics?.(),
    (configuration) => {
      try {
        info.project?.projectService?.logger?.info?.(
          `[vize] TypeScript project configuration ${JSON.stringify(configuration)}`,
        );
      } catch {
        /* Logging cannot affect the host. */
      }
    },
    (name) => projectOptions?.refresh(name),
  );
  const readSource = (fileName) => {
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
    return source;
  };
  const get = (fileName) => {
    const source = readSource(fileName);
    if (typeof source !== "string") return undefined;
    const settings = options();
    const project = engine.native
      ? projectConfiguration.get(settings)
      : { configuration: JSON.stringify([settings.checkJs, settings.configFilePath]) };
    const configuration = project.configuration;
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
      if (project.error) throw new Error(project.error);
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
  const options = () => {
    let original = originalOptions?.() || {};
    if (engine.native) {
      projectConfiguration.get(original);
      projectOptions?.apply();
      original = originalOptions?.() || {};
    }
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
    ...(originalProjectVersion
      ? {
          getProjectVersion() {
            const project = projectConfiguration.get(originalOptions?.() || {});
            projectOptions?.apply();
            return `${originalProjectVersion()}:${project.configuration}`;
          },
        }
      : {}),
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
  const support = { get, readSource, engine, resolve, fileExists, readFile, snapshots };
  let restore;
  support.dispose = () => {
    projectConfiguration.dispose();
    snapshots.clear();
    installedHosts.delete(host);
    restore?.();
  };
  if (!engine.native) return support;
  restore = installAtomically(host, replacements);
  if (!restore) {
    projectConfiguration.dispose();
    return undefined;
  }
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
    return () => {
      for (const [name, descriptor] of originals) {
        if (host[name] !== replacements[name]) continue;
        if (descriptor) Object.defineProperty(host, name, descriptor);
        else delete host[name];
      }
    };
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

// tsserver caches its extends resolution, including absent package candidates.
// Refresh only configured projects through public APIs; roots and references
// remain owned by tsserver. No extended-config cache enters this fresh parse.
function createProjectOptions(ts, info, originalOptions, readFile, fileExists) {
  const project = info.project;
  if (
    project?.projectKind !== ts.server?.ProjectKind.Configured ||
    typeof project.setCompilerOptions !== "function" ||
    typeof ts.getParsedCommandLineOfConfigFile !== "function"
  )
    return undefined;
  const server = info.serverHost || ts.sys;
  const controlled = new Set();
  let parsedOptions, applied;
  return {
    refresh(configFileName) {
      const parsed = ts.getParsedCommandLineOfConfigFile(configFileName, undefined, {
        ...server,
        readFile,
        fileExists,
        readDirectory: server.readDirectory.bind(server),
        getCurrentDirectory: project.getCurrentDirectory.bind(project),
        useCaseSensitiveFileNames: ts.sys.useCaseSensitiveFileNames,
        onUnRecoverableConfigFileDiagnostic() {},
      });
      if (!parsed) return;
      parsedOptions = parsed.options;
      applied = undefined;
      this.apply();
    },
    apply() {
      if (!parsedOptions) return;
      const original = originalOptions();
      if (original === applied) return;
      for (const name of Object.keys(parsedOptions))
        if (name !== "configFile") controlled.add(name);
      const changed = [...controlled].some(
        (name) => JSON.stringify(original[name]) !== JSON.stringify(parsedOptions[name]),
      );
      applied = original;
      if (!changed) return;
      const descriptors = Object.getOwnPropertyDescriptors(original);
      for (const name of controlled)
        if (!Object.hasOwn(parsedOptions, name)) delete descriptors[name];
      for (const [name, descriptor] of Object.entries(
        Object.getOwnPropertyDescriptors(parsedOptions),
      )) {
        // The server's SourceFile carries its root specs and reload state.
        // Preserve its exact non-enumerable descriptor and object identity.
        if (name === "configFile") continue;
        controlled.add(name);
        const prior = descriptors[name];
        descriptors[name] =
          prior && "value" in prior ? { ...prior, value: descriptor.value } : descriptor;
      }
      const options = Object.create(Object.getPrototypeOf(original), descriptors);
      applied = options;
      // Besides changing the option values, this public operation invalidates
      // the server's module-resolution cache when the selected options require it.
      // reloadProjects may subsequently restore its stale cached options. Reapply
      // this parsed snapshot to a changed host object without parsing it again.
      project.setCompilerOptions(options);
    },
  };
}

// One resolver snapshot belongs to the project, not to any individual SFC.
// Known files are read and hashed once per project version or watcher event.
function createProjectConfiguration(
  engine,
  readFile,
  watchFile,
  getProjectVersion,
  changed,
  trace,
  refreshOptions,
) {
  let configFile, epoch, metadata, error;
  let dirty = true;
  let disposed = false;
  let fingerprints = new Map();
  const watchers = new Map();
  const close = () => {
    for (const watcher of watchers.values()) watcher.close();
    watchers.clear();
  };
  function watch(files) {
    const retained = new Set(files);
    const added = new Map();
    try {
      if (watchFile)
        for (const file of retained)
          if (!watchers.has(file))
            added.set(
              file,
              watchFile(
                file,
                () => {
                  if (disposed) return;
                  dirty = true;
                  changed();
                },
                // Missing candidates use the server host's polling fallback.
                250,
              ),
            );
    } catch (cause) {
      for (const watcher of added.values()) watcher.close();
      throw cause;
    }
    for (const [file, watcher] of watchers)
      if (!retained.has(file)) {
        watcher.close();
        watchers.delete(file);
      }
    for (const [file, watcher] of added) watchers.set(file, watcher);
  }
  return {
    get(settings) {
      const name = typeof settings.configFilePath === "string" ? settings.configFilePath : "";
      if (name !== configFile) {
        close();
        configFile = name;
        metadata = undefined;
        error = undefined;
        fingerprints = new Map();
        dirty = true;
      }
      if (name) {
        const version = getProjectVersion?.();
        if (dirty || version !== epoch || (!metadata && !error)) {
          epoch = version;
          dirty = false;
          const observed = new Map();
          const fingerprint = (file) => {
            if (observed.has(file)) return observed.get(file);
            let source;
            try {
              source = readFile(file);
            } catch {
              // Resolver probes can be missing files or package directories.
            }
            const digest =
              typeof source === "string"
                ? crypto.createHash("sha256").update(source).digest("hex")
                : undefined;
            observed.set(file, digest);
            return digest;
          };
          let altered = false;
          // Every known input must precede the native snapshot, even after the
          // first mismatch. Later reads must not bless concurrently changed bytes.
          for (const [file, digest] of fingerprints)
            if (fingerprint(file) !== digest) altered = true;
          if (!metadata || error || altered) {
            try {
              if (typeof engine.native?.getVueLanguageProjectMetadata !== "function")
                throw new Error("Native project configuration metadata is unavailable");
              let next;
              for (let capture = 0; capture < 3; capture++) {
                const candidate = engine.native.getVueLanguageProjectMetadata(name);
                if (
                  !candidate ||
                  typeof candidate.configIdentity !== "string" ||
                  !Array.isArray(candidate.watchedFiles) ||
                  candidate.watchedFiles.some(
                    (file) => typeof file !== "string" || !path.isAbsolute(file),
                  )
                )
                  throw new Error("Invalid native project configuration metadata");
                const discovered = candidate.watchedFiles.filter((file) => !observed.has(file));
                if (!discovered.length) {
                  next = candidate;
                  break;
                }
                // Newly discovered inputs must also be watched and fingerprinted
                // before the snapshot whose identity we eventually admit.
                watch([...watchers.keys(), ...candidate.watchedFiles]);
                for (const file of discovered) fingerprint(file);
              }
              if (!next)
                throw new Error("Project configuration dependencies changed during three captures");
              watch(next.watchedFiles);
              metadata = next;
              error = undefined;
              trace({
                configFileName: name,
                configIdentity: next.configIdentity,
                nativeWatchedFiles: next.watchedFiles,
                registeredFiles: [...watchers.keys()],
              });
            } catch (cause) {
              error = cause instanceof Error ? cause.message : String(cause);
              watch([...new Set([name, ...watchers.keys()])]);
            }
            refreshOptions?.(name);
            epoch = getProjectVersion?.();
          }
          fingerprints = new Map(
            (metadata?.watchedFiles || [name]).map((file) => [file, fingerprint(file)]),
          );
        }
      }
      return {
        configuration: JSON.stringify([settings.checkJs, name, error || metadata?.configIdentity]),
        error,
      };
    },
    dispose() {
      disposed = true;
      close();
    },
  };
}

function isSfc(fileName) {
  return typeof fileName === "string" && /\.(?:vue|art)$/i.test(fileName);
}
module.exports = { installVueVirtualModules, isSfc };
