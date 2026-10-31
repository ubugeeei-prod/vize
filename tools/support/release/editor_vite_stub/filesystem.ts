import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  closeSync,
  lstatSync,
  mkdirSync,
  openSync,
  readSync,
  readdirSync,
  readlinkSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { dirname, isAbsolute, join, relative, sep } from "node:path";
import { asRecord, asString } from "../../json-value.ts";

export function inside(root: string, file: string): boolean {
  const path = relative(root, file);
  return path !== "" && !isAbsolute(path) && path !== ".." && !path.startsWith(".." + sep);
}
export function digest(file: string): { bytes: number; sha256: string } {
  const hash = createHash("sha256");
  const fd = openSync(file, "r"),
    buffer = Buffer.alloc(1024 * 1024);
  let bytes = 0;
  try {
    for (;;) {
      const count = readSync(fd, buffer, 0, buffer.length, null);
      if (!count) break;
      bytes += count;
      hash.update(buffer.subarray(0, count));
    }
  } finally {
    closeSync(fd);
  }
  return { bytes, sha256: hash.digest("hex") };
}
export type Entry = {
  path: string;
  kind: "directory" | "file" | "link";
  mode: number;
  bytes?: number;
  sha256?: string;
  target?: string;
  resolved?: string | null;
};
/** lstat every entry; never traverse the reported link into another installation. */
export function inventory(root: string): Entry[] {
  assert.ok(lstatSync(root).isDirectory() && !lstatSync(root).isSymbolicLink());
  const entries: Entry[] = [];
  function visit(file: string, name: string): void {
    const stat = lstatSync(file),
      mode = stat.mode;
    if (stat.isSymbolicLink()) {
      let resolved: string | null = null;
      try {
        resolved = realpathSync(file);
      } catch {
        /* Retain dangling-link identity. */
      }
      entries.push({ path: name, kind: "link", mode, target: readlinkSync(file), resolved });
    } else if (stat.isDirectory()) {
      entries.push({ path: name, kind: "directory", mode });
      for (const item of readdirSync(file).sort())
        visit(join(file, item), name ? name + "/" + item : item);
    } else {
      assert.ok(stat.isFile(), "unexpected filesystem object: " + file);
      entries.push({ path: name, kind: "file", mode, ...digest(file) });
    }
  }
  visit(root, "");
  return entries;
}
export function writeFixture(
  root: string,
  files: unknown,
  select: (name: string) => boolean,
): void {
  for (const [name, value] of Object.entries(asRecord(files))) {
    if (!select(name)) continue;
    const file = join(root, name);
    assert.ok(inside(root, file), "fixture escaped project");
    mkdirSync(dirname(file), { recursive: true });
    writeFileSync(file, asString(value));
  }
}
