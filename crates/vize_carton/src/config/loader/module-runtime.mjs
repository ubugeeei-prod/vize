// Embedded by the native loader. Use the selected project's installed Vite
// loader when available, without evaluating its default config factory twice.
import { lstatSync, realpathSync } from "node:fs";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { createRequire, globalPaths } from "node:module";
import { tmpdir } from "node:os";
import { dirname as moduleConfigDirectory, isAbsolute, join, relative, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

// Keep an absolute filesystem specifier in Vite's bundle graph. Converting
// the existing file URL removes Windows' verbatim prefix without changing
// the selected filesystem path or externalizing TypeScript imports in Vite 7.
export function configModuleImportPath(configURL, windows = process.platform === "win32") {
  return fileURLToPath(configURL, { windows });
}

export async function importConfigModule(
  configPath,
  withoutVite = (configURL) => import(configURL.href),
) {
  const configURL = pathToFileURL(configPath);
  const require = createRequire(configURL);
  const searchPaths = require.resolve.paths("vite") ?? [];
  const candidate = searchPaths
    .filter((directory) => !globalPaths.includes(directory))
    .map((directory) => join(directory, "vite"))
    .find((directory) => lstatSync(directory, { throwIfNoEntry: false }) !== undefined);
  // A dependency-free config retains its native Node module-loading contract.
  // A present but broken Vite installation, or any config error, is an error.
  if (candidate === undefined) return withoutVite(configURL);

  const viteDirectory = realpathSync(candidate);
  const viteEntry = realpathSync(require.resolve("vite"));
  const entryPath = relative(viteDirectory, viteEntry);
  if (entryPath === ".." || entryPath.startsWith(`..${sep}`) || isAbsolute(entryPath)) {
    throw new Error(`Resolved loader escapes selected project Vite: ${candidate} -> ${viteEntry}`);
  }

  const { loadConfigFromFile } = await import(pathToFileURL(viteEntry).href);
  const temporary = await mkdtemp(join(tmpdir(), "vize-config-module-"));
  try {
    const adapter = join(temporary, "config.mjs");
    // Vite evaluates the adapter's object, leaving the actual default export
    // and its source symbol intact for the existing native config projection.
    // A literal filesystem path also enters Vite 7's esbuild graph; its
    // dependency externalizer treats a file: URL as an external module.
    await writeFile(
      adapter,
      `import * as original from ${JSON.stringify(configModuleImportPath(configURL))};\nexport default { namespace: original };\n`,
      { flag: "wx", mode: 0o600 },
    );
    const loaded = await loadConfigFromFile(
      { mode: "development", command: "serve" },
      adapter,
      moduleConfigDirectory(configPath),
      "silent",
      undefined,
      "bundle",
    );
    if (!loaded) throw new Error(`Vite did not load the selected config: ${configPath}`);
    return loaded.config.namespace;
  } finally {
    await rm(temporary, { recursive: true, force: true });
  }
}
