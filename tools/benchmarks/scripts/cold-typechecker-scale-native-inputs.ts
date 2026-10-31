/** Exact CLI flags, raw diagnostics and potential-input sealing for untimed observation. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
type Row = Record<string, unknown>;
const hash = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
const fileHash = (file: string) => hash(fs.readFileSync(file));
const inside = (root: string, file: string) => {
  const rel = path.relative(root, file);
  return rel === "" || (rel !== ".." && !rel.startsWith(".." + path.sep) && !path.isAbsolute(rel));
};
const utf8 = (bytes: Buffer) => new TextDecoder("utf-8", { fatal: true }).decode(bytes);

/** Finite flags from the existing Rust command; no checker-width/option substitution. */
export function validateNativeArgs(args: unknown, config: string): string[] {
  assert(Array.isArray(args) && args.every((arg) => typeof arg === "string"));
  assert.equal(args.length, 9);
  assert.deepEqual(args.slice(0, 4), ["--checkers", "1", "--incremental", "--tsBuildInfoFile"]);
  assert.deepEqual(args.slice(5), ["--pretty", "false", "--project", config]);
  assert(path.isAbsolute(args[4]));
  return args;
}

/** Preserve every line. Diagnostic roles are physical path classes, never mapper reasons. */
export function classifyRaw(bytes: Buffer, cwd: string, project: string, provider: string) {
  const parts = utf8(bytes).split("\n");
  const rows = parts.map((part, index) => {
    const hasLF = index < parts.length - 1;
    const raw = part + (hasLF ? "\n" : "");
    const line = hasLF && part.endsWith("\r") ? part.slice(0, -1) : part;
    const diagnostic = /^(.*)\((\d+),(\d+)\): (error|warning|info) TS(\d+): (.*)$/u.exec(line);
    const global = /^(error|warning|info) TS(\d+): (.*)$/u.exec(line);
    if (!diagnostic)
      return { index, raw, kind: global ? "global" : "other", code: global?.[2] ?? null };
    const file = path.resolve(cwd, diagnostic[1]);
    let physical: string | null = null;
    try {
      physical = fs.realpathSync(file);
    } catch {
      /* Explicit unresolved role below. */
    }
    return {
      index,
      raw,
      kind: "positioned",
      code: diagnostic[5],
      file,
      physical,
      role:
        physical === null
          ? "unresolved"
          : inside(provider, physical)
            ? "provider-tree"
            : inside(project, physical)
              ? "authored-tree"
              : inside(cwd, physical)
                ? "materialized"
                : "outside-sealed-roots",
      line: Number(diagnostic[2]),
      column: Number(diagnostic[3]),
    };
  });
  const groups = new Map<string, number>();
  for (const row of rows)
    if (row.code !== null) {
      const key = JSON.stringify([row.code, "role" in row ? row.role : row.kind]);
      groups.set(key, (groups.get(key) ?? 0) + 1);
    }
  return { rows, groups: [...groups].sort(([a], [b]) => a.localeCompare(b)) };
}

/** Entire physical input closure plus exact lexical symlink edges; only one output is excluded. */
export function seal(roots: string[], buildInfo: string | null) {
  const canonicalRoots = roots.map((root) => fs.realpathSync(root));
  const visited = new Set<string>(),
    rows: Row[] = [];
  const visit = (file: string) => {
    const stat = fs.lstatSync(file, { bigint: true });
    const physical = fs.realpathSync(file);
    assert(
      canonicalRoots.some((root) => inside(root, physical)),
      "input symlink escaped sealed roots",
    );
    if (stat.isSymbolicLink()) {
      rows.push({
        path: file,
        kind: "symlink",
        target: utf8(fs.readlinkSync(file, { encoding: "buffer" })),
        physical,
      });
      if (visited.has(physical)) return;
      visit(physical);
      return;
    }
    if (visited.has(physical)) return;
    visited.add(physical);
    if (stat.isDirectory()) {
      rows.push({ path: physical, kind: "directory", mode: Number(stat.mode) });
      for (const name of fs.readdirSync(physical, { encoding: "buffer" }).sort(Buffer.compare)) {
        visit(path.join(physical, utf8(name)));
      }
      return;
    }
    assert(stat.isFile(), "unmodeled input file kind");
    if (buildInfo !== null && physical === buildInfo) return;
    rows.push({
      path: physical,
      kind: "file",
      bytes: Number(stat.size),
      mtimeNs: stat.mtimeNs.toString(),
      mode: Number(stat.mode),
      sha256: fileHash(physical),
    });
  };
  for (const root of canonicalRoots) visit(root);
  rows.sort((a, b) => JSON.stringify(a).localeCompare(JSON.stringify(b)));
  return {
    roots: canonicalRoots,
    rows,
    sha256: hash(JSON.stringify(rows)),
    files: rows.filter((row) => row.kind === "file").length,
    bytes: rows
      .filter((row) => row.kind === "file")
      .reduce((sum, row) => sum + Number(row.bytes), 0),
  };
}
