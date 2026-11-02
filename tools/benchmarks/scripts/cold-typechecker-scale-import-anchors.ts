/** Exact inverse for one frozen corpus's redundant Child import anchors; no output filtering. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import path from "node:path";
import { strictDebugParts } from "./cold-typechecker-scale-strict-pair.ts";

export interface AnchorModuleTransition {
  file: string;
  before: string;
  after: string;
}
type Packet = { status: number | null; stdout: Buffer; stderr: Buffer };
const hash = (body: string | Buffer) => createHash("sha256").update(body).digest("hex");
const anchorHeader = "\n  // Reference setup bindings (used in template/CSS v-bind)\n  ";
const query = "void (null as unknown as keyof typeof Child);";

/** This observer recognizes only the existing corpus's direct Child import, never UI.Child => UI. */
export function removeOwnedChildAnchor(code: string, source: string) {
  const imported = /^import Child from "\.\/(?:components\/)?Component[0-9]+\.vue";$/mu.test(
    source,
  );
  if (!imported) return { code, removed: 0 };
  const reads = "\n  // Mark used components as referenced\n  void Child;\n";
  if (!code.includes(reads)) return { code, removed: 0 };
  assert.equal(code.indexOf(reads), code.lastIndexOf(reads), "ambiguous owned value-read block");
  const offset = code.indexOf(anchorHeader);
  if (offset < 0) return { code, removed: 0 };
  assert.equal(offset, code.lastIndexOf(anchorHeader), "ambiguous owned binding-anchor block");
  assert(offset > code.indexOf(reads));
  const start = offset + anchorHeader.length,
    end = code.indexOf("\n", start);
  assert(end >= start);
  const line = code.slice(start, end),
    statements = line.split("; ");
  assert(
    statements.every((part) => /^void .+/u.test(part)),
    "unexpected anchor emission",
  );
  const tokens = statements.map((part, index) =>
    index === statements.length - 1 ? part : part + ";",
  );
  const count = tokens.filter((part) => part === query).length;
  assert(count <= 1, "duplicate same-name binding anchor");
  if (count === 0) return { code, removed: 0 };
  const remaining = tokens.filter((part) => part !== query);
  const replacement = remaining.length ? anchorHeader + remaining.join(" ") + "\n" : "";
  return { code: code.slice(0, offset) + replacement + code.slice(end + 1), removed: 1 };
}

/** Reconstruct every after JSON/debug byte. Unknown fields, helper bytes, order and suffix stay bound. */
export function verifyImportAnchorTransition(
  before: Packet,
  after: Packet,
  cwd: string,
  sources: { path: string; body: string }[],
) {
  assert.equal(before.status, 0);
  assert.equal(after.status, before.status);
  assert(path.isAbsolute(cwd));
  // runner/resolve.rs::display_path strips cwd only for actual descendants; siblings stay absolute.
  const source = new Map(
    sources.map((row) => {
      assert(
        path.isAbsolute(row.path) && path.normalize(row.path) === row.path,
        "exact absolute census path required",
      );
      const displayed = row.path.startsWith(cwd + path.sep)
        ? row.path.slice(cwd.length + 1)
        : row.path;
      return [displayed, row.body];
    }),
  );
  assert.equal(source.size, sources.length, "duplicate authored source paths");
  const old = JSON.parse(before.stdout.toString("utf8"));
  const current = JSON.parse(after.stdout.toString("utf8"));
  const left = strictDebugParts(before.stderr, old, cwd),
    right = strictDebugParts(after.stderr, current, cwd);
  assert(left.header.equals(right.header));
  assert(left.helper.equals(right.helper), "whole helper asset changed");
  const expected = structuredClone(old),
    modules: AnchorModuleTransition[] = [];
  let expectedJson = before.stdout.toString("utf8"),
    removedAnchors = 0;
  for (const file of expected.files) {
    if (typeof file.virtualTs !== "string") continue;
    const body = source.get(file.file);
    assert.equal(typeof body, "string", "generated module has no exact authored input");
    const next = removeOwnedChildAnchor(file.virtualTs, body!);
    if (!next.removed) continue;
    const token = JSON.stringify(file.virtualTs);
    assert.equal(
      expectedJson.indexOf(token),
      expectedJson.lastIndexOf(token),
      "ambiguous whole JSON module token",
    );
    assert(expectedJson.includes(token));
    expectedJson = expectedJson.replace(token, JSON.stringify(next.code));
    modules.push({ file: file.file, before: file.virtualTs, after: next.code });
    file.virtualTs = next.code;
    removedAnchors += next.removed;
  }
  assert(
    Buffer.from(expectedJson).equals(after.stdout),
    "unclassified whole JSON/code/diagnostic/opaque-byte change",
  );
  assert.deepEqual(current, expected, "whole packet fields changed");
  const suffix = Buffer.from(
    expected.files
      .filter((file: { virtualTs?: string }) => typeof file.virtualTs === "string")
      .toSorted((a: { file: string }, b: { file: string }) =>
        Buffer.compare(
          Buffer.from(path.resolve(cwd, a.file)),
          Buffer.from(path.resolve(cwd, b.file)),
        ),
      )
      .map(
        (file: { file: string; virtualTs: string }) =>
          `\n=== ${path.resolve(cwd, file.file)} ===\n${file.virtualTs}\n`,
      )
      .join(""),
  );
  assert(
    Buffer.concat([left.header, left.helper, suffix]).equals(after.stderr),
    "unclassified full generated debug byte change",
  );
  return { removedAnchors, affectedModules: modules.length, modules };
}

