// Publishes many npm package directories concurrently. npm does not need a
// dependency to exist at publish time, so release order does not matter.
import { collectOutput, println, runMain, writeStderr, writeStdout } from "../command_io.ts";
import {
  cliArgs,
  isDir,
  joinPath,
  lexicalCompare,
  pathExists,
  readDir,
  readFileToString,
  resolvePath,
  toolBin,
} from "../command_runtime.ts";

const USAGE =
  "Usage: node tools/support/release/publish_npm_package_dirs/main.ts <dir>... [--concurrency 1..16] [publish_npm_package flags]\n       node tools/support/release/publish_npm_package_dirs/main.ts --names <dir>...\n";

/** Space-separated package names, for the post-publish verification. */
function printNames(dirs: readonly string[]): number {
  const names = dirs
    .flatMap((dir) => expandPackageDirs(resolvePath(dir)))
    .map((dir) => {
      const manifest = readFileToString(joinPath(dir, "package.json"));
      return manifest === null ? null : (JSON.parse(manifest) as { name?: string }).name;
    });
  if (names.length === 0 || names.some((name) => typeof name !== "string")) {
    writeStderr("Every package directory needs a named package.json\n");
    return 1;
  }
  println(names.join(" "));
  return 0;
}

/** A package directory, or a parent whose children are package directories. */
export function expandPackageDirs(directory: string): string[] {
  if (pathExists(joinPath(directory, "package.json"))) return [directory];
  if (!isDir(directory)) return [];
  return readDir(directory)
    .map((entry) => joinPath(directory, entry))
    .filter((child) => isDir(child) && pathExists(joinPath(child, "package.json")))
    .sort(lexicalCompare);
}

export function parseArgs(args: readonly string[]) {
  const dirs: string[] = [];
  const forward: string[] = [];
  let concurrency = 8;
  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (arg === "--concurrency") {
      const value = args[++index] ?? "";
      if (!/^([1-9]|1[0-6])$/.test(value)) return null;
      concurrency = Number(value);
    } else if (arg === "--retries" || arg === "--delay") {
      forward.push(arg, args[++index] ?? "");
    } else if (arg.startsWith("--")) {
      forward.push(arg);
    } else {
      dirs.push(arg);
    }
  }
  return dirs.length === 0 ? null : { dirs, forward, concurrency };
}

export async function publishNpmPackageDirs(args: readonly string[]): Promise<number> {
  if (args[0] === "--names") return printNames(args.slice(1));
  const parsed = parseArgs(args);
  if (parsed === null) {
    writeStderr(USAGE);
    return 1;
  }
  const packageDirs = parsed.dirs.flatMap((dir) => expandPackageDirs(resolvePath(dir)));
  const missing = parsed.dirs.filter((dir) => expandPackageDirs(resolvePath(dir)).length === 0);
  if (missing.length > 0) {
    writeStderr(`No package directories in: ${missing.join(", ")}\n`);
    return 1;
  }
  const nodeBin = toolBin("NODE_BIN", process.execPath);
  const script = resolvePath("tools/support/release/publish_npm_package/main.ts");
  const started = Date.now();
  const failed: string[] = [];
  let next = 0;
  const worker = async (): Promise<void> => {
    while (next < packageDirs.length) {
      const dir = packageDirs[next++];
      const output = await collectOutput(nodeBin, [script, dir, ...parsed.forward]).catch(
        (error: unknown) => ({ status: 1, stdout: "", stderr: `${String(error)}\n` }),
      );
      // Children are buffered and printed whole so parallel logs stay readable.
      println(`::group::${dir} (exit ${output.status})`);
      writeStdout(output.stdout);
      writeStderr(output.stderr);
      println("::endgroup::");
      if (output.status !== 0) failed.push(dir);
    }
  };
  await Promise.all(Array.from({ length: parsed.concurrency }, worker));
  println(
    `Published ${packageDirs.length - failed.length}/${packageDirs.length} packages in ${Date.now() - started} ms`,
  );
  if (failed.length > 0) {
    writeStderr(`Failed: ${failed.join(", ")}\n`);
    return 1;
  }
  return 0;
}

if (import.meta.main) {
  runMain(() => publishNpmPackageDirs(cliArgs()));
}
