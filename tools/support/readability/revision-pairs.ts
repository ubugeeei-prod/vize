// Base/head file pairs for the source-identity checkers: which files a range
// changes, with both sides' bytes. The head is a revision, or the working tree
// when no head revision is given.

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

export interface FilePair {
  /** The head path, or the base path for a deleted file. */
  readonly path: string;
  readonly basePath?: string;
  readonly before?: string;
  readonly after?: string;
}

function git(root: string, args: readonly string[], input?: string): Buffer {
  const result = spawnSync("git", args, { cwd: root, input, maxBuffer: 1 << 30 });
  if (result.status !== 0)
    throw new Error(`git ${args.join(" ")} failed: ${result.stderr?.toString() ?? ""}`);
  return result.stdout;
}

/** Read many `rev:path` blobs with one `git cat-file --batch`. */
export function readBlobs(root: string, specs: readonly string[]): (string | undefined)[] {
  if (!specs.length) return [];
  const output = git(root, ["cat-file", "--batch"], `${specs.join("\n")}\n`);
  const blobs: (string | undefined)[] = [];
  let at = 0;
  for (let index = 0; index < specs.length; index++) {
    const newline = output.indexOf(10, at);
    const header = output.subarray(at, newline).toString();
    at = newline + 1;
    const size = /^[0-9a-f]+ blob (\d+)$/u.exec(header)?.[1];
    if (size === undefined) {
      blobs.push(undefined);
      continue;
    }
    blobs.push(output.subarray(at, at + Number(size)).toString("utf8"));
    at += Number(size) + 1;
  }
  return blobs;
}

/**
 * Files changed between `base` and `head` (or the working tree), renames
 * paired, filtered by `include`. `paths` limits the diff to those pathspecs.
 */
export function changedPairs(
  root: string,
  base: string,
  head: string | undefined,
  include: (file: string) => boolean,
  paths: readonly string[] = [],
): FilePair[] {
  const range = head ? [base, head] : [base];
  const listed = git(root, ["diff", "--name-status", "-M", "-z", ...range, "--", ...paths])
    .toString()
    .split("\0");
  const entries: { status: string; basePath?: string; path: string }[] = [];
  for (let index = 0; index < listed.length - 1;) {
    const status = listed[index++];
    if (status.startsWith("R") || status.startsWith("C")) {
      const from = listed[index++];
      entries.push({ status, basePath: from, path: listed[index++] });
    } else {
      const file = listed[index++];
      entries.push({ status, basePath: status === "A" ? undefined : file, path: file });
    }
  }
  const wanted = entries.filter(
    (entry) => include(entry.path) || (entry.basePath && include(entry.basePath)),
  );
  const before = readBlobs(
    root,
    wanted.flatMap((entry) => (entry.basePath ? [`${base}:${entry.basePath}`] : [])),
  );
  const headBlobs = head
    ? readBlobs(
        root,
        wanted.flatMap((entry) => (entry.status === "D" ? [] : [`${head}:${entry.path}`])),
      )
    : [];
  let beforeAt = 0;
  let headAt = 0;
  return wanted.map((entry) => {
    let after: string | undefined;
    if (entry.status !== "D") {
      const file = path.join(root, entry.path);
      if (head) after = headBlobs[headAt++];
      else if (fs.existsSync(file)) after = fs.readFileSync(file, "utf8");
    }
    return {
      path: entry.path,
      basePath: entry.basePath,
      before: entry.basePath ? before[beforeAt++] : undefined,
      after,
    };
  });
}
