import { withoutVerbatimPrefix } from "./path-identity.ts";

interface WatchServer {
  watcher: { add(file: string): unknown };
}

/**
 * Vite records a transform's addWatchFile calls as module imports. Compiler
 * inputs include absent config candidates and type-only files, so watch them
 * through the dev server's filesystem watcher. Build hooks keep Rollup watches.
 * Each file is watched once, in its plain spelling.
 */
export function createDependencyWatch(
  server: WatchServer | null,
  addWatchFile: ((file: string) => void) | undefined,
): ((file: string) => void) | undefined {
  const watch = server ? (file: string) => void server.watcher.add(file) : addWatchFile;
  if (!watch) return undefined;
  const watched = new Set<string>();
  return (file) => {
    const plain = withoutVerbatimPrefix(file);
    if (watched.has(plain)) return;
    watched.add(plain);
    watch(plain);
  };
}
