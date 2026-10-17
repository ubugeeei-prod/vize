// Embedded by the native loader. The nearest installed project Vite owns loading.
import { lstatSync, realpathSync } from "node:fs";
import { mkdtemp, realpath, rm, writeFile } from "node:fs/promises";
import { createRequire, globalPaths } from "node:module";
import * as nodeModule from "node:module";
import { tmpdir } from "node:os";
import { dirname as moduleConfigDirectory, isAbsolute, join, relative, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

export function configModuleImportPath(configURL, windows = process.platform === "win32") {
  return fileURLToPath(configURL, { windows });
}

export async function importConfigModule(configPath, withoutVite) {
  return (
    await importConfigModuleWithDependencies(configPath, nodeModule.registerHooks, withoutVite)
  ).namespace;
}

/** Keep runtime imports observable until the optional export evaluator settles. */
async function captureConfigEvaluation(
  load,
  configPath,
  registerModuleHooks,
  evaluate,
  privateDirectory,
) {
  const files = new Set([configPath]);
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
              }
            }
            return result;
          },
        })
      : undefined;
  try {
    const { namespace, dependencies = [] } = await load();
    for (const path of dependencies) if (!isPrivate(path)) files.add(path);
    const value = evaluate ? await evaluate(namespace) : namespace;
    return { namespace, value, dependencies: [...files] };
  } finally {
    hooks?.deregister();
  }
}

/** Await evaluate(namespace) once before sealing its graph; retain the namespace unchanged. */
export async function importConfigModuleWithDependencies(
  configPath,
  registerModuleHooks = nodeModule.registerHooks,
  withoutVite,
  evaluate,
) {
  const configURL = pathToFileURL(configPath);
  const require = createRequire(configURL);
  const searchPaths = require.resolve.paths("vite") ?? [];
  const candidate = searchPaths
    .filter((directory) => !globalPaths.includes(directory))
    .map((directory) => join(directory, "vite"))
    .find((directory) => lstatSync(directory, { throwIfNoEntry: false }) !== undefined);
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
    return captureConfigEvaluation(
      async () => ({
        namespace: await (withoutVite ? withoutVite(configURL) : import(configURL.href)),
      }),
      configPath,
      registerModuleHooks,
      evaluate,
    );
  }
  const viteDirectory = realpathSync(candidate);
  const viteEntry = realpathSync(require.resolve("vite"));
  const entryPath = relative(viteDirectory, viteEntry);
  if (entryPath === ".." || entryPath.startsWith(`..${sep}`) || isAbsolute(entryPath)) {
    throw new Error(`Resolved loader escapes selected project Vite: ${candidate} -> ${viteEntry}`);
  }
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
    );
  } finally {
    await rm(temporary, { recursive: true, force: true });
  }
}