export function anchorTransitionMetadata(value: ReturnType<typeof verifyImportAnchorTransition>) {
  return {
    removedAnchors: value.removedAnchors,
    affectedModules: value.affectedModules,
    eligible: value.removedAnchors > 0,
    modules: value.modules.map((row) => ({
      file: row.file,
      before: { bytes: Buffer.byteLength(row.before), sha256: hash(row.before) },
      after: { bytes: Buffer.byteLength(row.after), sha256: hash(row.after) },
    })),
    scope:
      "exact source-bound Child value-read/import-anchor deletion only; complete raw JSON and generated debug inverse, no checker-work estimate or diagnostic normalization",
  };
}

/** Whole native input seals: only exact observed generated bodies and recreated mtimes may differ. */
export function verifyAnchorMaterializedRows(
  before: unknown,
  after: unknown,
  root: string,
  modules: AnchorModuleTransition[],
) {
  assert(path.isAbsolute(root));
  const rows = (value: unknown) => {
    assert(value && typeof value === "object");
    const seal = value as {
      roots: string[];
      rows: Record<string, unknown>[];
      sha256: string;
      files: number;
      bytes: number;
    };
    assert.deepEqual(Object.keys(seal).sort(), ["bytes", "files", "roots", "rows", "sha256"]);
    assert.equal(hash(JSON.stringify(seal.rows)), seal.sha256);
    let bytes = 0,
      files = 0;
    const seen = new Set<string>();
    const result = seal.rows.map((row) => {
      assert.equal(typeof row.path, "string");
      const file = row.path as string;
      assert(path.isAbsolute(file) && path.normalize(file) === file);
      assert(seal.roots.some((base) => file === base || file.startsWith(base + path.sep)));
      assert(!seen.has(file));
      seen.add(file);
      if (row.kind !== "file") return row;
      assert(Number.isSafeInteger(row.bytes) && Number(row.bytes) >= 0);
      assert.match(String(row.sha256), /^[a-f0-9]{64}$/u);
      files++;
      bytes += Number(row.bytes);
      if (!file.startsWith(root + path.sep)) return row;
      const { mtimeNs, ...rest } = row;
      assert.equal(typeof mtimeNs, "string");
      return rest;
    });
    assert.equal(files, seal.files);
    assert.equal(bytes, seal.bytes);
    return { roots: seal.roots, rows: result };
  };
  const left = rows(before),
    right = rows(after);
  assert.deepEqual(right.roots, left.roots);
  const expected = left.rows.map((row) => ({ ...row })),
    changed = [];
  for (const module of modules) {
    assert(module.file.startsWith("src/") && module.file.endsWith(".vue"));
    const matches = expected.flatMap((row, index) =>
      row.kind === "file" &&
      String(row.path).startsWith(root + path.sep) &&
      String(row.path).endsWith(path.sep + module.file + ".ts")
        ? [index]
        : [],
    );
    assert.equal(matches.length, 1, "each changed module needs one exact native target");
    const row = expected[matches[0]];
    assert.equal(row.bytes, Buffer.byteLength(module.before));
    assert.equal(row.sha256, hash(module.before), "whole debug to materialized body join");
    row.bytes = Buffer.byteLength(module.after);
    row.sha256 = hash(module.after);
    changed.push(row.path);
  }
  assert.deepEqual(
    right.rows,
    expected,
    "unclassified native input/source/provider/config/path/body change",
  );
  return {
    changedModules: changed,
    rows: expected.length,
    scope:
      "all whole native input rows; only exact source-derived module deletion and recreated file mtime differ; native loaded-file membership unobserved",
  };
}
