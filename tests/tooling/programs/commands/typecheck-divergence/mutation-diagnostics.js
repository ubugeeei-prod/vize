import fs from "node:fs";

export function listVueFiles(root) {
  if (!fs.existsSync(root)) return [];
  const entries = [];
  for (const name of fs.readdirSync(root, { withFileTypes: true })) {
    const child = `${root}/${name.name}`;
    if (name.isDirectory()) entries.push(...listVueFiles(child));
    else if (name.isFile() && child.endsWith(".vue")) entries.push(child);
  }
  return entries;
}

export function mutationDiagnostic(source, mode, tool, file) {
  if (!source.includes("__vize_typecheck_mutation_probe: string = 1")) return null;
  if (source.includes("vize-mutation-invisible")) return null;
  if (mode === "missing") return null;
  const line = source
    .slice(0, source.indexOf("__vize_typecheck_mutation_probe"))
    .split(/\r?\n/).length;
  const reportedLine = mode === "shifted" ? line + 1 : line;
  const message =
    mode === "mismatch"
      ? "Type 'number' is not assignable to type 'boolean'."
      : "Type 'number' is not assignable to type 'string'.";
  if (tool === "vize") return `error:${reportedLine}:1 [TS2322] ${message}`;
  return `${file}(${reportedLine},1): error TS2322: ${message}\n`;
}
