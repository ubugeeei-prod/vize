// Embedded by the native loader. Use the selected project's installed Vite
// loader when available, without evaluating its default config factory twice.
import { existsSync } from "node:fs";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname as moduleConfigDirectory, join } from "node:path";
import { pathToFileURL } from "node:url";

export async function importConfigModule(configPath) {
  const configURL = pathToFileURL(configPath);
  const require = createRequire(configURL);
  const searchPaths = require.resolve.paths("vite") ?? [];
  const hasVite = searchPaths.some((directory) => existsSync(join(directory, "vite")));
  // A dependency-free config retains its native Node module-loading contract.
  // A present but broken Vite installation, or any config error, is an error.
  if (!hasVite) return import(configURL.href);

  const { loadConfigFromFile } = await import(pathToFileURL(require.resolve("vite")).href);
  const temporary = await mkdtemp(join(tmpdir(), "vize-config-module-"));
  try {
    const adapter = join(temporary, "config.mjs");
    // Vite evaluates the adapter's object, leaving the actual default export
    // and its source symbol intact for the existing native config projection.
    // A literal filesystem path also enters Vite 7's esbuild graph; its
    // dependency externalizer treats a file: URL as an external module.
    await writeFile(
      adapter,
      `import * as original from ${JSON.stringify(configPath)};\nexport default { namespace: original };\n`,
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
