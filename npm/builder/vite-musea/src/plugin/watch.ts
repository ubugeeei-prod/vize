import type { ViteDevServer } from "vite";

type DevWatcher = Pick<ViteDevServer["watcher"], "add">;

export function createMuseaWatchTargets(files: readonly string[]): string[] {
  return [...new Set(files)];
}

export function watchMuseaArtFiles(watcher: DevWatcher, files: readonly string[]): void {
  const targets = createMuseaWatchTargets(files);
  if (targets.length === 0) {
    return;
  }

  watcher.add(targets);
}
