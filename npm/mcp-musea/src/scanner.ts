import fs from "node:fs";
import path from "node:path";

export async function findArtFiles(
  root: string,
  include: string[],
  exclude: string[],
): Promise<string[]> {
  const files: string[] = [];

  async function scan(dir: string): Promise<void> {
    const entries = await fs.promises.readdir(dir, { withFileTypes: true });

    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      const relative = normalizePath(path.relative(root, fullPath));

      let excluded = false;
      for (const pattern of exclude) {
        if (matchGlob(relative, pattern) || matchGlob(entry.name, pattern)) {
          excluded = true;
          break;
        }
      }

      if (excluded) continue;

      if (entry.isDirectory()) {
        await scan(fullPath);
      } else if (entry.isFile() && entry.name.endsWith(".art.vue")) {
        for (const pattern of include) {
          if (matchGlob(relative, pattern)) {
            files.push(fullPath);
            break;
          }
        }
      }
    }
  }

  await scan(root);
  return files;
}

function normalizePath(value: string): string {
  return value.replaceAll(path.sep, "/");
}

function matchGlob(filepath: string, pattern: string): boolean {
  return globToRegExp(pattern).test(normalizePath(filepath));
}

function globToRegExp(pattern: string): RegExp {
  const normalized = normalizePath(pattern);
  if (normalized.endsWith("/**")) {
    return new RegExp(`^${globSource(normalized.slice(0, -3))}(?:/.*)?$`);
  }
  return new RegExp(`^${globSource(normalized)}$`);
}

function globSource(pattern: string): string {
  let source = "";

  for (let index = 0; index < pattern.length; ) {
    const char = pattern[index];
    const next = pattern[index + 1];
    const afterNext = pattern[index + 2];

    if (char === "*" && next === "*" && afterNext === "/") {
      source += "(?:.*/)?";
      index += 3;
      continue;
    }
    if (char === "*" && next === "*") {
      source += ".*";
      index += 2;
      continue;
    }
    if (char === "*") {
      source += "[^/]*";
      index += 1;
      continue;
    }
    if (char === "?") {
      source += "[^/]";
      index += 1;
      continue;
    }

    source += escapeRegExp(char);
    index += 1;
  }

  return source;
}

function escapeRegExp(value: string): string {
  return value.replace(/[\\^$.*+?()[\]{}|]/g, "\\$&");
}
