"use strict";

// These are the existing native Content Mapper protocol bits, not Canon's
// different internal ProjectionFeatures bits. Keep the protocol tuple intact.
const LanguageFeature = Object.freeze({
  Hover: 1 << 0,
  SignatureHelp: 1 << 1,
  Completion: 1 << 2,
  Definition: 1 << 3,
  TypeDefinition: 1 << 4,
  Implementation: 1 << 5,
  References: 1 << 6,
  DocumentHighlights: 1 << 7,
  Rename: 1 << 8,
  CallHierarchy: 1 << 9,
  CodeActions: 1 << 10,
  Formatting: 1 << 11,
  InlayHints: 1 << 12,
  SemanticTokens: 1 << 13,
  FoldingRanges: 1 << 14,
  SelectionRanges: 1 << 15,
  LinkedEditing: 1 << 16,
  AutoInsert: 1 << 17,
  DocumentSymbols: 1 << 18,
  CodeLens: 1 << 19,
});

function utf8Boundaries(text) {
  const boundaries = new Map([[0, 0]]);
  let bytes = 0;
  let units = 0;
  for (const scalar of text) {
    bytes += Buffer.byteLength(scalar, "utf8");
    units += scalar.length;
    boundaries.set(bytes, units);
  }
  return (offset) => {
    if (!Number.isSafeInteger(offset) || !boundaries.has(offset)) {
      throw new RangeError(`Native projection offset ${offset} is not a UTF-8 boundary`);
    }
    return boundaries.get(offset);
  };
}

/** Convert both sides of a native projection; never reinterpret UTF-8 as UTF-16. */
function toUtf16LanguageProjection(source, rawTransformJSON) {
  const raw = JSON.parse(rawTransformJSON);
  if (typeof raw.text !== "string" || raw.extension !== ".tsx" || !Array.isArray(raw.mappings)) {
    throw new TypeError("Invalid native language projection");
  }
  const original = utf8Boundaries(source);
  const generated = utf8Boundaries(raw.text);
  const range = (convert, start, length) => {
    if (!Number.isSafeInteger(length) || length < 0)
      throw new RangeError("Invalid projection length");
    return [convert(start), convert(start + length) - convert(start)];
  };
  const mappings = raw.mappings.map((tuple) => {
    if (!Array.isArray(tuple) || tuple.length !== 6) throw new TypeError("Invalid projection span");
    const [start, length, sourceStart, sourceLength, kind, features] = tuple;
    if (
      ![0, 1, 2].includes(kind) ||
      !Number.isInteger(features) ||
      features < 0 ||
      features > 0xfffff
    ) {
      throw new TypeError("Invalid projection span metadata");
    }
    return [
      ...range(generated, start, length),
      ...range(original, sourceStart, sourceLength),
      kind,
      features,
    ];
  });
  const diagnostics = raw.diagnostics.map((diagnostic) => {
    const [start, length] = range(original, diagnostic.start, diagnostic.length);
    return { ...diagnostic, start, length };
  });
  const semanticLinks = (raw.semanticLinks ?? []).map((link) => {
    const [sourceStart, sourceLength] = range(generated, link.sourceStart, link.sourceLength);
    const [targetStart, targetLength] = range(generated, link.targetStart, link.targetLength);
    return { ...link, sourceStart, sourceLength, targetStart, targetLength };
  });
  const diagnosticDirectives = raw.diagnosticDirectives && {
    ...raw.diagnosticDirectives,
    directives: raw.diagnosticDirectives.directives.map((tuple) => {
      if (!Array.isArray(tuple) || tuple.length !== 6)
        throw new TypeError("Invalid diagnostic directive");
      const [start, length, virtualStart, virtualEnd, policy, index] = tuple;
      if (![0, 1].includes(policy)) throw new TypeError("Invalid diagnostic directive policy");
      return [
        ...range(original, start, length),
        generated(virtualStart),
        generated(virtualEnd),
        policy,
        index,
      ];
    }),
  };
  return { ...raw, mappings, diagnostics, semanticLinks, diagnosticDirectives };
}

/** Apply native suppression/owner-copy intervals to one complete diagnostic collection. */
function applyDiagnosticDirectives(projection, generatedDiagnostics) {
  const directives = projection.diagnosticDirectives?.directives ?? [];
  const used = new Set();
  const diagnostics = generatedDiagnostics.filter((diagnostic) => {
    if (diagnostic.start === undefined) return true;
    let suppressed = false;
    for (let index = 0; index < directives.length; index++) {
      const [, , start, end, policy] = directives[index];
      if (start <= diagnostic.start && diagnostic.start < end) {
        suppressed = true;
        if (policy === 1) used.add(index);
      }
    }
    return !suppressed;
  });
  const sourceDiagnostics = [...projection.diagnostics];
  directives.forEach(([start, length, , , policy, tableIndex], index) => {
    if (policy !== 1 || used.has(index)) return;
    const diagnostic = projection.diagnosticDirectives.unusedExpectDirectiveDiagnostics[tableIndex];
    if (!diagnostic) throw new TypeError("Missing unused expectation diagnostic");
    sourceDiagnostics.push({ ...diagnostic, start, length });
  });
  return { diagnostics, sourceDiagnostics };
}

module.exports = { LanguageFeature, toUtf16LanguageProjection, applyDiagnosticDirectives };
