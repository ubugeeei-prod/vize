import fs from "node:fs";
import path from "node:path";

export function copyTree(source: string, destination: string) {
  if (fs.lstatSync(source).isSymbolicLink())
    throw new Error("fixture copies forbid symbolic links: " + source);
  const from = fs.realpathSync.native(source);
  let ancestor = path.resolve(destination);
  const missing: string[] = [];
  while (!fs.existsSync(ancestor)) {
    missing.unshift(path.basename(ancestor));
    ancestor = path.dirname(ancestor);
  }
  const to = path.join(fs.realpathSync.native(ancestor), ...missing);
  const contains = (parent: string, child: string) => {
    const relative = path.relative(parent, child);
    return (
      relative === "" ||
      (!path.isAbsolute(relative) && relative !== ".." && !relative.startsWith(".." + path.sep))
    );
  };
  if (contains(from, to) || contains(to, from))
    throw new Error("fixture copy paths must be disjoint");
  function copyEntry(input: string, output: string) {
    const stat = fs.lstatSync(input);
    if (stat.isSymbolicLink()) throw new Error("fixture copies forbid symbolic links: " + input);
    if (stat.isDirectory()) {
      fs.mkdirSync(output, { recursive: true });
      for (const name of fs.readdirSync(input).sort())
        copyEntry(path.join(input, name), path.join(output, name));
    } else if (stat.isFile()) {
      fs.mkdirSync(path.dirname(output), { recursive: true });
      fs.copyFileSync(input, output);
    } else {
      throw new Error("unsupported fixture file type: " + input);
    }
    fs.chmodSync(output, stat.mode & 0o777);
  }
  copyEntry(from, to);
}
