// JSX/TSX entry cases of the compiler corpus. Each case pins one `App.jsx`/`App.tsx`
// source and the complete product facets (`code`, `errors`, `warnings`, `scopedStyles`)
// of every registered target. `crates/vize_atelier_jsx/tests/compiler_corpus.rs` compiles
// the pinned entry with the source-built `compile_jsx` and compares the facets byte for byte.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export const JSX_ENTRY_ADAPTER = "compiler-jsx-entry-source-v1";
export const JSX_ENTRY_RUNNER = "crates/vize_atelier_jsx/tests/compiler_corpus.rs";
/** Targets in canonical order: client VDOM, client Vapor, and their SSR output. */
export const JSX_ENTRY_TARGETS = ["vdom", "vapor", "vdom-ssr", "vapor-ssr"] as const;
export const JSX_ENTRY_COMPAT = ["native", "babel"] as const;
const ENTRIES = ["App.jsx.txt", "App.tsx.txt"];
const FIXTURES = "tests/_fixtures/differential/compiler/";
const FACETS = ["target", "code", "errors", "warnings", "scopedStyles"];

type Read = (path: string) => Buffer | null;

const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

function closed(value: unknown, keys: readonly string[], label: string) {
  assert(value !== null && typeof value === "object" && !Array.isArray(value), label);
  assert.deepEqual(Object.keys(value).sort(), [...keys].sort(), `closed ${label} fields`);
  return value as Record<string, unknown>;
}

function pinned(read: Read, path: string, sha256: unknown, label: string): Buffer {
  assert(typeof sha256 === "string" && /^[a-f0-9]{64}$/.test(sha256), `${label} SHA-256`);
  const bytes = read(`${FIXTURES}${path}`);
  assert(bytes !== null, `missing pinned ${label}: ${path}`);
  assert.equal(hash(bytes), sha256, `SHA256 mismatch: changed pinned ${label}: ${path}`);
  return bytes;
}

/** JSX compiler sources whose fixes a new JSX entry case registers. */
export function jsxEntrySource(path: string): boolean {
  return path.startsWith("crates/vize_atelier_jsx/src/");
}

/** Validates one case, its pinned entry and its complete per-target reference. */
export function validateJsxEntryCase(value: unknown, read: Read): string {
  const row = closed(
    value,
    ["id", "state", "adapter", "compat", "targets", "inputs", "reference"],
    "JSX entry case",
  );
  const id = row.id;
  assert(typeof id === "string" && /^compiler\/jsx\/[a-z0-9]+(?:-[a-z0-9]+)*$/.test(id), "JSX ID");
  const root = `jsx-${id.slice("compiler/jsx/".length)}`;
  assert.equal(row.state, "active", id);
  assert.equal(row.adapter, JSX_ENTRY_ADAPTER, id);
  assert((JSX_ENTRY_COMPAT as readonly unknown[]).includes(row.compat), `${id}: compat`);
  const targets = row.targets;
  assert(Array.isArray(targets) && targets.length > 0, `${id}: targets`);
  assert.deepEqual(
    targets,
    JSX_ENTRY_TARGETS.filter((target) => targets.includes(target)),
    `${id}: targets are distinct registered targets in canonical order`,
  );
  const inputs = closed(row.inputs, ["root", "files"], "JSX entry inputs");
  assert.equal(inputs.root, root, `${id}: input root`);
  assert(Array.isArray(inputs.files) && inputs.files.length === 1, `${id}: one entry file`);
  const entry = closed(inputs.files[0], ["path", "role", "sha256"], "JSX entry file");
  const path = entry.path;
  assert(
    typeof path === "string" && ENTRIES.includes(path),
    `${id}: entry is App.jsx.txt or App.tsx.txt`,
  );
  assert.equal(entry.role, "entry", id);
  pinned(read, `${root}/${path}`, entry.sha256, "JSX entry");
  const reference = closed(row.reference, ["path", "sha256", "source"], "JSX entry reference");
  assert.equal(reference.path, `${root}/modules.expected.json`, `${id}: reference path`);
  assert(typeof reference.source === "string" && reference.source.length > 0, `${id}: source`);
  const modules = JSON.parse(
    pinned(read, reference.path, reference.sha256, "JSX reference").toString("utf8"),
  );
  assert(Array.isArray(modules), `${id}: reference is a target list`);
  assert.deepEqual(
    modules.map((module: unknown) => closed(module, FACETS, "JSX module facets").target),
    targets,
    `${id}: one reference module per target`,
  );
  return id;
}

/** Validates every registered JSX entry case and returns their IDs. */
export function loadJsxEntryCases(value: unknown, read: Read): string[] {
  assert(Array.isArray(value), "jsxEntryCases must be an array");
  const ids = value.map((row) => validateJsxEntryCase(row, read));
  assert.equal(new Set(ids).size, ids.length, "duplicate JSX entry case");
  return ids;
}

/** New JSX entry cases of a change; registered cases stay planned. */
export function newJsxEntryCases(previous: unknown, current: unknown, read: Read): string[] {
  if (current === undefined) {
    assert.equal(previous, undefined, "drops the JSX entry cases");
    return [];
  }
  const ids = loadJsxEntryCases(current, read);
  const before = previous === undefined ? [] : (previous as { id?: unknown }[]);
  assert(Array.isArray(before), "previous jsxEntryCases must be an array");
  const kept = new Set(ids);
  const old = new Set(before.map((row) => row?.id));
  for (const id of old)
    assert(kept.has(id as string), `drops planned JSX entry case: ${String(id)}`);
  return ids.filter((id) => !old.has(id));
}
