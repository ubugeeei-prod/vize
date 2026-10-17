// Embedded by the native loader. The nearest installed project Vite owns loading.
import { lstatSync, realpathSync } from "node:fs";
import { mkdtemp, realpath, rm, writeFile } from "node:fs/promises";
import { createRequire, globalPaths } from "node:module";
import * as nodeModule from "node:module";
import { tmpdir } from "node:os";
import {
  dirname as moduleConfigDirectory,
  extname,
  isAbsolute,
  join,
  relative,
  sep,
} from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

export function configModuleImportPath(configURL, windows = process.platform === "win32") {
  return fileURLToPath(configURL, { windows });
}

export async function importConfigModule(configPath, withoutVite) {
  return (
    await importConfigModuleWithDependencies(configPath, nodeModule.registerHooks, withoutVite)
  ).namespace;
}

/** Observe real failed requests, without choosing a replacement resolution. */
export function failedConfigModuleRequests(error) {
  return failedConfigModuleObservation(error).paths;
}

function failedConfigModuleObservation(error) {
  const paths = new Set();
  let complete = false;
  if (error?.code === "ERR_MODULE_NOT_FOUND" && typeof error.url === "string") {
    try {
      const url = new URL(error.url);
      if (url.protocol === "file:") {
        paths.add(fileURLToPath(url));
        complete = true;
      }
    } catch {}
  }
  const diagnostics = Array.isArray(error?.errors) ? error.errors : [];
  if (diagnostics.length) complete = true;
  for (const diagnostic of diagnostics) {
    let requested = false;
    if (
      diagnostic?.code !== "UNRESOLVED_IMPORT" ||
      typeof diagnostic.id !== "string" ||
      !isAbsolute(diagnostic.id) ||
      typeof diagnostic.exporter !== "string"
    ) {
      complete = false;
      continue;
    }
    paths.add(diagnostic.id);
    const request = diagnostic.exporter;
    // Public bundle diagnostics expose the requested importer/specifier, not
    // extension or package search candidates. Observe only explicit file paths.
    if (!(isAbsolute(request) || request.startsWith("./") || request.startsWith("../"))) {
      complete = false;
      continue;
    }
    // Bundle specifiers may treat these characters as literal filenames or URL
    // syntax. Its diagnostic does not expose which interpretation won.
    if (/[%#?]/.test(request)) {
      complete = false;
      continue;
    }
    try {
      const url = isAbsolute(request)
        ? pathToFileURL(request)
        : new URL(request, pathToFileURL(diagnostic.id));
      if (url.protocol === "file:" && extname(url.pathname)) {
        paths.add(fileURLToPath(url));
        requested = true;
      }
    } catch {}
    complete &&= requested;
  }
  return { paths: [...paths], complete };
}

/** Keep runtime imports observable until the optional export evaluator settles. */
async function captureConfigEvaluation(
  load,
  configPath,
  registerModuleHooks,
  evaluate,
  privateDirectory,
  observeDependencies,
) {
  const files = new Set([configPath]);
  const requestedFiles = new Set();
  let complete = true;
  let moduleGraphLoaded = false;
  const owners = new Set([pathToFileURL(configPath).href]);
  const isPrivate = (path) => {
    if (!privateDirectory) return false;
    const inside = relative(privateDirectory, path);
    return inside !== ".." && !inside.startsWith(`..${sep}`) && !isAbsolute(inside);
  };
  const hooks =
    typeof registerModuleHooks === "function"
      ? registerModuleHooks({
          resolve(specifier, context, nextResolve) {
            const result = nextResolve(specifier, context);
            if (result.url.startsWith("file:")) {
              const path = fileURLToPath(result.url);
              // Vite's generated module owns config execution, but its build engine
              // and private adapters are not authored configuration dependencies.
              if (owners.has(result.url) || owners.has(context.parentURL) || isPrivate(path)) {
                owners.add(result.url);
                if (!isPrivate(path)) files.add(path);
                // Node reports the real target, but the actual explicit request
                // also owns a symlink route whose retargeting must be observed.
                // Keep this watch-only: it does not replace Node's resolution.
                if (
                  owners.has(context.parentURL) &&
                  (specifier.startsWith("./") ||
                    specifier.startsWith("../") ||
                    specifier.startsWith("/") ||
                    specifier.startsWith("file:"))
                ) {
                  try {
                    const requested = new URL(specifier, context.parentURL);
                    if (requested.protocol === "file:") {
                      const requestedPath = fileURLToPath(requested);
                      if (extname(requestedPath) && !isPrivate(requestedPath)) {
                        requestedFiles.add(requestedPath);
                      }
                    }
                  } catch {}
                }
              }
            }
            return result;
          },
        })
      : undefined;
  try {
    const { namespace, dependencies = [] } = await load();
    for (const path of dependencies) if (!isPrivate(path)) files.add(path);
    moduleGraphLoaded = true;
    const value = evaluate ? await evaluate(namespace) : namespace;
    return { namespace, value, dependencies: [...files] };
  } catch (error) {
    complete = false;
    // User code may throw arbitrary objects, including getters that throw.
    // Observation must never replace the loader's original exception.
    try {
      const observed = failedConfigModuleObservation(error);
      // A failed bundle reports its failing importer/request, but can omit
      // intermediate modules read by the build engine. Keep useful targets
      // without treating that partial graph as enough to suppress repair.
      complete = observed.complete && (privateDirectory === undefined || moduleGraphLoaded);
      for (const path of observed.paths) if (!isPrivate(path)) files.add(path);
    } catch {}
    throw error;
  } finally {
    hooks?.deregister();
    observeDependencies?.([...files, ...requestedFiles], complete);
  }
}

/** Await evaluate(namespace) once before sealing its graph; retain the namespace unchanged. */
export async function importConfigModuleWithDependencies(
  configPath,
  registerModuleHooks = nodeModule.registerHooks,
  withoutVite,
  evaluate,
  observeDependencies,
  beforeRead,
) {
  const watchInputs = new Set([configPath]);
  let complete = false;
  const observe = (paths, observedComplete) => {
    for (const path of paths) watchInputs.add(path);
    complete = observedComplete;
  };
  try {
    const configURL = pathToFileURL(configPath);
    const require = createRequire(configURL);
    const searchPaths = require.resolve.paths("vite") ?? [];
    let candidate;
    for (const directory of searchPaths.filter((path) => !globalPaths.includes(path))) {
      const requested = join(directory, "vite");
      watchInputs.add(requested);
      await beforeRead?.([requested]);
      if (lstatSync(requested, { throwIfNoEntry: false }) !== undefined) {
        candidate = requested;
        watchInputs.add(join(candidate, "package.json"));
        break;
      }
    }
    // The public CLI can retain its explicit TypeScript transform fallback. Native
    // editor snapshots always request the complete real import graph instead.
    if (typeof registerModuleHooks !== "function") {
      if (evaluate) {
        throw new Error(
          "Tracking imports while evaluating a Vite config requires Node.js 22.15 or newer. Upgrade Node.js; installed Vite alone cannot track runtime imports.",
        );
      }
      if (candidate === undefined && !withoutVite) {
        throw new Error(
          "Loading a Vite config without project Vite requires Node.js 22.15 or newer to track imported configuration files. Upgrade Node.js or install Vite in the project.",
        );
      }
    }
    if (candidate === undefined) {
      await beforeRead?.([configPath]);
      return await captureConfigEvaluation(
        async () => ({
          namespace: await (withoutVite ? withoutVite(configURL) : import(configURL.href)),
        }),
        configPath,
        registerModuleHooks,
        evaluate,
        undefined,
        observe,
      );
    }
    const viteDirectory = realpathSync(candidate);
    watchInputs.add(join(viteDirectory, "package.json"));
    await beforeRead?.([join(candidate, "package.json"), join(viteDirectory, "package.json")]);
    const resolvedEntry = require.resolve("vite");
    watchInputs.add(resolvedEntry);
    const viteEntry = realpathSync(resolvedEntry);
    watchInputs.add(viteEntry);
    const entryPath = relative(viteDirectory, viteEntry);
    if (entryPath === ".." || entryPath.startsWith(`..${sep}`) || isAbsolute(entryPath)) {
      throw new Error(
        `Resolved loader escapes selected project Vite: ${candidate} -> ${viteEntry}`,
      );
    }
    await beforeRead?.([resolvedEntry, viteEntry]);
    const { loadConfigFromFile } = await import(pathToFileURL(viteEntry).href);
    const temporary = await realpath(await mkdtemp(join(tmpdir(), "vize-config-module-")));
    try {
      const adapter = join(temporary, "config.mjs");
      await writeFile(
        adapter,
        `import * as original from ${JSON.stringify(configModuleImportPath(configURL))};\nexport default { namespace: original };\n`,
        { flag: "wx", mode: 0o600 },
      );
      return await captureConfigEvaluation(
        async () => {
          await beforeRead?.([configPath]);
          const loaded = await loadConfigFromFile(
            { mode: "development", command: "serve" },
            adapter,
            moduleConfigDirectory(configPath),
            "silent",
            undefined,
            "bundle",
          );
          if (!loaded) throw new Error(`Vite did not load the selected config: ${configPath}`);
          return {
            namespace: loaded.config.namespace,
            dependencies: loaded.dependencies,
          };
        },
        configPath,
        registerModuleHooks,
        evaluate,
        temporary,
        observe,
      );
    } finally {
      await rm(temporary, { recursive: true, force: true });
    }
  } finally {
    observeDependencies?.([...watchInputs], complete);
  }
}
