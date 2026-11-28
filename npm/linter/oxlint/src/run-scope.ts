import { createRequire } from "node:module";
import path from "node:path";

/**
 * Whether `argv` starts one `oxlint` CLI run: the `oxlint` bin is the entry
 * and it does not serve the language server. Such a process lints one fixed
 * set of files and exits, so what it reads from disk holds for its whole life.
 * Every other host, such as the language server, a test runner or a tool that
 * calls the plugin from a long-lived process, may lint the same files again
 * after they change, so it must not remember them.
 */
export function isOneShotOxlintRun(argv: readonly string[]): boolean {
  const entry = argv[1];
  return (
    entry !== undefined &&
    path.basename(entry).replace(/\.[cm]?js$/u, "") === "oxlint" &&
    !argv.includes("--lsp")
  );
}

/** Whether this process is one `oxlint` CLI run. */
export const ONE_SHOT_OXLINT_RUN = isOneShotOxlintRun(process.argv);

/**
 * Whether `specifier` resolves from the directory of a file. A one-shot run
 * remembers each directory's answer, since resolution walks the directories
 * above a file, which every file of a directory shares; any other host asks
 * again, so a package installed while it runs is found.
 */
export function packageResolution(
  specifier: string,
  remember = ONE_SHOT_OXLINT_RUN,
): (filename: string) => boolean {
  const byDirectory = remember ? new Map<string, boolean>() : undefined;
  return (filename) => {
    const directory = path.dirname(path.resolve(filename));
    const known = byDirectory?.get(directory);
    if (known !== undefined) return known;
    let resolvable: boolean;
    try {
      createRequire(path.join(directory, "index.js")).resolve(specifier);
      resolvable = true;
    } catch {
      resolvable = false;
    }
    byDirectory?.set(directory, resolvable);
    return resolvable;
  };
}
