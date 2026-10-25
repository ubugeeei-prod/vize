import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

export type RouterGeneratedDocument = { file: string; source: string; sha256: string };
export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };

export const experimentalRouterWorkspaceUnits = 128;

/** One fresh, collision-free child of the caller's owned temporary directory. */
export function createExperimentalRouterWorkspace(parent: string, label: string): string {
  assert.match(label, /^[a-z-]+$/u);
  const owner = fs.realpathSync(parent);
  const prefix = `${label}-雪😀-`;
  const padding =
    experimentalRouterWorkspaceUnits - owner.length - path.sep.length - prefix.length - 6;
  assert.ok(
    padding >= 0,
    "the owned temp parent must leave room for the complete fixed-length root",
  );
  const directory = fs.mkdtempSync(path.join(owner, prefix + "x".repeat(padding)));
  assert.equal(path.dirname(directory), owner);
  assert.equal(directory.length, experimentalRouterWorkspaceUnits);
  assert.equal(
    Buffer.byteLength(directory),
    experimentalRouterWorkspaceUnits + 4,
    "the fixed oracle requires exactly its one known Unicode root fragment",
  );
  assert.equal(fs.realpathSync(directory), directory);
  return directory;
}

/** Relocate only compiler-owned leading reference directives, never authored text. */
export function experimentalRouterDocumentTransport(
  source: string,
  projectRoot: string,
  relativeReferences: readonly string[] = ["typed-router.d.ts"],
) {
  assert.equal(path.isAbsolute(projectRoot), true);
  assert.notEqual(projectRoot.at(-1), path.sep);
  const spans: Array<{ start: number; end: number }> = [];
  const paths: string[] = [];
  let cursor = 0;
  while (cursor < source.length) {
    const newline = source.indexOf("\n", cursor);
    const end = newline === -1 ? source.length : newline + 1;
    const line = source.slice(cursor, end).replace(/\r?\n$/u, "");
    const reference = /^\/\/\/ <reference path=(?:"([^"]+)"|'([^']+)') \/>$/u.exec(line);
    if (reference) {
      paths.push(reference[1] ?? reference[2]);
      const start = cursor + '/// <reference path="'.length;
      spans.push({ start, end: start + projectRoot.length });
    } else if (!/^\/\/\/ <reference lib="[^"]+" \/>$/u.test(line)) {
      break;
    }
    cursor = end;
  }
  assert.deepEqual(
    paths,
    relativeReferences.map((relative) => `${projectRoot}/${relative}`),
    "the complete leading reference inventory must belong to this admitted project",
  );
  let normalizedSource = source;
  for (const span of [...spans].reverse())
    normalizedSource =
      normalizedSource.slice(0, span.start) + "$PROJECT" + normalizedSource.slice(span.end);
  const byteSpans = spans.map((span) => ({
    start: Buffer.byteLength(source.slice(0, span.start)),
    end: Buffer.byteLength(source.slice(0, span.end)),
  }));
  const sourceBytes = Buffer.byteLength(source);
  return {
    normalizedSource,
    relocate(position: number) {
      assert.ok(Number.isSafeInteger(position) && position >= 0 && position <= sourceBytes);

      assert.ok(
        byteSpans.every((span) => position <= span.start || position >= span.end),
        "a code position cannot be inside a relocated transport address",
      );
      // GoTS completion data stores UTF8 byte offsets. Authored LSP ranges remain UTF16.
      const delta = byteSpans
        .filter((span) => span.end <= position)
        .reduce((sum, span) => sum + (span.end - span.start) - Buffer.byteLength("$PROJECT"), 0);
      return position - delta;
    },
  };
}

/** Preserve all fields; only proven path fields and their source coordinates move. */
export function normalizeExperimentalRouterPackets(
  value: unknown,
  projectRoots: readonly string[],
  canonicalProjectRoot: string,
  generated?: RouterGeneratedDocument | null,
  relocateCoordinates = true,
): Json {
  const suffix = "/src/pages/products/[id=int].vue.ts";
  let virtualRoot: string | undefined;
  const transport =
    generated && relocateCoordinates
      ? experimentalRouterDocumentTransport(generated.source, canonicalProjectRoot)
      : undefined;
  if (generated) {
    assert.equal(generated.file.slice(-suffix.length), suffix);
    virtualRoot = generated.file.slice(0, -suffix.length);
  }
  const addresses = [
    ...projectRoots.map((address) => ({ address, replacement: "$PROJECT" })),
    ...projectRoots.flatMap((address) => [
      { address: pathToFileURL(address).href, replacement: "file://$PROJECT" },
      {
        address: pathToFileURL(address).href.replaceAll("'", "%27"),
        replacement: "file://$PROJECT",
      },
    ]),
    ...(virtualRoot
      ? [
          { address: virtualRoot, replacement: "$VIRTUAL" },
          { address: pathToFileURL(virtualRoot).href, replacement: "file://$VIRTUAL" },
        ]
      : []),
  ].sort((left, right) => right.address.length - left.address.length);
  const visit = (entry: unknown, field: string): Json => {
    if (typeof entry === "string") {
      if (!["uri", "requestUri", "fileName"].includes(field)) return entry;
      const mapping = addresses.find(
        ({ address }) => entry === address || entry.startsWith(`${address}/`),
      );
      return mapping ? mapping.replacement + entry.slice(mapping.address.length) : entry;
    }
    if (Array.isArray(entry)) return entry.map((item) => visit(item, ""));
    if (entry !== null && typeof entry === "object") {
      const fields = entry as Record<string, unknown>;
      const result = Object.fromEntries(
        Object.entries(fields).map(([key, item]) => [key, visit(item, key)]),
      );
      if (
        typeof fields.fileName === "string" &&
        typeof fields.name === "string" &&
        typeof fields.position === "number"
      ) {
        assert.ok(generated, "native coordinates require the actual generated document");
        assert.equal(
          fields.fileName,
          generated.file,
          "completion coordinates must name that exact document",
        );
        if (transport) result.position = transport.relocate(fields.position);
      }
      return result;
    }
    assert.ok(entry === null || typeof entry === "boolean" || typeof entry === "number");
    return entry;
  };
  return visit(value, "");
}
