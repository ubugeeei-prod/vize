import { readFileSync } from "node:fs";

type Launcher = "exec" | "exec-fixed" | "profile-lsp" | "record-pid";

export function shellLauncher(
  name: Launcher,
  values: Record<string, string | readonly string[]>,
): string {
  const template = readFileSync(new URL("./launchers/" + name + ".sh", import.meta.url), "utf8");
  return template.replace(/@@([A-Z_]+)@@/gu, (_, key: string) => {
    const value = values[key];
    if (value === undefined) throw new Error("Missing launcher argument " + key);
    const args = typeof value === "string" ? [value] : value;
    return args.map((arg) => "'" + arg.replaceAll("'", "'\\''") + "'").join(" ");
  });
}

export function windowsLauncher(command: readonly string[]): string {
  const template = readFileSync(new URL("./launchers/node.cmd", import.meta.url), "utf8");
  const args = command.map((value) => '"' + value.replaceAll("%", "%%") + '"');
  return template.replace("@@COMMAND@@", () => args.join(" ")).replace(/\r?\n/gu, "\r\n");
}

/** Preserve the interpreter and the exact configured executable bytes in capture receipts. */
export function configuredNodeLauncher(
  source: URL,
  settings: Readonly<Record<string, unknown>>,
): string {
  const program = readFileSync(source, "utf8");
  const slot = JSON.stringify("@@PROGRAM_SETTINGS@@");
  if (program.split(slot).length !== 2)
    throw new Error("Expected one settings slot in " + source.href);
  const encoded = JSON.stringify(JSON.stringify(settings));
  return "#!" + process.execPath + "\n" + program.replace(slot, () => encoded);
}
