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
