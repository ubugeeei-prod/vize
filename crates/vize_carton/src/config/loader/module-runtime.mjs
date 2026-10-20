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

// Optional compiled artifacts only; no imported namespace or evaluated config is retained.
export function enableConfigCompileCache(directory) {
  if (!directory || typeof nodeModule.enableCompileCache !== "function") return;
  if (
    [
      "NODE_COMPILE_CACHE",
      "NODE_DISABLE_COMPILE_CACHE",
      "NODE_COMPILE_CACHE_PORTABLE",
      "NODE_V8_COVERAGE",
    ].some((name) => Object.hasOwn(process.env, name)) ||
    process.env.NODE_OPTIONS?.includes("coverage")
  )
    return;
  // The parent owns this private directory until every fresh child has exited.
  // Uses Node's standard source-checksum/version contract, not Canon input identity.
  // Normal process exit flushes compiled code before the parent releases its lease.
  return nodeModule.enableCompileCache(directory);
}

// Failure identity is host metadata; never attach it to authored exports/errors.
const configFailureArtifacts = new WeakMap();
const configEvaluationTimings = new WeakMap();

// Private host diagnostics, never attached to authored values or exceptions.
export function configModuleEvaluationTimings(value) {
  return configEvaluationTimings.get(value)?.map((row) => ({ ...row }));
}

function beginConfigTiming(rows, phase, boundary) {
  const start = rows ? performance.now() : undefined;
  let finished = false;
  if (boundary) boundary.active = true;
  return () => {
    if (start !== undefined && !finished) {
      const end = performance.now();
      rows.push({ phase, elapsedMs: end - start, processId: process.pid });
      if (boundary) {
        boundary.active = false;
        // No observed resolution means UNKNOWN: never fabricate zero or guess
        // an SDK-generated path outside our already-owned private directory.
        if (boundary.resolvedAt !== undefined) {
          rows.push({
            phase: "public_load_until_owned_resolve",
            elapsedMs: boundary.resolvedAt - start,
            processId: process.pid,
          });
          rows.push({
            phase: "public_load_after_owned_resolve",
            elapsedMs: end - boundary.resolvedAt,
            processId: process.pid,
          });
        }
      }
      finished = true;
    }
  };
}

// Diagnostic only: retain the complete sampling packet, never an evaluated value.
// Setup/stop costs are separate; samples also include the adjacent host call edges.
async function beginPublicLoadProfile() {
  const started = performance.now();
  let session;
  const close = () => {
    try {
      session?.disconnect();
    } catch {}
  };
  try {
    const { Session } = await import("node:inspector/promises");
    session = new Session();
    session.connect();
    await session.post("Profiler.enable");
    await session.post("Profiler.start");
    const setupMs = performance.now() - started;
    return async () => {
      const stopping = performance.now();
      try {
        const { profile } = await session.post("Profiler.stop");
        return { status: "recorded", setupMs, stopMs: performance.now() - stopping, profile };
      } catch (error) {
        return { status: "unavailable", setupMs, reason: String(error) };
      } finally {
        close();
      }
    };
  } catch (error) {
    close();
    return async () => ({ status: "unavailable", reason: String(error) });
  }
}

