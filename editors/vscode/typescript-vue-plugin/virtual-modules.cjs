"use strict";

const crypto = require("node:crypto");
const { createRequire } = require("node:module");
const path = require("node:path");

const installedHosts = new WeakMap();
const nativePackage = "@vizejs/native";
const nativeAnchors = [
  "@vizejs/vite-plugin",
  "@vizejs/unplugin",
  "@vizejs/rspack-plugin",
  "@vizejs/vite-plugin-musea",
  "vize",
];

function installVueVirtualModules(ts, info) {
  const host = info.languageServiceHost;
  if (!host) return false;

  const installed = installedHosts.get(host);
  if (installed !== undefined) return installed;

  const projectRoot =
    safeCall(() => info.project?.getCurrentDirectory?.()) ||
    safeCall(() => host.getCurrentDirectory?.());
  const native = loadNative(projectRoot);
  if (!native) {
    installedHosts.set(host, false);
    return false;
  }

  const serverHost = info.serverHost || ts.sys;
  const fileExists = bind(host.fileExists, host) || bind(serverHost.fileExists, serverHost);
  const readFile = bind(host.readFile, host) || bind(serverHost.readFile, serverHost);
  const getSnapshot = bind(host.getScriptSnapshot, host);
  const getScriptKind = bind(host.getScriptKind, host);
  const getScriptVersion = bind(host.getScriptVersion, host);
  const resolveLiterals = bind(host.resolveModuleNameLiterals, host);
  const resolveNames = bind(host.resolveModuleNames, host);
  if (!fileExists || !readFile || !getSnapshot) {
    installedHosts.set(host, false);
    return false;
  }

  const snapshots = new Map();
  const replacements = {
    getScriptKind(fileName) {
      return isVuePath(fileName)
        ? ts.ScriptKind.TS
        : (getScriptKind?.(fileName) ?? ts.ScriptKind.Unknown);
    },
    getScriptSnapshot(fileName) {
      if (!isVuePath(fileName)) return getSnapshot(fileName);
      const source = readSnapshotText(getSnapshot(fileName)) ?? readFile(fileName);
      if (typeof source !== "string") return undefined;

      const cached = snapshots.get(fileName);
      if (cached?.source === source) return cached.snapshot;
      const version = sourceVersion(source);

      try {
        const result = native.typeCheck(source, {
          filename: fileName,
          includeVirtualTs: true,
        });
        if (typeof result?.virtualTs !== "string" || result.virtualTs.length === 0) {
          throw new Error("native typeCheck returned no virtual TypeScript");
        }
        const snapshot = ts.ScriptSnapshot.fromString(result.virtualTs);
        snapshots.set(fileName, { snapshot, source, version });
        return snapshot;
      } catch (error) {
        logPluginError(info, "generate virtual module", error);
        const snapshot = ts.ScriptSnapshot.fromString(fallbackVirtualModule());
        snapshots.set(fileName, { snapshot, source, version });
        return snapshot;
      }
    },
    getScriptVersion(fileName) {
      if (!isVuePath(fileName)) return getScriptVersion?.(fileName) ?? "0";
      const snapshot = getSnapshot(fileName);
      const source = readSnapshotText(snapshot) ?? readFile(fileName) ?? "";
      const cached = snapshots.get(fileName);
      return cached?.source === source ? cached.version : sourceVersion(source);
    },
    resolveModuleNameLiterals(...args) {
      const [literals, containingFile] = args;
      const previous = toArray(resolveLiterals?.(...args));
      if (!Array.isArray(literals)) return previous;
      return literals.map((literal, index) => {
        if (previous[index]?.resolvedModule) return previous[index];
        const resolvedModule = resolveVueModule(ts, literal?.text, containingFile, fileExists);
        return resolvedModule
          ? { resolvedModule }
          : previous[index] || { resolvedModule: undefined };
      });
    },
    resolveModuleNames(...args) {
      const [names, containingFile] = args;
      const previous = toArray(resolveNames?.(...args));
      if (!Array.isArray(names)) return previous;
      return names.map(
        (name, index) => previous[index] || resolveVueModule(ts, name, containingFile, fileExists),
      );
    },
  };

  if (!installAtomically(host, replacements)) {
    installedHosts.set(host, false);
    return false;
  }
  installedHosts.set(host, true);
  return true;
}

