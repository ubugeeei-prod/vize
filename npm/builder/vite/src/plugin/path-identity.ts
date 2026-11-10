import fs from "node:fs";
import path from "node:path";

/**
 * Every stable filesystem identity Vite may use for one authored path.
 *
 * Vite normally resolves workspace links, while chokidar and plugins can
 * report either the logical link or its physical target. Missing watched files
 * (notably a tsconfig that is created later) still inherit the real identity
 * of their nearest existing parent.
 */
export function filesystemPathIdentities(file: string): readonly string[] {
  const resolved = path.resolve(withoutVerbatimPrefix(file));
  const identities = new Set([resolved]);
  let existing = resolved;
  const missing: string[] = [];

  for (;;) {
    try {
      const physical = fs.realpathSync.native(existing);
      identities.add(path.join(physical, ...missing));
      break;
    } catch (error) {
      const code = (error as NodeJS.ErrnoException).code;
      if (code !== "ENOENT" && code !== "ENOTDIR") break;
      const parent = path.dirname(existing);
      if (parent === existing) break;
      missing.unshift(path.basename(existing));
      existing = parent;
    }
  }

  return [...identities];
}

const DOS_DEVICES = new Set([
  "CON",
  "PRN",
  "AUX",
  "NUL",
  "CONIN$",
  "CONOUT$",
  ...["COM", "LPT"].flatMap((device) => [..."0123456789¹²³"].map((port) => device + port)),
]);

/**
 * `\\?\C:\x` → `C:\x` and `\\?\UNC\host\share\x` → `\\host\share\x` (also the
 * `//?/` spelling), when the plain spelling names the same file. chokidar
 * cannot watch verbatim paths: it walks their parents up to a bare `C:`.
 */
export function withoutVerbatimPrefix(file: string): string {
  const separator = file.startsWith("\\\\?\\") ? "\\" : file.startsWith("//?/") ? "/" : "";
  if (!separator) return file;
  const rest = file.slice(4);
  const [head = "", ...names] = rest.split(separator);
  let plain = rest;
  if (head.toUpperCase() === "UNC") {
    const [server = "", share = ""] = names.splice(0, 2);
    if (!isWin32Name(server) || !isWin32Name(share)) return file;
    plain = separator + separator + rest.slice(4);
  } else if (!/^[a-z]:$/i.test(head) || names.length === 0) {
    // A bare `C:` is drive-relative.
    return file;
  }
  return names.every((name) => name === "" || isWin32Name(name)) ? plain : file;
}

/** Verbatim paths keep `..`, trailing dots and spaces, and device names literally. */
function isWin32Name(name: string): boolean {
  if (name === "" || name.length > 255 || name.endsWith(".") || name.endsWith(" ")) return false;
  for (const character of name) {
    if (character < " " || '<>:"/\\|?*'.includes(character)) return false;
  }
  const stem = (name.split(".", 1)[0] ?? "").replace(/ +$/, "");
  return !DOS_DEVICES.has(stem.toUpperCase());
}
