import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { normalizePath, type ResolvedConfig } from "vite";

const moduleDir = path.dirname(fileURLToPath(import.meta.url));
const globPath = (entry: string): string => normalizePath(entry).replace(/([*?{}[\]()!])/g, "\\$1");

export async function registerSourceGalleryDependencies(config: ResolvedConfig): Promise<void> {
  const client = config.environments.client;
  const optimizer = client.optimizeDeps;
  const gallery = path.join(moduleDir, "../gallery/index.html");
  if (
    config.command !== "serve" ||
    optimizer.disabled === true ||
    optimizer.disabled === "dev" ||
    optimizer.noDiscovery ||
    existsSync(path.join(moduleDir, "gallery/index.html")) ||
    !existsSync(gallery)
  ) {
    return;
  }

  // Middleware serves this source graph outside the application's HTML scan.
  // Scan its real entry so bare import IDs, aliases and lazy editor imports
  // join the first generation under the same IDs used by browser requests.
  let entries = optimizer.entries;
  if (!entries) {
    const input = client.build.rollupOptions.input;
    if (input) {
      const inputs =
        typeof input === "string" ? [input] : Array.isArray(input) ? input : Object.values(input);
      const resolve = config.createResolver();
      entries = await Promise.all(
        inputs.map(async (entry) => {
          const resolved = await resolve(entry, path.join(process.cwd(), "*"));
          if (!resolved)
            throw new Error(`Failed to resolve gallery dependency scan input: ${entry}`);
          return globPath(resolved);
        }),
      );
    } else {
      // Explicit entries replace inference, including its default exclusions.
      entries = ["**/*.html", "!**/__tests__/**", "!**/coverage/**"];
    }
  }
  optimizer.entries = [
    ...new Set([...(typeof entries === "string" ? [entries] : entries), globPath(gallery)]),
  ];
}
