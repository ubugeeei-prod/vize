import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

// import.meta is banned in this file (Nuxt 2 webpack). A fixture can load
// dist/index.mjs without installing @vizejs/nuxt into rootDir, so the running
// dist entry is taken from the stack only after rootDir resolution fails.
const RUNNING_DIST_ENTRY =
  /((?:file:\/\/)?[^\s)]+?[/\\]dist[/\\](?:index\.mjs|nuxt2-entry\.cjs))/;

export function createNuxtModuleResolver(rootDir: string) {
  const moduleDir = resolveInstalledModuleDir(rootDir);
  return {
    resolve: (...segments: string[]) => resolve(moduleDir, ...segments),
  };
}

function resolveInstalledModuleDir(rootDir: string): string {
  try {
    return dirname(createRequire(`${rootDir}/package.json`).resolve("@vizejs/nuxt"));
  } catch (error) {
    const running = runningModuleDir();
    if (running) return running;
    throw error;
  }
}

function runningModuleDir(): string | undefined {
  const found = new Error().stack?.match(RUNNING_DIST_ENTRY)?.[1];
  if (!found) return undefined;
  const file = found.startsWith("file://") ? fileURLToPath(found) : found;
  return dirname(file);
}