function loadNative(projectRoot) {
  if (typeof projectRoot !== "string") return undefined;
  for (const anchor of nativeAnchors) {
    try {
      const anchorPath = require.resolve(anchor, { paths: [projectRoot] });
      const native = tryRequire(createRequire(anchorPath), nativePackage);
      if (native) return native;
    } catch {
      // Try the next package that carries the matching native binding.
    }
  }
  return tryRequire(require, nativePackage, projectRoot);
}

function tryRequire(loader, specifier, searchRoot) {
  try {
    const resolved = searchRoot
      ? require.resolve(specifier, { paths: [searchRoot] })
      : loader.resolve(specifier);
    const loaded = loader(resolved);
    return typeof loaded?.typeCheck === "function" ? loaded : undefined;
  } catch {
    return undefined;
  }
}

function resolveVueModule(ts, specifier, containingFile, fileExists) {
  if (!isRelativeVueSpecifier(specifier) || typeof containingFile !== "string") return undefined;
  const vuePath = path.resolve(path.dirname(containingFile), specifier);
  if (!fileExists(vuePath)) return undefined;
  return {
    extension: ts.Extension.Ts,
    isExternalLibraryImport: false,
    resolvedFileName: vuePath,
  };
}

function installAtomically(host, replacements) {
  const entries = Object.entries(replacements);
  if (!entries.every(([name]) => canAssignProperty(host, name))) return false;
  const originals = new Map(
    entries.map(([name]) => [name, { own: Object.hasOwn(host, name), value: host[name] }]),
  );
  try {
    for (const [name, replacement] of entries) host[name] = replacement;
    return true;
  } catch {
    for (const [name, original] of originals) {
      try {
        if (original.own) host[name] = original.value;
        else delete host[name];
      } catch {
        // Best-effort rollback; the caller retains the unmodified language service.
      }
    }
    return false;
  }
}

function canAssignProperty(target, key) {
  let current = target;
  while (current) {
    const descriptor = Object.getOwnPropertyDescriptor(current, key);
    if (descriptor) {
      return "writable" in descriptor ? descriptor.writable : typeof descriptor.set === "function";
    }
    current = Object.getPrototypeOf(current);
  }
  return Object.isExtensible(target);
}

function isRelativeVueSpecifier(specifier) {
  return (
    typeof specifier === "string" &&
    isVuePath(specifier) &&
    (specifier.startsWith("./") || specifier.startsWith("../"))
  );
}

function isVuePath(fileName) {
  return typeof fileName === "string" && fileName.toLowerCase().endsWith(".vue");
}

function readSnapshotText(snapshot) {
  return snapshot ? snapshot.getText(0, snapshot.getLength()) : undefined;
}

function sourceVersion(source) {
  return crypto.createHash("sha1").update(source).digest("base64url");
}

function fallbackVirtualModule() {
  return "declare const component: import('vue').DefineComponent<Record<string, unknown>>;\nexport default component;\n";
}

function logPluginError(info, phase, error) {
  try {
    const message = error instanceof Error ? error.message : String(error);
    info?.project?.projectService?.logger?.info?.(`[vize] ${phase} failed: ${message}`);
  } catch {
    // Logging must never terminate tsserver.
  }
}

function bind(fn, thisArg) {
  return typeof fn === "function" ? fn.bind(thisArg) : undefined;
}

function safeCall(fn) {
  try {
    return fn();
  } catch {
    return undefined;
  }
}

function toArray(value) {
  return Array.isArray(value) ? value : [];
}

module.exports = { installVueVirtualModules };
