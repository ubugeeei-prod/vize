import { withoutVerbatimPrefix } from "./path-identity.ts";

interface WatchServer {
  watcher: { add(file: string): unknown };
  /** Vite's watcher already covers every file under its root. */
  config?: { root?: string };
}

// Transform calls share one filesystem watcher, including across environments.
// A restarted server has a new watcher and receives its own registrations.
const serverWatches = new WeakMap<WatchServer["watcher"], Set<string>>();

/**
 * Which registrations a build's watcher reads, so a transform registers a
 * compiler input as often as the rebuild needs it and no more often.
 *
 * - `false`: the build does not watch. Nothing reads a registration.
 * - `"module"`: a rebuild re-transforms only the modules whose own watched
 *   files changed (Rollup's watch cache, Rolldown's incremental build), so each
 *   module registers every input it depends on.
 * - an object: a rebuild re-transforms every module and watches the union of
 *   the build's registrations (Rolldown's full rebuild), so each path is
 *   registered once per build of that scope.
 */
export type BuildWatchScope = false | "module" | object;

export interface BuildWatchContext {
  meta?: { watchMode?: boolean; rolldownVersion?: string };
  environment?: object & {
    config?: { build?: { rolldownOptions?: { experimental?: { incrementalBuild?: boolean } } } };
  };
}

const buildWatches = new WeakMap<object, Set<string>>();

export function buildWatchScope(context: BuildWatchContext): BuildWatchScope {
  if (!context.meta?.watchMode) return false;
  const environment = context.environment;
  if (!context.meta.rolldownVersion || !environment) return "module";
  const incremental = environment.config?.build?.rolldownOptions?.experimental?.incrementalBuild;
  return incremental ? "module" : environment;
}

/** Each build collects its own watch files; a rebuild must register them again. */
export function resetBuildWatches(scope: object | undefined): void {
  if (scope) buildWatches.delete(scope);
}

/** The watch of a build that does not watch: compiles still validate, nothing registers. */
export const UNWATCHED_DEPENDENCY = (_file: string): void => {};

/**
 * Vite records a transform's addWatchFile calls as module imports. Compiler
 * inputs include absent config candidates and type-only files, so watch them
 * through the dev server's filesystem watcher, once per watcher, and only
 * outside its root, as Vite's own `ensureWatchedFile` does. Build hooks
 * register through the host as their `BuildWatchScope` asks. Every path is
 * registered in its plain spelling.
 */
export function createDependencyWatch(
  server: WatchServer | null,
  addWatchFile: ((file: string) => void) | undefined,
  buildScope: BuildWatchScope = "module",
): ((file: string) => void) | undefined {
  if (!server && buildScope === false) return UNWATCHED_DEPENDENCY;
  const watch = server
    ? (file: string) => {
        server.watcher.add(file);
      }
    : addWatchFile;
  if (!watch) return undefined;
  const root = server?.config?.root ? watchRootPrefix(server.config.root) : undefined;
  let watched: Set<string> | undefined;
  return (file) => {
    if (!watched) {
      watched = sharedWatches(server, buildScope);
    }
    const plain = withoutVerbatimPrefix(file);
    if (watched.has(plain)) return;
    watched.add(plain);
    if (root && plain.replace(/\\/g, "/").startsWith(root)) return;
    watch(plain);
  };
}

function sharedWatches(server: WatchServer | null, buildScope: BuildWatchScope): Set<string> {
  const owner = server ? server.watcher : typeof buildScope === "object" ? buildScope : undefined;
  if (!owner) return new Set();
  let watched = server ? serverWatches.get(server.watcher) : buildWatches.get(owner);
  if (!watched) {
    watched = new Set();
    if (server) serverWatches.set(server.watcher, watched);
    else buildWatches.set(owner, watched);
  }
  return watched;
}

function watchRootPrefix(root: string): string {
  const normalized = withoutVerbatimPrefix(root).replace(/\\/g, "/");
  return normalized.endsWith("/") ? normalized : `${normalized}/`;
}
