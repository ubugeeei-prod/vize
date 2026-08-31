import { existsSync, statSync } from "node:fs";
import { dirname, isAbsolute, relative, resolve, sep } from "node:path";

export function expectedCompilerOutputs(cwd, patterns, inputPaths) {
  // Build plans layout from existing input roots, including roots that match no files.
  const patternRoots = patterns.map((pattern) => compilerInputRoot(cwd, pattern)).filter(Boolean);
  const roots =
    patternRoots.length > 0
      ? patternRoots
      : inputPaths.map((entry) => dirname(resolve(cwd, entry)));
  const root = commonPathRoot(roots);
  const outputs = new Map(
    inputPaths.map((inputPath) => {
      const relativeInput = relative(root, resolve(cwd, inputPath));
      if (
        relativeInput === ".." ||
        relativeInput.startsWith(`..${sep}`) ||
        isAbsolute(relativeInput)
      ) {
        throw new Error(`compiler input is outside its output root: ${inputPath}`);
      }
      return [relativeInput.replaceAll("\\", "/").replace(/\.vue$/, ".json"), inputPath];
    }),
  );
  if (outputs.size !== inputPaths.length) {
    throw new Error("compiler inputs map to duplicate output paths");
  }
  return outputs;
}

function compilerInputRoot(cwd, pattern) {
  const literal = resolve(cwd, pattern);
  if (existsSync(literal)) {
    const metadata = statSync(literal);
    if (metadata.isFile()) return dirname(literal);
    if (metadata.isDirectory()) return literal;
    return null;
  }

  const normalized = pattern.replaceAll("\\", "/");
  const metacharacter = normalized.search(/[*?[]/);
  if (metacharacter < 0) return null;
  const prefix = normalized.slice(0, metacharacter);
  const separator = prefix.lastIndexOf("/");
  const root = resolve(cwd, separator < 0 ? "." : prefix.slice(0, separator));
  return existsSync(root) && statSync(root).isDirectory() ? root : null;
}

function commonPathRoot(paths) {
  let root = paths[0];
  while (
    paths.some((entry) => {
      const nested = relative(root, entry);
      return nested === ".." || nested.startsWith(`..${sep}`) || isAbsolute(nested);
    })
  ) {
    const parent = dirname(root);
    if (parent === root) throw new Error("compiler inputs have no common output root");
    root = parent;
  }
  return root;
}
