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
): T {
  const out = structuredClone(preserved);
  const parsed = parseModule(erased.code, parse);
  assert.deepEqual(
    comparable(parsed.program),
    comparable(parseModule(preserved.code, parse).program),
    "the erased module keeps the preserved program",
  );
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
      assert.equal(typeof source, "string");
      const specifiers = node.specifiers as BabelNode[];
      if (specifiers.length === 0) hasSideEffectImport = true;
      for (const specifier of specifiers) {
        const local = (specifier.local as { name?: unknown } | undefined)?.name;
        assert.equal(typeof local, "string");
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
      if (declaration.type === "Identifier") {
        const name = declaration.name;
        assert.equal(typeof name, "string");
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
