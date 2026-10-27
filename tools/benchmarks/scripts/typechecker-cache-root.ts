/** Recover the mirror's semantic root only from actual generated primary input paths. */
import fs from "node:fs";
import path from "node:path";

const inside = (root: string, child: string) => {
  const tail = path.relative(root, child);
  return (
    tail === "" || (tail !== ".." && !tail.startsWith(".." + path.sep) && !path.isAbsolute(tail))
  );
};

export function inferCacheSemanticRoot(options: {
  projectRoot: string;
  entries: { path: string }[];
  virtualRoot: string;
  configPath: string;
  config: { files?: unknown; include?: unknown };
}) {
  // Canon uses explicit generated include paths; files is retained for other exact-root hosts.
  // Do not expand authored globs or infer membership from a file that merely exists on disk.
  const files = [options.config.files, options.config.include]
    .flatMap((list) => (Array.isArray(list) ? list : []))
    .filter((file): file is string => typeof file === "string" && !/[*?]/u.test(file))
    .map((file) => path.resolve(path.dirname(options.configPath), file));
  const sfcs = options.entries.filter((entry) => entry.path.endsWith(".vue"));
  const names = new Map<string, number>();
  for (const entry of sfcs) {
    const name = path.basename(entry.path);
    names.set(name, (names.get(name) ?? 0) + 1);
  }
  const anchors = sfcs.filter((entry) => names.get(path.basename(entry.path)) === 1);
  const listedFiles = new Set(files);
  const candidates: string[] = [];
  for (const anchor of anchors.slice(0, 8)) {
    const original = fs.realpathSync(path.join(options.projectRoot, anchor.path));
    for (const file of files.filter(
      (file) => path.basename(file) === path.basename(anchor.path) + ".ts",
    )) {
      const relative = path.relative(options.virtualRoot, file).slice(0, -3);
      if (
        !relative ||
        path.isAbsolute(relative) ||
        relative.startsWith(".." + path.sep) ||
        relative.startsWith("__vize_external__" + path.sep)
      )
        continue;
      const suffix = path.sep + relative;
      if (original.endsWith(suffix))
        candidates.push(original.slice(0, -suffix.length) || path.parse(original).root);
    }
  }
  const root = candidates[0] ?? null;
  const unambiguous = root !== null && candidates.every((candidate) => candidate === root);
  const mappings = unambiguous
    ? sfcs.map((entry) => {
        const original = fs.realpathSync(path.join(options.projectRoot, entry.path));
        const virtual = path.join(options.virtualRoot, path.relative(root, original) + ".ts");
        let exists: boolean | null = null,
          error: string | null = null;
        try {
          exists = fs.statSync(virtual).isFile();
        } catch (failure) {
          error = (failure as NodeJS.ErrnoException).code ?? String(failure);
          exists = error === "ENOENT" || error === "ENOTDIR" ? false : null;
        }
        return { original, virtual, exists, error, listed: listedFiles.has(virtual) };
      })
    : [];
  return {
    kind: "mirrored-primary-input-bijection",
    root,
    candidates,
    qualified:
      unambiguous &&
      sfcs.length > 0 &&
      mappings.every((row) => inside(root, row.original) && row.exists && row.listed),
    sourceFiles: sfcs.length,
    configFiles: files.length,
    mappings,
  };
}

export function rootOutsideOwnedNamespace(root: string, owned: string) {
  return !inside(fs.realpathSync(root), fs.realpathSync(owned));
}
