import assert from "node:assert/strict";

/** `@babel/parser`'s `parse`, read from the caller's installed Vue compiler. */
export type ParseModule = (code: string, options: Record<string, unknown>) => BabelFile;

type BabelToken = { start: number; end: number; value?: unknown; type: { label: string } };
type BabelNode = { type: string; start: number; end: number; [key: string]: unknown };
type BabelFile = { program: { body: BabelNode[] }; tokens: BabelToken[] };

type ErasedPacket = {
  code: string;
  map?: unknown;
  moduleShape?: Record<string, unknown> | null;
  [key: string]: unknown;
};

/** Whether an SFC source's script is authored in TypeScript. */
export function isTypeScriptSource(source: string): boolean {
  return /<script\b[^>]*\slang=(["'])ts\1/.test(source);
}

// Advance one preserved packet of a TypeScript-source SFC compiled to
// JavaScript by the reviewed per-piece erasure (#11698): the module is no
// longer reprinted whole, so its bytes change while it keeps every preserved
// program node. The reference owns code, the default-export offsets and the
// map. The native HMR facts are reconstructed independently from the erased
// module; every other field must stay byte-for-byte equal to the preserved
// producer.
export function typescriptErasureReference<T extends ErasedPacket>(
  preserved: T,
  erased: ErasedPacket,
  parse: ParseModule,
  assertComparableProgram?: (erasedProgram: unknown, preservedProgram: unknown) => void,
): T {
  const out = structuredClone(preserved);
  const parsed = parseModule(erased.code, parse);
  const erasedProgram = comparable(parsed.program);
  const preservedProgram = comparable(parseModule(preserved.code, parse).program);
  if (assertComparableProgram) {
    assertComparableProgram(erasedProgram, preservedProgram);
  } else {
    assert.deepEqual(
      erasedProgram,
      preservedProgram,
      "the erased module keeps the preserved program",
    );
  }
  out.code = erased.code;
  if (preserved.moduleShape) {
    out.moduleShape = completeModuleShape(
      preserved.moduleShape,
      moduleFacts(parsed),
      defaultExportOffsets(erased.code, parsed),
    );
  }
  if (preserved.map != null) {
    assert.equal(typeof preserved.map, "string", "maps are serialized");
    assert.equal(typeof erased.map, "string", "a mapped packet stays mapped");
    const [erasedMap, preservedMap] = [erased.map as string, preserved.map as string];
    const { mappings: _erased, ...erasedFields } = JSON.parse(erasedMap);
    const { mappings: _preserved, ...preservedFields } = JSON.parse(preservedMap);
    assert.deepEqual(erasedFields, preservedFields);
    assert.deepEqual(
      anchors(erasedMap, erased.code, parsed),
      anchors(preservedMap, preserved.code, parseModule(preserved.code, parse)),
      "every authored position maps to the same generated token",
    );
    out.map = erased.map;
  }
  return out;
}

/** Add the native HMR facts using an independent Babel parse of packet code. */
export function moduleShapeFactsReference<T extends ErasedPacket>(
  packet: T,
  parse: ParseModule,
): T {
  const out = structuredClone(packet);
  if (out.moduleShape) {
    out.moduleShape = completeModuleShape(
      out.moduleShape,
      moduleFacts(parseModule(out.code, parse)),
    );
  }
  return out;
}

function completeModuleShape(
  shape: Record<string, unknown>,
  facts: Record<string, unknown>,
  offsets: Record<string, number> = {},
): Record<string, unknown> {
  const {
    imports: _imports,
    hasSideEffectImport: _hasSideEffectImport,
    hasReexport: _hasReexport,
    hasDynamicImport: _hasDynamicImport,
    defaultExportStart,
    defaultExportKeywordEnd,
    defaultExportEnd,
    defaultExportLocal: _defaultExportLocal,
    ...geometry
  } = shape;
  const { defaultExportLocal, ...hmrFacts } = facts;
  const positions = {
    ...(defaultExportStart === undefined ? {} : { defaultExportStart }),
    ...(defaultExportKeywordEnd === undefined ? {} : { defaultExportKeywordEnd }),
    ...(defaultExportEnd === undefined ? {} : { defaultExportEnd }),
    ...offsets,
  };
  return {
    ...geometry,
    ...hmrFacts,
    ...positions,
    ...(defaultExportLocal === undefined ? {} : { defaultExportLocal }),
  };
}

function parseModule(code: string, parse: ParseModule): BabelFile {
  return parse(code, { sourceType: "module", createParenthesizedExpressions: false, tokens: true });
}

function moduleFacts(parsed: BabelFile): Record<string, unknown> {
  const imports: { source: string; local: string; namespace: boolean }[] = [];
  let hasSideEffectImport = false;
  let hasReexport = false;
  let defaultExportLocal: string | undefined;
  for (const node of parsed.program.body) {
    if (node.type === "ImportDeclaration") {
      const source = (node.source as { value?: unknown } | undefined)?.value;
      assert(typeof source === "string");
      const specifiers = node.specifiers as BabelNode[];
      if (specifiers.length === 0) hasSideEffectImport = true;
      for (const specifier of specifiers) {
        const local = (specifier.local as { name?: unknown } | undefined)?.name;
        assert(typeof local === "string");
        imports.push({
          source,
          local,
          namespace: specifier.type === "ImportNamespaceSpecifier",
        });
      }
    }
    if (
      (node.type === "ExportNamedDeclaration" || node.type === "ExportAllDeclaration") &&
      node.source
    ) {
      hasReexport = true;
    }
    if (node.type === "ExportDefaultDeclaration") {
      const declaration = node.declaration as BabelNode;
      let name: unknown;
      if (declaration.type === "Identifier") {
        name = declaration.name;
      } else if (
        declaration.type === "FunctionDeclaration" ||
        declaration.type === "ClassDeclaration"
      ) {
        name = (declaration.id as { name?: unknown } | null | undefined)?.name;
      }
      if (name !== undefined) {
        assert(typeof name === "string");
        defaultExportLocal = name;
      }
    }
  }
  const facts: Record<string, unknown> = {
    imports,
    hasSideEffectImport,
    hasReexport,
    hasDynamicImport: containsDynamicImport(parsed.program),
  };
  if (defaultExportLocal !== undefined) facts.defaultExportLocal = defaultExportLocal;
  return facts;
}

function containsDynamicImport(value: unknown): boolean {
  if (Array.isArray(value)) return value.some(containsDynamicImport);
  if (!value || typeof value !== "object") return false;
  const node = value as Record<string, unknown>;
  if (node.type === "ImportExpression") return true;
  if (
    node.type === "CallExpression" &&
    (node.callee as Record<string, unknown> | undefined)?.type === "Import"
  ) {
    return true;
  }
  return Object.entries(node).some(
    ([key, child]) =>
      !POSITION_KEYS.has(key) && !COMMENT_KEYS.includes(key) && containsDynamicImport(child),
  );
}

const POSITION_KEYS = new Set(["start", "end", "loc", "range", "extra", "tokens", "errors"]);
const COMMENT_KEYS = ["leadingComments", "trailingComments", "innerComments"];
const ANNOTATION = /^\s*[#@]__(?:PURE|NO_SIDE_EFFECTS)__\s*$/;

/** The program without positions, spelling or comments; bundler annotations stay. */
function comparable(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(comparable);
  if (!value || typeof value !== "object") return value;
  const node = value as Record<string, unknown>;
  const out: Record<string, unknown> = {};
  for (const [key, child] of Object.entries(node)) {
    if (POSITION_KEYS.has(key) || COMMENT_KEYS.includes(key)) continue;
    // `{ a }` and `{ a: a }` are one property.
    if (key === "shorthand" && (node.type === "ObjectProperty" || node.type === "Property"))
      continue;
    out[key] = comparable(child);
  }
  const annotations = COMMENT_KEYS.flatMap((key) =>
    ((node[key] as { value: string }[] | undefined) ?? [])
      .map((comment) => comment.value.trim())
      .filter((comment) => ANNOTATION.test(comment)),
  );
  if (annotations.length > 0) out.annotations = annotations;
  return out;
}

/** UTF-8 byte offsets of the top-level `export default` statement. */
function defaultExportOffsets(code: string, parsed: BabelFile): Record<string, number> {
  const declaration = parsed.program.body.find((node) => node.type === "ExportDefaultDeclaration");
  if (!declaration) return {};
  const index = parsed.tokens.findIndex((token) => token.start === declaration.start);
  const keyword = parsed.tokens[index + 1];
  assert.equal(code.slice(parsed.tokens[index].start, parsed.tokens[index].end), "export");
  assert.equal(code.slice(keyword.start, keyword.end), "default");
  const bytes = (offset: number) => Buffer.byteLength(code.slice(0, offset));
  return {
    defaultExportStart: bytes(declaration.start),
    defaultExportKeywordEnd: bytes(keyword.end),
    defaultExportEnd: bytes(declaration.end),
  };
}

function vlq(segment: string): number[] {
  const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  const fields: number[] = [];
  let value = 0,
    shift = 0;
  for (const character of segment) {
    const digit = alphabet.indexOf(character);
    assert(digit >= 0);
    value += (digit & 31) * 2 ** shift;
    if (digit & 32) shift += 5;
    else {
      fields.push(value & 1 ? -(value >> 1) : value >> 1);
      value = 0;
      shift = 0;
    }
  }
  assert.equal(shift, 0);
  return fields;
}

/** Each mapping as its authored position, name and the generated token it marks. */
function anchors(map: string, code: string, parsed: BabelFile) {
  const { mappings, names } = JSON.parse(map) as { mappings: string; names: string[] };
  const lineStarts = [0];
  for (const match of code.matchAll(/\r\n|[\n\r\u2028\u2029]/g)) {
    lineStarts.push(match.index + match[0].length);
  }
  const tokens = new Map(parsed.tokens.map((token) => [token.start, token]));
  const result: unknown[] = [];
  let sourceIndex = 0,
    sourceLine = 0,
    sourceColumn = 0,
    nameIndex = 0;
  mappings.split(";").forEach((line, generatedLine) => {
    let column = 0;
    for (const segment of line.split(",").filter(Boolean)) {
      const fields = vlq(segment);
      column += fields[0];
      if (fields.length === 1) continue;
      assert(fields.length === 4 || fields.length === 5);
      sourceIndex += fields[1];
      sourceLine += fields[2];
      sourceColumn += fields[3];
      let name = null;
      if (fields.length === 5) {
        nameIndex += fields[4];
        name = names[nameIndex];
      }
      const offset = lineStarts[generatedLine] + column;
      const token = tokens.get(offset);
      result.push({
        authored: [sourceIndex, sourceLine, sourceColumn],
        name,
        generated: token ? [token.type.label, token.value] : code.slice(offset, offset + 1),
      });
    }
  });
  return result;
}

type Segment = { column: number; authored?: [number, number, number]; name?: number };

/** Every generated line's segments, with absolute positions. */
function decodeMappings(mappings: string): Segment[][] {
  let sourceIndex = 0,
    sourceLine = 0,
    sourceColumn = 0,
    nameIndex = 0;
  return mappings.split(";").map((line) => {
    let column = 0;
    return line
      .split(",")
      .filter(Boolean)
      .map((text) => {
        const fields = vlq(text);
        column += fields[0];
        if (fields.length === 1) return { column };
        assert(fields.length === 4 || fields.length === 5);
        sourceIndex += fields[1];
        sourceLine += fields[2];
        sourceColumn += fields[3];
        const segment: Segment = { column, authored: [sourceIndex, sourceLine, sourceColumn] };
        if (fields.length === 5) {
          nameIndex += fields[4];
          segment.name = nameIndex;
        }
        return segment;
      });
  });
}

function encodeMappings(lines: Segment[][]): string {
  const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  const field = (value: number) => {
    let rest = value < 0 ? -value * 2 + 1 : value * 2;
    let out = "";
    do {
      const digit = rest % 32;
      rest = Math.floor(rest / 32);
      out += alphabet[rest > 0 ? digit + 32 : digit];
    } while (rest > 0);
    return out;
  };
  let sourceIndex = 0,
    sourceLine = 0,
    sourceColumn = 0,
    nameIndex = 0;
  const encoded = lines.map((line) => {
    let column = 0;
    return line
      .map((segment) => {
        let out = field(segment.column - column);
        column = segment.column;
        if (!segment.authored) return out;
        const [index, authoredLine, authoredColumn] = segment.authored;
        out += field(index - sourceIndex);
        out += field(authoredLine - sourceLine);
        out += field(authoredColumn - sourceColumn);
        [sourceIndex, sourceLine, sourceColumn] = segment.authored;
        if (segment.name !== undefined) {
          out += field(segment.name - nameIndex);
          nameIndex = segment.name;
        }
        return out;
      })
      .join(",");
  });
  return encoded.join(";").replace(/;+$/, "");
}

/** Whether a segment is authored inside the SFC's `<template>` block. */
function templateSegments(source: string): (segment: Segment) => boolean {
  const start = source.search(/<template\b/i);
  const end = [...source.matchAll(/<\/template\s*>/gi)].at(-1)?.index ?? -1;
  const lineStarts = [0];
  for (const match of source.matchAll(/\r\n|[\n\r]/g)) {
    lineStarts.push(match.index + match[0].length);
  }
  return ({ authored }) => {
    if (!authored || authored[0] !== 0 || start < 0 || end <= start) return false;
    const offset = lineStarts[authored[1]] + authored[2];
    return offset > start && offset <= end;
  };
}

/**
 * The candidate packet as its map was before the render mapped back to the
 * template: without the segments authored inside `<template>`, and
 * without a map when none remain. The references above describe this view.
 */
export function withoutTemplateSegments<T extends ErasedPacket>(current: T, source: string): T {
  if (current.map == null) return current;
  assert.equal(typeof current.map, "string", "maps are serialized");
  const map = JSON.parse(current.map as string) as { mappings: string };
  const inTemplate = templateSegments(source);
  const all = decodeMappings(map.mappings);
  const lines = all.map((line) => line.filter((segment) => !inTemplate(segment)));
  if (lines.every((line, index) => line.length === all[index].length)) return current;
  const out = structuredClone(current);
  if (lines.every((line) => line.length === 0)) delete out.map;
  else out.map = JSON.stringify({ ...map, mappings: encodeMappings(lines) });
  return out;
}

// Advance one preserved packet by the reviewed template provenance:
// the render of a component maps back to its template. The candidate's map is
// accepted only when it keeps every preserved segment and every other map
// field, and each segment it gains is authored inside `<template>`. A packet
// whose script emitted no segment gains its map, where the candidate lists it.
// Every other field stays the preserved producer's.
export function templateMapReference<T extends ErasedPacket>(
  preserved: T,
  current: ErasedPacket | undefined,
  source: string,
): T {
  if (current?.map == null || current.map === preserved.map) return preserved;
  assert.equal(typeof current.map, "string", "maps are serialized");
  const { mappings, ...fields } = JSON.parse(current.map as string) as {
    mappings: string;
    sourcesContent?: unknown;
  };
  let kept = "";
  if (preserved.map == null) {
    assert.deepEqual(fields.sourcesContent, [source], "a gained map describes this source");
  } else {
    assert.equal(typeof preserved.map, "string", "maps are serialized");
    const { mappings: before, ...preservedFields } = JSON.parse(preserved.map as string);
    assert.deepEqual(fields, preservedFields, "the map keeps every other field");
    kept = before;
  }
  const key = (line: number, { column, authored, name }: Segment) =>
    `${line}:${column}>${authored?.join(":") ?? ""}#${name ?? ""}`;
  const gained = new Map<string, Segment>();
  decodeMappings(mappings).forEach((line, index) => {
    for (const segment of line) gained.set(key(index, segment), segment);
  });
  decodeMappings(kept).forEach((line, index) => {
    for (const segment of line) {
      assert(gained.delete(key(index, segment)), "the map keeps every preserved segment");
    }
  });
  assert(gained.size > 0, "a changed map gains template segments");
  const inTemplate = templateSegments(source);
  for (const [position, segment] of gained) {
    assert(inTemplate(segment), `${position}: a gained segment is authored in the template`);
  }
  const out: ErasedPacket = structuredClone(preserved);
  if (preserved.map != null) {
    out.map = current.map;
    return out as T;
  }
  // The gained map takes the place the candidate packet lists it in.
  const names = Object.keys(current);
  const previous = names[names.indexOf("map") - 1];
  const placed: Record<string, unknown> = {};
  for (const [name, value] of Object.entries(out)) {
    if (name !== "map") placed[name] = value;
    if (name === previous) placed.map = current.map;
  }
  placed.map ??= current.map;
  return placed as T;
}
