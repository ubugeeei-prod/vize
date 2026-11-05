interface WatchServer {
  watcher: { add(file: string): unknown };
}

/**
 * Vite records a transform's addWatchFile calls as module imports. Compiler
 * inputs include absent config candidates and type-only files, so watch them
 * through the dev server's filesystem watcher. Build hooks keep Rollup watches.
 */
export function createDependencyWatch(
  server: WatchServer | null,
  addWatchFile: ((file: string) => void) | undefined,
): ((file: string) => void) | undefined {
  return server
    ? (file) => {
        server.watcher.add(file);
      }
    : addWatchFile;
}
