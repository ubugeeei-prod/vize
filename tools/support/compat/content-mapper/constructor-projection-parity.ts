/** Prove the exact helper relocation while retaining every complete packet. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { isDeepStrictEqual } from "node:util";
import { withoutProducerUnmappedIgnores } from "./producer-unmapped-ignores.ts";
import {
  certifiedTransition,
  uniqueOffset,
  previousRuntimeProp,
  currentRuntimeProp,
} from "./canonical-helper-transition.ts";

import {
  readAssets,
  previousNativeAttributeAliases,
  currentNativeAttributeAliases,
} from "./constructor-helper-assets.ts";
export {
  readAssets,
  canonicalPaths,
  nativeAttributesPath,
  previousNativeAttributeAliases,
  currentNativeAttributeAliases,
} from "./constructor-helper-assets.ts";
import {
  integer,
  bounds,
  point,
  range,
  rangeIncludingInsertions,
  bufferOccurrences,
  comments,
  type Edit,
} from "./projection-byte-relocation.ts";
import { certifyEmitOverloadInventory } from "./owned-emit-overload-transition.ts";
import {
  certifyComponentConstructorTransition,
  certifyGenericComponentContextTransition,
} from "./component-constructor-transition.ts";
import { certifyComponentExportTransition } from "./component-export-transition.ts";
import { writeConstructorProjectionReceipt } from "./constructor-projection-receipt.ts";
const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");

type Link = {
  sourceStart: number;
  sourceLength: number;
  targetStart: number;
  targetLength: number;
};
type Packet = {
  text: string;
  mappings: number[][];
  semanticLinks?: Link[];
  diagnosticDirectives?: { directives: number[][] };
  diagnostics: { start: number; length: number }[];
};
export type Row = {
  case: string;
  newline: string;
  source: string;
  target: string;
  options?: number;
  packet: Packet;
};
export type Assets = ReturnType<typeof readAssets>;

export const previousSlotsExport = "export type Slots = $Slots;";
export const currentSlotsExport = "export type { $Slots as Slots };";
export const previousSlotsMarker = "Partial<Slots>";
export const currentSlotsMarker = "Partial<$Slots>";

function relocationEdits(
  before: Buffer,
  after: Buffer,
  sourceBytes: number,
  baseGeneratedRanges: [number, number][],
  headGeneratedRanges: [number, number][],
  assets: Assets,
  replacement: Buffer,
) {
  // Shared packets may already contain both the embedded body and its imports.
  if (!assets.embedded.equals(assets.headEmbedded) && after.includes(assets.headEmbedded))
    replacement = assets.headEmbedded;
  const helperCandidates = replacement.equals(assets.headEmbedded)
    ? [assets.embedded]
    : [assets.embedded, assets.baseImported];
  const previousHelpers = helperCandidates.filter(
    (candidate, index, candidates) =>
      !candidate.equals(replacement) &&
      candidates.findIndex((other) => other.equals(candidate)) === index,
  );
  const previousCounts = previousHelpers.map((candidate) => ({
    candidate,
    before: bufferOccurrences(before, candidate),
    after: bufferOccurrences(after, candidate),
  }));
  const helperCounts = {
    beforePrevious: previousCounts.reduce((total, count) => total + count.before, 0),
    beforeCurrent: bufferOccurrences(before, replacement),
    afterPrevious: previousCounts.reduce((total, count) => total + count.after, 0),
    afterCurrent: bufferOccurrences(after, replacement),
  };
  const helperAbsent = Object.values(helperCounts).every((count) => count === 0);
  const helperPrevious =
    helperCounts.beforePrevious === 1 &&
    helperCounts.beforeCurrent === 0 &&
    helperCounts.afterPrevious === 1 &&
    helperCounts.afterCurrent === 0;
  const helperCurrent =
    helperCounts.beforePrevious === 0 &&
    helperCounts.beforeCurrent === 1 &&
    helperCounts.afterPrevious === 0 &&
    helperCounts.afterCurrent === 1;
  const helperTransition =
    helperCounts.beforePrevious === 1 &&
    helperCounts.beforeCurrent === 0 &&
    helperCounts.afterPrevious === 0 &&
    helperCounts.afterCurrent === 1;
  assert.ok(
    helperAbsent || helperPrevious || helperCurrent || helperTransition,
    `helper inventory changed: ${JSON.stringify(helperCounts)}`,
  );
  const start = helperAbsent
    ? null
    : helperCurrent
      ? uniqueOffset(before, replacement)
      : uniqueOffset(
          before,
          previousCounts.find((count) => count.before === 1)?.candidate ?? assets.embedded,
        );
  const certifiedEdits: (Edit & { label: string })[] = [];
  if (start !== null && helperTransition)
    certifiedEdits.push({
      start,
      end:
        start +
        (previousCounts.find((count) => count.before === 1)?.candidate.length ??
          assets.embedded.length),
      replacement,
      label: "helper prefix",
    });
  if (!assets.baseNativeAttributes.equals(assets.headNativeAttributes)) {
    const transition = certifiedTransition(
      before,
      after,
      previousNativeAttributeAliases,
      currentNativeAttributeAliases,
      "native attribute alias",
    );
    if (transition) certifiedEdits.push(transition);
  }
  // Whole helper replacement owns its alias; pruned helpers retain only the declaration.
  if (!helperTransition) {
    const runtimeProp = certifiedTransition(
      before,
      after,
      previousRuntimeProp,
      currentRuntimeProp,
      "runtime prop value alias",
    );
    if (runtimeProp) certifiedEdits.push(runtimeProp);
  }
  for (const transition of [
    certifiedTransition(before, after, previousSlotsExport, currentSlotsExport, "Slots export"),
    certifiedTransition(before, after, previousSlotsMarker, currentSlotsMarker, "Slots marker"),
  ])
    if (transition) certifiedEdits.push(transition);
  const emitTransition = certifyEmitOverloadInventory(before, after, assets.emitOverloads);
  if (emitTransition) certifiedEdits.push(emitTransition);
  const componentConstructorTransition = certifyComponentConstructorTransition(before, after);
  if (componentConstructorTransition) certifiedEdits.push(componentConstructorTransition);
  const genericComponentContextTransition = certifyGenericComponentContextTransition(before, after);
  if (genericComponentContextTransition) certifiedEdits.push(genericComponentContextTransition);
  const componentExportTransition = certifyComponentExportTransition(before, after);
  if (componentExportTransition) certifiedEdits.push(...componentExportTransition.edits);
  const originalComments = comments(before.toString(), baseGeneratedRanges);
  const actualComments = comments(after.toString(), headGeneratedRanges);
  assert.equal(
    actualComments.length,
    originalComments.length,
    "generated map-comment inventory changed",
  );
  const events = [
    ...certifiedEdits.map((edit) => ({ kind: "replacement" as const, start: edit.start, edit })),
    ...originalComments.map((original, index) => ({
      kind: "comment" as const,
      start: original.start,
      original,
      actual: actualComments[index],
    })),
  ].sort((left, right) => left.start - right.start);
  const edits: Edit[] = [];
  for (const event of events) {
    if (event.kind === "replacement") {
      const { edit } = event;
      assert.ok((edits.at(-1)?.end ?? 0) <= edit.start, "certified replacements must be disjoint");
      assert.equal(
        uniqueOffset(after, edit.replacement),
        point(edit.start, edits),
        `${edit.label} boundary moved`,
      );
      edits.push(edit);
      continue;
    }
    const { actual, original } = event;
    const [g0, g1, s0, s1] = original.values;
    bounds(g0, g1 - g0, before.length);
    bounds(s0, s1 - s0, sourceBytes);
    assert.ok(g1 <= original.start, "a map comment must describe preceding code");
    const values = [point(g0, edits), point(g1, edits), s0, s1];
    assert.deepEqual(
      actual.values,
      values,
      "self-described coordinates must obey the same relocation",
    );
    assert.equal(actual.start, point(original.start, edits));
    const replacementText = `${original.indent}// @vize-map: ${values[0]}:${values[1]} -> ${s0}:${s1}${original.text.endsWith("\r") ? "\r" : ""}`;
    assert.equal(actual.text, replacementText, "only derived coordinate digits may change");
    edits.push({
      start: original.start,
      end: original.start + Buffer.byteLength(original.text),
      replacement: Buffer.from(replacementText),
    });
  }
  let cursor = 0;
  const fragments: Buffer[] = [];
  for (const edit of edits) {
    assert.ok(cursor <= edit.start, "certified edits must be ordered and disjoint");
    fragments.push(before.subarray(cursor, edit.start), edit.replacement);
    cursor = edit.end;
  }
  fragments.push(before.subarray(cursor));
  assert.deepEqual(
    after,
    Buffer.concat(fragments),
    "all other complete generated bytes must remain exact",
  );
  return { start, edits, componentExportTransition };
}
function validateSource(packet: Packet, sourceBytes: number) {
  const generatedBytes = Buffer.byteLength(packet.text);
  for (const tuple of packet.mappings) {
    assert.equal(tuple.length, 6);
    tuple.forEach(integer);
    bounds(tuple[0], tuple[1], generatedBytes);
    bounds(tuple[2], tuple[3], sourceBytes);
  }
  for (const diagnostic of packet.diagnostics)
    bounds(diagnostic.start, diagnostic.length, sourceBytes);
  for (const tuple of packet.diagnosticDirectives?.directives ?? []) {
    assert.equal(tuple.length, 6);
    tuple.forEach(integer);
    bounds(tuple[0], tuple[1], sourceBytes);
    bounds(tuple[2], tuple[3] - tuple[2], generatedBytes);
  }
  for (const link of packet.semanticLinks ?? []) {
    bounds(link.sourceStart, link.sourceLength, generatedBytes);
    bounds(link.targetStart, link.targetLength, generatedBytes);
  }
}

export function relocatePacket(
  base: Packet,
  head: Packet,
  source: string,
  assets: Assets,
  replacement = assets.imported,
) {
  base = withoutProducerUnmappedIgnores(base);
  head = withoutProducerUnmappedIgnores(head, {
    requireComplete: assets.canonical[0].baseSha256 !== assets.canonical[0].headSha256,
  });
  const sourceBytes = Buffer.byteLength(source);
  validateSource(base, sourceBytes);
  validateSource(head, sourceBytes);
  const before = Buffer.from(base.text),
    after = Buffer.from(head.text);
  const { start, edits, componentExportTransition } = relocationEdits(
    before,
    after,
    sourceBytes,
    base.mappings.map(([generated, length]) => [generated, length]),
    head.mappings.map(([generated, length]) => [generated, length]),
    assets,
    replacement,
  );
  const expected = structuredClone(base);
  expected.text = after.toString();
  expected.mappings = base.mappings.map(([g, length, ...rest]) => [
    ...range(g, length, edits, before.length),
    ...rest,
  ]);
  if (componentExportTransition?.addedMapping) {
    assert.ok(
      (expected.mappings.at(-1)?.[0] ?? 0) <= componentExportTransition.addedMapping[0],
      "component export mapping must retain generated order",
    );
    expected.mappings.push(componentExportTransition.addedMapping);
  }
  if (expected.semanticLinks)
    expected.semanticLinks = expected.semanticLinks.map((link) => {
      const [sourceStart, sourceLength] = range(
        link.sourceStart,
        link.sourceLength,
        edits,
        before.length,
      );
      const [targetStart, targetLength] = range(
        link.targetStart,
        link.targetLength,
        edits,
        before.length,
      );
      return { ...link, sourceStart, sourceLength, targetStart, targetLength };
    });
  if (expected.diagnosticDirectives)
    expected.diagnosticDirectives.directives = expected.diagnosticDirectives.directives.map(
      (tuple) => {
        const [g, length] = rangeIncludingInsertions(
          tuple[2],
          tuple[3] - tuple[2],
          edits,
          before.length,
        );
        return [tuple[0], tuple[1], g, g + length, ...tuple.slice(4)];
      },
    );
  assert.deepEqual(head, expected, "the complete packet must equal only the certified relocation");
  const proof = {
    helperStart: start,
    edits: edits.map((edit) => ({
      start: edit.start,
      end: edit.end,
      replacementSha256: digest(edit.replacement),
      replacementBytes: edit.replacement.length,
    })),
  };
  return componentExportTransition?.addedMapping
    ? { ...proof, componentExportMapping: componentExportTransition.addedMapping }
    : proof;
}

type VirtualPacket = {
  virtual_ts: string;
  virtual_ts_mappings: {
    authored: { start: number; end: number };
    generated: { start: number; end: number };
    subSpans: {
      authored: { start: number; end: number };
      generated: { start: number; end: number };
    }[];
  }[];
  virtual_ts_semantic_links: {
    source: { start: number; end: number };
    target: { start: number; end: number };
  }[];
};

export function relocateVirtualPacket(
  base: VirtualPacket,
  head: VirtualPacket,
  source: string,
  assets: Assets,
  replacement: Buffer,
) {
  const sourceBytes = Buffer.byteLength(source);
  const before = Buffer.from(base.virtual_ts),
    after = Buffer.from(head.virtual_ts);
  const collectGeneratedRanges = (packet: VirtualPacket, generatedBytes: number) => {
    const ranges: [number, number][] = [];
    for (const mapping of packet.virtual_ts_mappings) {
      bounds(mapping.authored.start, mapping.authored.end - mapping.authored.start, sourceBytes);
      bounds(
        mapping.generated.start,
        mapping.generated.end - mapping.generated.start,
        generatedBytes,
      );
      ranges.push([mapping.generated.start, mapping.generated.end - mapping.generated.start]);
      for (const span of mapping.subSpans) {
        bounds(span.authored.start, span.authored.end - span.authored.start, sourceBytes);
        bounds(span.generated.start, span.generated.end - span.generated.start, generatedBytes);
        ranges.push([span.generated.start, span.generated.end - span.generated.start]);
      }
    }
    for (const link of packet.virtual_ts_semantic_links) {
      bounds(link.source.start, link.source.end - link.source.start, generatedBytes);
      bounds(link.target.start, link.target.end - link.target.start, generatedBytes);
    }
    return ranges;
  };
  const baseGeneratedRanges = collectGeneratedRanges(base, before.length);
  const headGeneratedRanges = collectGeneratedRanges(head, after.length);
  const { start, edits, componentExportTransition } = relocationEdits(
    before,
    after,
    sourceBytes,
    baseGeneratedRanges,
    headGeneratedRanges,
    assets,
    replacement,
  );
  assert.equal(
    componentExportTransition,
    undefined,
    "virtual packets cannot acquire content-mapper export rows",
  );
  const shift = ({ start, end }: { start: number; end: number }) => {
    const [relocated, length] = range(start, end - start, edits, before.length);
    return { start: relocated, end: relocated + length };
  };
  const expected = structuredClone(base);
  expected.virtual_ts = head.virtual_ts;
  expected.virtual_ts_mappings = base.virtual_ts_mappings.map((mapping) => ({
    ...mapping,
    generated: shift(mapping.generated),
    subSpans: mapping.subSpans.map((span) => ({ ...span, generated: shift(span.generated) })),
  }));
  expected.virtual_ts_semantic_links = base.virtual_ts_semantic_links.map((link) => ({
    ...link,
    source: shift(link.source),
    target: shift(link.target),
  }));
  assert.deepEqual(
    head,
    expected,
    "the complete virtual packet must equal only the certified relocation",
  );
  return {
    helperStart: start,
    edits: edits.map((edit) => ({
      start: edit.start,
      end: edit.end,
      replacementSha256: digest(edit.replacement),
      replacementBytes: edit.replacement.length,
    })),
  };
}

export function comparePackets(base: Row[], head: Row[], assets: Assets) {
  assert.equal(base.length, 228);
  assert.equal(head.length, 228);
  const identity = (row: Row) => JSON.stringify([row.case, row.newline, row.target, row.options]);
  assert.equal(new Set(base.map(identity)).size, 228);
  assert.equal(new Set(head.map(identity)).size, 228);
  const relocations = [];
  let unchanged = 0;
  let shared = 0;
  let ownedEmitPackets = 0;
  for (const [index, before] of base.entries()) {
    const after = head[index];
    assert.deepEqual({ ...before, packet: null }, { ...after, packet: null });
    const usesSharedHelpers =
      before.target === "content-mapper" && ((before.options ?? 0) & 8) !== 0;
    if (usesSharedHelpers) shared++;
    if ("text" in after.packet)
      withoutProducerUnmappedIgnores(after.packet, {
        requireComplete: assets.canonical[0].baseSha256 !== assets.canonical[0].headSha256,
      });
    const generated = (packet: Packet) => {
      const text = "text" in packet ? packet.text : (packet as unknown as VirtualPacket).virtual_ts;
      return typeof text === "string" ? Buffer.from(text) : undefined;
    };
    const ownedEmit = certifyEmitOverloadInventory(
      generated(before.packet),
      generated(after.packet),
      assets.emitOverloads,
    );
    if (ownedEmit) ownedEmitPackets++;
    if (isDeepStrictEqual(after, before)) {
      if ("text" in before.packet) validateSource(before.packet, Buffer.byteLength(before.source));
      unchanged++;
    } else {
      const replacement = usesSharedHelpers ? assets.imported : assets.headEmbedded;
      const proof =
        "text" in before.packet
          ? relocatePacket(before.packet, after.packet, before.source, assets, replacement)
          : relocateVirtualPacket(
              before.packet as unknown as VirtualPacket,
              after.packet as unknown as VirtualPacket,
              before.source,
              assets,
              replacement,
            );
      relocations.push({ identity: identity(before), ...proof });
    }
  }
  assert.equal(shared, 96);
  assert.equal(unchanged + relocations.length, 228);
  if (assets.emitOverloads)
    assert.equal(
      ownedEmitPackets,
      152,
      "the authenticated observer must retain all 152 owned emit packets",
    );
  return {
    completePackets: 228,
    unchangedPackets: unchanged,
    unsharedPackets: 132,
    sharedPackets: shared,
    relocatedPackets: relocations.length,
    ownedEmitPackets,
    canonical: assets.canonical,
    aliasOrder: assets.names,
    relocations,
  };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  writeConstructorProjectionReceipt(comparePackets, fileURLToPath(import.meta.url));
