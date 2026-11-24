import { withoutVerbatimPrefix } from "./path-identity.ts";

interface WatchServer {
  watcher: { add(file: string): unknown };
}

// Transform calls share one filesystem watcher, including across environments.
// A restarted server has a new watcher and receives its own registrations.
const serverWatches = new WeakMap<WatchServer["watcher"], Set<string>>();

/**
 * Vite records a transform's addWatchFile calls as module imports. Compiler
 * inputs include absent config candidates and type-only files, so watch them
 * through the dev server's filesystem watcher. Build hooks keep Rollup watches.
 * Each file is watched once per dev watcher, in its plain spelling. Build
 * hooks retain per-module registration so every module owns its watch inputs.
 */
export function createDependencyWatch(
  server: WatchServer | null,
  addWatchFile: ((file: string) => void) | undefined,
): ((file: string) => void) | undefined {
  const watch = server
    ? (file: string) => {
        server.watcher.add(file);
      }
    : addWatchFile;
  if (!watch) return undefined;
  let watched: Set<string> | undefined;
  return (file) => {
    if (!watched) {
      watched = server ? serverWatches.get(server.watcher) : undefined;
      if (!watched) {
        watched = new Set<string>();
        if (server) serverWatches.set(server.watcher, watched);
      }
    }
    const plain = withoutVerbatimPrefix(file);
    if (watched.has(plain)) return;
    watched.add(plain);
    watch(plain);
  };
}