export function configModuleFailureArtifacts(error) {
  return configFailureArtifacts.get(error)?.map(([owned, source]) => [owned, source]) ?? [];
}

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
  ownedAdapter,
  timings,
  boundary,
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
              if (boundary?.active && boundary.resolvedAt === undefined && isPrivate(path)) {
                // Diagnostic only, after the same successful nextResolve. No
                // extra resolution/read and no change to dependency ownership.
                boundary.resolvedAt = performance.now();
              }
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
    const finish = beginConfigTiming(timings, "export_evaluation");
    let value;
    try {
      value = evaluate ? await evaluate(namespace) : namespace;
    } finally {
      finish();
    }
    return { namespace, value, dependencies: [...files] };
  } catch (error) {
    if (
      ownedAdapter &&
      error !== null &&
      (typeof error === "object" || typeof error === "function")
    ) {
      configFailureArtifacts.set(error, [
        [ownedAdapter, configPath],
        [pathToFileURL(ownedAdapter).href, pathToFileURL(configPath).href],
        [
          relative(moduleConfigDirectory(configPath), ownedAdapter),
          relative(moduleConfigDirectory(configPath), configPath),
        ],
      ]);
    }
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
  const timings = process.env.VIZE_TRACE_EDITOR_PREPARATION === "1" ? [] : undefined;
  const boundary = timings ? { active: false, resolvedAt: undefined } : undefined;
  const finishTotal = beginConfigTiming(timings, "module_total");
  const finishSelection = beginConfigTiming(timings, "engine_selection_with_ack");
  let selectionFinished = false;
  let result;
  const watchInputs = new Set([configPath]);
  let complete = false;
  const observe = (paths, observedComplete) => {
    for (const path of paths) watchInputs.add(path);
    complete = observedComplete;
  };
  try {
    const configURL = pathToFileURL(configPath);
    const require = createRequire(configURL);
    const engineNames = ["vite", "vite-plus"];
    let candidate;
    for (const engineName of engineNames) {
      const searchPaths = require.resolve.paths(engineName) ?? [];
      for (const directory of searchPaths.filter((path) => !globalPaths.includes(path))) {
        const requested = join(directory, engineName);
        watchInputs.add(requested);
        await beforeRead?.([requested]);
        if (lstatSync(requested, { throwIfNoEntry: false }) !== undefined) {
          candidate = { engineName, directory: requested };
          watchInputs.add(join(candidate.directory, "package.json"));
          break;
        }
      }
      if (candidate !== undefined) break;
    }
    // The public CLI can retain its explicit TypeScript transform fallback. Native
    // editor snapshots always request the complete real import graph instead.
    if (typeof registerModuleHooks !== "function") {
      if (evaluate) {
        throw new Error(
          "Tracking imports while evaluating a Vite config requires Node.js 22.15 or newer. Upgrade Node.js; an installed config bundler alone cannot track runtime imports.",
        );
      }
      if (candidate === undefined && !withoutVite) {
        throw new Error(
          "Loading a Vite config without project Vite or Vite+ requires Node.js 22.15 or newer to track imported configuration files. Upgrade Node.js or install Vite or Vite+ in the project.",
        );
      }
    }
    if (candidate === undefined) {
      finishSelection();
      selectionFinished = true;
      await beforeRead?.([configPath]);
      result = await captureConfigEvaluation(
        async () => ({
          namespace: await (withoutVite ? withoutVite(configURL) : import(configURL.href)),
        }),
        configPath,
        registerModuleHooks,
        evaluate,
        undefined,
        observe,
        undefined,
        timings,
      );
      return result;
    }
    const engineDirectory = realpathSync(candidate.directory);
    watchInputs.add(join(engineDirectory, "package.json"));
    await beforeRead?.([
      join(candidate.directory, "package.json"),
      join(engineDirectory, "package.json"),
    ]);
    const resolvedEntry = require.resolve(candidate.engineName);
    watchInputs.add(resolvedEntry);
    const engineEntry = realpathSync(resolvedEntry);
    watchInputs.add(engineEntry);
    const entryPath = relative(engineDirectory, engineEntry);
    if (entryPath === ".." || entryPath.startsWith(`..${sep}`) || isAbsolute(entryPath)) {
      throw new Error(
        `Resolved loader escapes selected project ${candidate.engineName}: ${candidate.directory} -> ${engineEntry}`,
      );
    }
    await beforeRead?.([resolvedEntry, engineEntry]);
    finishSelection();
    selectionFinished = true;
    const finishImport = beginConfigTiming(timings, "engine_import");
    let loadConfigFromFile;
    try {
      const engine = await import(pathToFileURL(engineEntry).href);
      loadConfigFromFile = engine.loadConfigFromFile ?? engine.default?.loadConfigFromFile;
      if (typeof loadConfigFromFile !== "function") {
        throw new Error(`${candidate.engineName} does not export loadConfigFromFile`);
      }
    } finally {
      finishImport();
    }
    const finishAdapter = beginConfigTiming(timings, "adapter_prepare");
    let temporary;
    try {
      temporary = await realpath(await mkdtemp(join(tmpdir(), "vize-config-module-")));
    } catch (error) {
      finishAdapter();
      throw error;
    }
    try {
      const adapter = join(temporary, "config.mjs");
      await writeFile(
        adapter,
        `import * as original from ${JSON.stringify(configModuleImportPath(configURL))};\nexport default { namespace: original };\n`,
        { flag: "wx", mode: 0o600 },
      );
      finishAdapter();
      result = await captureConfigEvaluation(
        async () => {
          const finishAck = beginConfigTiming(timings, "selected_before_read_ack");
          try {
            await beforeRead?.([configPath]);
          } finally {
            finishAck();
          }
          const finishProfile = timings ? await beginPublicLoadProfile() : undefined;
          const finishLoad = beginConfigTiming(timings, "public_load", boundary);
          let loaded;
          try {
            loaded = await loadConfigFromFile(
              { mode: "development", command: "serve" },
              adapter,
              moduleConfigDirectory(configPath),
              "silent",
              undefined,
              "bundle",
            );
          } finally {
            finishLoad();
            if (finishProfile) {
              timings.find((row) => row.phase === "public_load").cpuProfile = await finishProfile();
            }
          }
          if (!loaded) {
            throw new Error(
              `${candidate.engineName} did not load the selected config: ${configPath}`,
            );
          }
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
        adapter,
        timings,
        boundary,
      );
      return result;
    } catch (error) {
      finishAdapter();
      throw error;
    } finally {
      const finishCleanup = beginConfigTiming(timings, "adapter_cleanup");
      try {
        await rm(temporary, { recursive: true, force: true });
      } finally {
        finishCleanup();
      }
    }
  } catch (error) {
    if (timings && error !== null && (typeof error === "object" || typeof error === "function")) {
      configEvaluationTimings.set(error, timings);
    }
    throw error;
  } finally {
    if (!selectionFinished) finishSelection();
    finishTotal();
    if (timings && result) configEvaluationTimings.set(result, timings);
    observeDependencies?.([...watchInputs], complete);
  }
}
