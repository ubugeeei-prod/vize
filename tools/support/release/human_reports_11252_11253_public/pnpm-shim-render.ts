import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { posix as path } from "node:path";
import { sha256 } from "./value.ts";

const raw = readFileSync(new URL("./pnpm-shim-templates.json.txt", import.meta.url));
assert.equal(sha256(raw), "f90ee621163ab2f8550bbb9b5b7503580592e034b7e5c975287f30027164763a");
export const pnpmSource = JSON.parse(raw.toString("utf8")) as {
  schema: string;
  packageIntegrity: string;
  sourceCommit: string;
  header: string;
  platform: string;
  physicalPrelude: string;
  managerSources: {
    path: string;
    base64: string;
    bytes: number;
    sha256: string;
  }[];
};
assert.equal(pnpmSource.sourceCommit, "c7b46aedef7b81c07816cb551187449a3dd81792");
export interface UnixShimInputs {
  target: string;
  shimPath: string;
  runtime: "node" | null;
  nodePath: readonly string[];
  relocatableRoot: string | null;
}
export function absoluteUnix(value: string): string {
  assert.equal(typeof value, "string");
  assert(value.startsWith("/") && path.normalize(value) === value);
  assert(!/[\\\0\r\n"$`:]/u.test(value), "unsupported unsafe Unix source path");
  return value;
}
export function within(root: string, candidate: string): boolean {
  return candidate === root || candidate.startsWith(root + "/");
}
const relative = (from: string, to: string) => path.relative(from, to) || ".";

/** Exact c7b46a Unix branches used by a Node shebang without extra arguments,
 * or a target without an interpreter. Other runtimes/options remain refused. */
export function renderPnpm12161Unix(input: UnixShimInputs): Buffer {
  absoluteUnix(input.target);
  absoluteUnix(input.shimPath);
  assert(input.runtime === "node" || input.runtime === null);
  const directory = path.dirname(input.shimPath),
    target = relative(directory, input.target);
  const root = input.relocatableRoot;
  if (root !== null) {
    absoluteUnix(root);
    assert(within(root, directory) && within(root, input.target));
  }
  let body = pnpmSource.header + pnpmSource.physicalPrelude + pnpmSource.platform;
  const nodePath = input.nodePath
    .map((entry) => {
      absoluteUnix(entry);
      return root !== null && within(root, entry)
        ? "$basedir_abs/" + relative(directory, entry)
        : entry;
    })
    .join(":");
  if (nodePath)
    body += `if [ -z "$NODE_PATH" ]; then\n  export NODE_PATH="${nodePath}"\nelse\n  export NODE_PATH="${nodePath}:$NODE_PATH"\nfi\n`;
  const quoted = `"$basedir_abs/${target}"`,
    windows = `"$basedir_win/${target}"`;
  if (input.runtime === "node") {
    body += `if [ -n "$exe" ] && [ -x "$basedir/node.exe" ]; then\n  exec "$basedir/node.exe"  ${windows} "$@"\nelif [ -x "$basedir/node" ]; then\n  exec "$basedir/node"  ${quoted} "$@"\nelif [ -n "$msys" ] && command -v node >/dev/null 2>&1; then\n  exec node  ${windows} "$@"\nelif command -v node >/dev/null 2>&1; then\n  exec node  ${quoted} "$@"\nelif [ -n "$exe" ] && command -v node.exe >/dev/null 2>&1; then\n  exec node.exe  ${windows} "$@"\nelse\n  exec node  ${quoted} "$@"\nfi\n`;
  } else body += `exec ${quoted}  "$@"\nexit $?\n`;
  body += "# cmd-shim-target=" + (root === null ? input.target : target) + "\n";
  return Buffer.from(body);
}

/** c7b46a executable.rs:84 and linking_paths.rs:148. The options below must
 * first be authenticated by the separate source-owned input record. */
export function derivedNodePath(owner: string, extra: readonly string[]): string[] {
  absoluteUnix(owner);
  extra.forEach(absoluteUnix);
  if (extra.length === 0) return [];
  let cursor = owner;
  while (
    path.basename(cursor) !== "node_modules" ||
    path.basename(path.dirname(cursor)) === "node_modules"
  ) {
    const parent = path.dirname(cursor);
    if (parent === cursor) return [...new Set(extra)];
    cursor = parent;
  }
  const below = path.relative(cursor, owner).split("/"),
    first = below[0];
  assert(first);
  const packageDirectory = first.startsWith("@")
    ? path.join(cursor, first, below[1]!)
    : path.join(cursor, first);
  return [...new Set([path.join(packageDirectory, "node_modules"), cursor, ...extra])];
}
