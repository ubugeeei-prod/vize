import fs from "node:fs";
import path from "node:path";

/**
 * Names the action when `vize init` is pointed at a directory inside a package.
 *
 * The directory itself has no `package.json`, so init must not write files or
 * select features there. The nearest parent package is the place to run it.
 * Returns null when this directory is a package root, or when no parent is.
 */
export function packageSubdirectoryMessage(root: string): string | null {
  const resolved = path.resolve(root);
  if (fs.existsSync(path.join(resolved, "package.json"))) return null;

  const stop = path.parse(resolved).root;
  let current = resolved;
  while (current !== stop) {
    const parent = path.dirname(current);
    if (parent === current) break;
    if (fs.existsSync(path.join(parent, "package.json"))) {
      return (
        `No package.json in ${resolved}. ` +
        `Run \`vize init\` from ${parent} instead of a package subdirectory. ` +
        "Type checking was not configured."
      );
    }
    current = parent;
  }
  return null;
}
