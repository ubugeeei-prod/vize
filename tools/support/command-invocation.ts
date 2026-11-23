import fs from "node:fs";
import path from "node:path";

// Windows npm/cmd-shims need cmd.exe; pass escaped arguments rather than
// shell:true, which joins paths containing spaces or metacharacters verbatim.
export function windowsInvocation(command: string, args: readonly string[]) {
  if (process.platform !== "win32") return { command, args: [...args], verbatim: false };
  let executable = command;
  if (!path.extname(executable)) {
    const directories =
      command.includes("/") || command.includes("\\")
        ? [""]
        : (process.env.PATH ?? "").split(path.delimiter);
    const extensions = (process.env.PATHEXT ?? ".COM;.EXE;.BAT;.CMD").split(";");
    outer: for (const directory of directories)
      for (const extension of extensions) {
        const candidate = path.join(directory, command + extension.toLowerCase());
        if (fs.statSync(candidate, { throwIfNoEntry: false })?.isFile()) {
          executable = candidate;
          break outer;
        }
      }
  }
  if (!/\.(cmd|bat)$/i.test(executable))
    return { command: executable, args: [...args], verbatim: false };
  const doubleEscape = /node_modules[\\/]\.bin[\\/]/i.test(executable);
  const escaped = args.map((arg) => escapeWindowsArgument(arg, doubleEscape));
  const line = [path.normalize(executable).replace(WINDOWS_META, "^$1"), ...escaped].join(" ");
  return {
    command: process.env.ComSpec ?? "cmd.exe",
    args: ["/d", "/s", "/c", `"${line}"`],
    verbatim: true,
  };
}
/*! Argument escaping adapted from cross-spawn (MIT), Copyright (c) 2018 Made With
 * MOXY Lda <hello@moxy.studio>. Permission is hereby granted, free of charge, to
 * any person obtaining a copy of this software and associated documentation
 * files (the "Software"), to deal in the Software without restriction, including
 * without limitation the rights to use, copy, modify, merge, publish, distribute,
 * sublicense, and/or sell copies of the Software, and to permit persons to whom
 * the Software is furnished to do so, subject to the following conditions:
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software. THE SOFTWARE IS PROVIDED
 * "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT
 * LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
 * PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
 * HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
 * ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
 * WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */
const WINDOWS_META = /([()\][%!^"`<>&|;, *?])/g;
function escapeWindowsArgument(value: string, twice: boolean): string {
  const quoted = `"${value.replace(/(?=(\\+?)?)\1"/g, '$1$1\\"').replace(/(?=(\\+?)?)\1$/, "$1$1")}"`;
  const escaped = quoted.replace(WINDOWS_META, "^$1");
  return twice ? escaped.replace(WINDOWS_META, "^$1") : escaped;
}
