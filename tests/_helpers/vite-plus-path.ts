import path from "node:path";

const VITE_PLUS_BIN = path.join(process.env.HOME ?? "", ".vite-plus", "bin");

function normalizePathListEntry(entry: string): string {
  const resolved = path.resolve(entry);
  return process.platform === "win32" ? resolved.toLowerCase() : resolved;
}

export function withVitePlusBinFallback(pathEnv = process.env.PATH ?? ""): string {
  const vitePlusBin = normalizePathListEntry(VITE_PLUS_BIN);
  const entries = pathEnv
    .split(path.delimiter)
    .filter(Boolean)
    .filter((entry) => normalizePathListEntry(entry) !== vitePlusBin);

  return [...entries, VITE_PLUS_BIN].join(path.delimiter);
}
