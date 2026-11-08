/** Exact current emitter scaffolding; actual packets are never normalized. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { beforeGenericEventSpecialization } from "../../../../tests/tooling/support/generic-event-specialization-source.ts";
import {
  beforeIntegratedStaticListenerSource,
  staticListenerSource,
} from "../../../../tests/tooling/support/static-listener-prop-source.ts";
import { certifyComponentConstructorTransition } from "./component-constructor-transition.ts";
import { certifyComponentExportTransition } from "./component-export-transition.ts";
import { beforeDiscriminatedJsxUnion } from "./discriminated-jsx-union-transition.ts";
import { beforeCanonLeftovers, beforeStrictExtraKeys } from "./strict-extra-keys-transition.ts";

export type CallbackSourceReader = (path: string) => string;
type Edit = { start: number; end: number; replacement: Buffer };
const emitSource = "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-inference.d.ts.txt";
const componentHelperSource = "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt";
const emitOverloadsSource =
  "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt";
const emitOverloadTransitionSource =
  "crates/vize_canon/tests/empty_vue_star_exports/membership/constructor_without_args/payload_update/emit_overload_window.rs";
const integratedSourceTransition =
  "tests/_fixtures/differential/typechecker/tsx-single-required-exact-keys/integrated-source-transition-11964.json";
const root = new URL("../../../../", import.meta.url);
const read = (path: string) => readFileSync(new URL(path, root), "utf8");
const escape = (text: string) => text.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
function template(source: string) {
  const match = source.match(/"(\{indent\}type \{inferred_args\} = (?:[^"\\]|\\.)*;\\n)"/u);
  assert.ok(match, "one certified inference template");
  return JSON.parse(`"${match[1]}"`) as string;
}
function matcher(source: string) {
  const patterns: Record<string, string> = {
    indent: "[ \\t]*",
    inferred_args: "__[A-Za-z_$][\\w$]*_inferred_emit_args",
    library: "__[A-Za-z_$][\\w$]*_library",
    emit_props: "__vize_emit_props_[0-9]+",
    prop_key: 'on[A-Za-z_$][\\w$]*|"on(?:[^"\\\\]|\\\\.)*"',
    loose_marker: "unknown\\[\\]|infer __L extends unknown\\[\\]",
    loose_args: "unknown\\[\\]|__L",
  };
  const seen = new Set<string>();
  let pattern = "^",
    offset = 0;
  for (const match of source.matchAll(/\{\{|\}\}|\{([a-z_]+)\}/gu)) {
    pattern += escape(source.slice(offset, match.index));
    if (match[0] === "{{") pattern += "\\{";
    else if (match[0] === "}}") pattern += "\\}";
    else {
      const name = match[1]!;
      assert.ok(patterns[name]);
      pattern += seen.has(name) ? `\\k<${name}>` : `(?<${name}>${patterns[name]})`;
      seen.add(name);
    }
    offset = match.index + match[0].length;
  }
  return new RegExp(pattern + escape(source.slice(offset)), "gmu");
}
function render(source: string, fields: Record<string, string>) {
  return source.replace(/\{\{|\}\}|\{([a-z_]+)\}/gu, (token, name?: string) => {
    if (token === "{{") return "{";
    if (token === "}}") return "}";
    assert.ok(fields[name!]);
    return fields[name!]!;
  });
}

/** Certify a direct emit-props call moved under its authored `v-if` guard. */
export function guardedInferenceEdits(before: string, after: string): Edit[] {
  const edits: Edit[] = [];
  const header = /^([ \t]*)const (__vize_emit_props_[0-9]+) = \(\(\) => \{\r?\n/gmu;
  for (const match of after.matchAll(header)) {
    const [opening, indent, emitProps] = match;
    const newline = opening.endsWith("\r\n") ? "\r\n" : "\n";
    const comment =
      `${indent}  // @ts-ignore Inference-only guard; authored binding checks own diagnostics.` +
      newline;
    const guardedLineStart = match.index + opening.length;
    if (!after.startsWith(comment, guardedLineStart)) continue;
    const ifStart = guardedLineStart + comment.length;
    const ifEnd = after.indexOf(newline, ifStart);
    assert.ok(ifEnd > ifStart, "guarded inference keeps one complete guard line");
    const ifLine = after.slice(ifStart, ifEnd);
    const prefix = `${indent}  if (`;
    const separator = ") return ";
    assert.ok(ifLine.startsWith(prefix), "guarded inference keeps the authored guard");
    const separatorAt = ifLine.lastIndexOf(separator);
    assert.ok(separatorAt > prefix.length, "guarded inference keeps one return call");
    const call = ifLine.slice(separatorAt + separator.length);
    assert.ok(
      call.startsWith("(undefined as unknown as __") && call.endsWith(")({"),
      "guarded inference keeps the complete resolver call",
    );
    const tail = `${indent}  return undefined as never;${newline}${indent}})();${newline}`;
    const tailStart = after.indexOf(tail, ifEnd + newline.length);
    assert.ok(tailStart > ifEnd, "guarded inference keeps its unreachable fallback");
    const blockEnd = tailStart + tail.length;
    const current = after.slice(match.index, blockEnd);
    if (before.includes(current)) continue;

    const nested = after.slice(ifEnd + newline.length, tailStart);
    const directBody = nested
      .split(newline)
      .map((line) => {
        if (!line) return line;
        assert.ok(
          line.startsWith(`${indent}  `),
          "guarded inference indents every resolver body line",
        );
        return indent + line.slice(indent.length + 2);
      })
      .join(newline);
    const direct = `${indent}const ${emitProps} = ${call}${newline}${directBody}`;
    const directAt = before.indexOf(direct);
    if (directAt < 0) continue;
    assert.equal(
      before.indexOf(direct, directAt + direct.length),
      -1,
      "one direct inference call owns the guarded transition",
    );
    assert.equal(
      after.includes(direct),
      false,
      "the direct inference call is fully replaced by its guard",
    );
    edits.push({
      start: Buffer.byteLength(before.slice(0, directAt)),
      end: Buffer.byteLength(before.slice(0, directAt + direct.length)),
      replacement: Buffer.from(current),
    });
  }
  return edits;
}

/**
 * Certify a guarded sibling reusing only declarations that are byte-identical
 * to an earlier listener contract after substituting its owner identity.
 */
function collapseGuardedContractOwnershipEdits(
  before: string,
  after: string,
  previousTemplate: string,
  currentTemplate: string,
  guardedEdits: Edit[],
): Edit[] {
  const edits: Edit[] = [];
  const byteAt = (offset: number) => Buffer.byteLength(before.slice(0, offset));
  const add = (start: number, end: number, replacement: string) =>
    edits.push({ start: byteAt(start), end: byteAt(end), replacement: Buffer.from(replacement) });
  const guardedHeader = /^([ \t]*)const (__vize_emit_props_[0-9]+) = \(\(\) => \{\r?$/gmu;
  for (const guarded of after.matchAll(guardedHeader)) {
    const [, indent, emitProps] = guarded;
    const directPattern = new RegExp(
      `^${escape(indent!)}const ${escape(emitProps!)} = \\(undefined as unknown as [^\\r\\n]+$`,
      "gmu",
    );
    const directMatches = [...before.matchAll(directPattern)];
    const wrappedPattern = new RegExp(
      `^${escape(indent!)}const ${escape(emitProps!)} = \\(\\(\\) => \\{\\r?$`,
      "gmu",
    );
    const wrappedMatches = [...before.matchAll(wrappedPattern)];
    const inferenceMatches = directMatches.length > 0 ? directMatches : wrappedMatches;
    if (inferenceMatches.length === 0) continue;
    assert.equal(inferenceMatches.length, 1, "one guarded-listener inference owner");
    const directAt = inferenceMatches[0]!.index;

    const ownerPattern = new RegExp(
      `^${escape(indent!)}type ([^\\s<=>]+)__listener_missing<[^\\r\\n]+$`,
      "gmu",
    );
    const owner = [...before.slice(0, directAt).matchAll(ownerPattern)].at(-1);
    assert.ok(owner, "guarded inference follows one local listener contract");
    const localOwner = owner[1]!;
    const blockStart = owner.index;

    const inferredPattern = new RegExp(
      `^${escape(indent!)}type ([^\\s=]+_inferred_emit_args) = ([^\\s=]+)__library extends true [^\\r\\n]+$`,
      "gmu",
    );
    const inferred = [...after.slice(guarded.index).matchAll(inferredPattern)][0];
    assert.ok(inferred, "guarded inference retains its inferred listener tuple");
    const inferredName = inferred[1]!;
    const sharedOwner = inferred[2]!;
    if (!inferredName.startsWith(`${localOwner}_`) || sharedOwner === localOwner) continue;
    const eventStem = inferredName.slice(localOwner.length, -"_inferred_emit_args".length);
    assert.ok(eventStem.startsWith("_") && eventStem.length > 1);
    const listenerName = `${localOwner}${eventStem}_listener`;
    const callbackAt = before.indexOf(`Parameters<${listenerName}>`, directAt);
    assert.ok(callbackAt > directAt, "guarded listener retains its callback body");
    const closingPattern = new RegExp(`^${escape(indent!)}\\}\\);\\r?$`, "gmu");
    const closing = [...before.slice(callbackAt).matchAll(closingPattern)][0];
    assert.ok(closing, "guarded listener retains one complete callback");
    const closingEnd = callbackAt + closing.index + closing[0].length;
    const newline = before.startsWith("\r\n", closingEnd) ? "\r\n" : "\n";
    assert.equal(before.slice(closingEnd, closingEnd + newline.length), newline);
    const blockEnd = closingEnd + newline.length;
    const block = before.slice(blockStart, blockEnd);
    const retained = new Set(
      ["inferred_emit_args", "emit_args", "args", "event", "listener", "handler"].map(
        (suffix) => `${localOwner}${eventStem}_${suffix}`,
      ),
    );

    const declarations: Array<{
      name: string;
      shared: string;
      start: number;
      end: number;
    }> = [];
    let lineStart = blockStart;
    for (const line of block.match(/[^\r\n]*(?:\r\n|\n|$)/gu) ?? []) {
      if (!line) continue;
      const declaration = line.match(
        new RegExp(`^${escape(indent!)}type ([^\\s<=>]+)(?:<.*?>)? = `, "u"),
      );
      if (declaration?.[1]?.startsWith(localOwner) && !retained.has(declaration[1])) {
        const normalized = line.replaceAll(localOwner, sharedOwner);
        const earlierBefore = before.slice(0, blockStart).split(normalized).length - 1;
        const earlierAfter = after.slice(0, guarded.index).split(normalized).length - 1;
        if (earlierBefore === 1 && earlierAfter === 1) {
          declarations.push({
            name: declaration[1],
            shared: declaration[1].replace(localOwner, sharedOwner),
            start: lineStart,
            end: lineStart + line.length,
          });
        }
      }
      lineStart += line.length;
    }
    assert.ok(declarations.length >= 10, "the complete duplicate listener contract is reused");
    for (const declaration of declarations) add(declaration.start, declaration.end, "");

    const removed = (offset: number) =>
      declarations.some(({ start, end }) => start <= offset && offset < end);
    const staticRanges: Array<{ start: number; end: number }> = [];
    for (const match of block.matchAll(matcher(previousTemplate))) {
      const start = blockStart + match.index;
      const replacements = ["unknown[]", "__L"]
        .map((loose_args) =>
          render(currentTemplate, {
            ...match.groups!,
            loose_args,
          }),
        )
        .map((replacement) => {
          for (const declaration of declarations)
            replacement = replacement.replaceAll(declaration.name, declaration.shared);
          return replacement;
        })
        .filter((replacement, index, all) => all.indexOf(replacement) === index)
        .filter((replacement) => after.split(replacement).length === 2);
      assert.equal(replacements.length, 1, "one shared current static inference declaration");
      add(start, start + match[0].length, replacements[0]!);
      staticRanges.push({ start, end: start + match[0].length });
    }
    assert.equal(staticRanges.length, 1, "one guarded static listener migration");

    const inStaticRange = (offset: number) =>
      staticRanges.some(({ start, end }) => start <= offset && offset < end);
    for (const declaration of [...declarations].sort(
      (left, right) => right.name.length - left.name.length,
    )) {
      const identifier = new RegExp(
        `(?<![A-Za-z0-9_$])${escape(declaration.name)}(?![A-Za-z0-9_$])`,
        "gu",
      );
      for (const match of block.matchAll(identifier)) {
        const start = blockStart + match.index;
        if (removed(start) || inStaticRange(start)) continue;
        add(start, start + match[0].length, declaration.shared);
      }
    }

    const blockStartByte = byteAt(blockStart);
    const blockEndByte = byteAt(blockEnd);
    const blockEdits = [...guardedEdits, ...edits]
      .filter(({ start, end }) => blockStartByte <= start && end <= blockEndByte)
      .sort((left, right) => left.start - right.start);
    for (let index = 1; index < blockEdits.length; index++)
      assert.ok(blockEdits[index - 1]!.end <= blockEdits[index]!.start);
    const source = Buffer.from(before).subarray(blockStartByte, blockEndByte);
    const chunks: Buffer[] = [];
    let position = blockStartByte;
    for (const edit of blockEdits) {
      chunks.push(
        source.subarray(position - blockStartByte, edit.start - blockStartByte),
        edit.replacement,
      );
      position = edit.end;
    }
    chunks.push(source.subarray(position - blockStartByte));
    const expected = Buffer.concat(chunks).toString();
    const firstLineEnd = expected.indexOf(newline) + newline.length;
    const actualAt = after.indexOf(expected.slice(0, firstLineEnd));
    const actualBlock = actualAt < 0 ? "" : after.slice(actualAt, actualAt + expected.length);
    let mismatch = -1;
    for (let index = 0; index < expected.length; index++) {
      if (expected[index] !== actualBlock[index]) {
        mismatch = index;
        break;
      }
    }
    assert.equal(
      after.split(expected).length,
      2,
      `one exact guarded shared-contract block at ${mismatch}: ${JSON.stringify(expected.slice(Math.max(0, mismatch - 80), mismatch + 160))} != ${JSON.stringify(actualBlock.slice(Math.max(0, mismatch - 80), mismatch + 160))}`,
    );
  }
  return edits;
}

function invertEdits(source: string, edits: Edit[]): Edit[] {
  const bytes = Buffer.from(source);
  const coalesced: Edit[] = [];
  for (const edit of [...edits].sort((left, right) => left.start - right.start)) {
    const previous = coalesced.at(-1);
    if (previous?.end === edit.start) {
      previous.end = edit.end;
      previous.replacement = Buffer.concat([previous.replacement, edit.replacement]);
    } else coalesced.push({ ...edit });
  }
  let delta = 0;
  return coalesced.map((edit) => {
    const start = edit.start + delta;
    const inverse = {
      start,
      end: start + edit.replacement.length,
      replacement: bytes.subarray(edit.start, edit.end),
    };
    delta += edit.replacement.length - (edit.end - edit.start);
    return inverse;
  });
}

function guardedContractOwnershipEdits(
  before: string,
  after: string,
  previousTemplate: string,
  currentTemplate: string,
  guardedEdits: Edit[],
): Edit[] {
  const forward = collapseGuardedContractOwnershipEdits(
    before,
    after,
    previousTemplate,
    currentTemplate,
    guardedEdits,
  );
  if (forward.length > 0) return forward;
  const reverseGuarded = guardedInferenceEdits(after, before);
  const reverse = collapseGuardedContractOwnershipEdits(
    after,
    before,
    previousTemplate,
    currentTemplate,
    reverseGuarded,
  );
  return invertEdits(after, [...reverseGuarded, ...reverse]);
}

export function currentCallbackSourceEdits(
  before: string,
  after: string,
  authored?: string,
  readSource: CallbackSourceReader = read,
): { edits: Edit[]; addedMappings: number[][] } {
  const currentEmit = readSource(emitSource);
  const previousEmit = beforeGenericEventSpecialization(readSource, emitSource, currentEmit);
  const currentGenerator = readSource(staticListenerSource);
  const previousGenerator = beforeIntegratedStaticListenerSource(
    readSource,
    staticListenerSource,
    currentGenerator,
  );
  const edits: Edit[] = [];
  const addedMappings: number[][] = [];
  const add = (start: number, length: number, replacement: string) => {
    const offset = Buffer.byteLength(before.slice(0, start));
    edits.push({
      start: offset,
      end: offset + Buffer.byteLength(before.slice(start, start + length)),
      replacement: Buffer.from(replacement),
    });
  };
  const transitionRaw = readSource(integratedSourceTransition);
  const digest = (text: string) => createHash("sha256").update(text).digest("hex");
  assert.equal(
    digest(transitionRaw),
    "9f4babc2f1d99fc129ceab018266503a78b60d8174374122fcce679694159518",
    "certified integrated component helper transition",
  );
  const transition = JSON.parse(transitionRaw) as {
    file: {
      path: string;
      before: { bytes: number; sha256: string };
      after: { bytes: number; sha256: string };
      edits: Array<{ before: string; after: string }>;
    };
  };
  assert.equal(transition.file.path, componentHelperSource);
  const currentHelper = readSource(componentHelperSource);
  const strictExtraKeysPredecessor = beforeStrictExtraKeys(currentHelper);
  const integratedHelper = beforeDiscriminatedJsxUnion(strictExtraKeysPredecessor);
  assert.equal(Buffer.byteLength(integratedHelper), transition.file.after.bytes);
  assert.equal(digest(integratedHelper), transition.file.after.sha256);
  let previousHelper = integratedHelper;
  for (const edit of [...transition.file.edits].reverse()) {
    assert.equal(previousHelper.split(edit.after).length, 2);
    previousHelper = previousHelper.replace(edit.after, edit.before);
  }
  assert.equal(Buffer.byteLength(previousHelper), transition.file.before.bytes);
  assert.equal(digest(previousHelper), transition.file.before.sha256);
  const helperVariants = [
    previousHelper,
    integratedHelper,
    strictExtraKeysPredecessor,
    currentHelper,
  ];
  assert.equal(new Set(helperVariants).size, helperVariants.length);
  const helperCounts = [before, after].map((text) =>
    helperVariants.map((helper) => text.split(helper).length - 1),
  );
  if (helperCounts.some((counts) => counts.some((count, i) => count !== helperCounts[0]![i]))) {
    const predecessor = helperCounts[0]!.findIndex((count, index) => {
      return index < helperVariants.length - 1 && count === 1;
    });
    assert.notEqual(predecessor, -1, "one certified component helper predecessor is required");
    assert.deepEqual(
      helperCounts[0],
      helperVariants.map((_, index) => (index === predecessor ? 1 : 0)),
      "only one whole certified component helper predecessor may migrate",
    );
    assert.deepEqual(
      helperCounts[1],
      helperVariants.map((_, index) => (index === helperVariants.length - 1 ? 1 : 0)),
      "the whole current component helper is required",
    );
    add(
      before.indexOf(helperVariants[predecessor]!),
      helperVariants[predecessor]!.length,
      currentHelper,
    );
  } else {
    assert.ok(
      helperCounts.every(
        (counts) =>
          counts.every((count) => count <= 1) && counts.reduce((sum, count) => sum + count, 0) <= 1,
      ),
      "canonical component helper is unique",
    );
  }
  const overloadTransition = readSource(emitOverloadTransitionSource);
  assert.equal(
    digest(overloadTransition),
    "62d8b5eec112d39005a234ac4c819a8c9de4668e1d94149d16fdfc5b010d6e95",
    "certified emit-overload window transition",
  );
  const overloadAsset = (name: string): string => {
    const match = new RegExp(
      `pub\\(super\\) const ${name}: &str = ("(?:[^"\\\\]|\\\\.)*");`,
      "u",
    ).exec(overloadTransition);
    assert.ok(match, name);
    return JSON.parse(match[1]!) as string;
  };
  const previousOverloads = overloadAsset("ASSET_BEFORE");
  const currentOverloads = overloadAsset("ASSET_AFTER");
  assert.equal(
    digest(previousOverloads),
    "1bc7b3912b96929575a09809d42d23a6b1b8015e6e2014ef493cd7820da33477",
  );
  assert.equal(
    digest(currentOverloads),
    "bbd7d2f184d1f9c5aff57d781ae6ccd393985a0ac4f01981282eb715cfc81ee0",
  );
  const currentOverloadAsset = beforeCanonLeftovers(
    emitOverloadsSource,
    readSource(emitOverloadsSource),
  );
  assert.equal(Buffer.byteLength(currentOverloadAsset), 2535);
  assert.equal(
    digest(currentOverloadAsset),
    "924bb683376f6c2ab3b9225cd6eff73328f8dfe4e87529a1c1a8068c03923e74",
  );
  assert.equal(currentOverloadAsset.split(currentOverloads).length, 2);
  assert.equal(currentOverloadAsset.split(previousOverloads).length, 1);
  const overloadCounts = [before, after].map((text) => [
    text.split(previousOverloads).length - 1,
    text.split(currentOverloads).length - 1,
  ]);
  if (
    overloadCounts[0]![0] !== overloadCounts[1]![0] ||
    overloadCounts[0]![1] !== overloadCounts[1]![1]
  ) {
    assert.deepEqual(
      overloadCounts,
      [
        [1, 0],
        [0, 1],
      ],
      "only the certified emit-overload window may migrate",
    );
    add(before.indexOf(previousOverloads), previousOverloads.length, currentOverloads);
  } else {
    assert.ok(
      overloadCounts.every((pair) => pair.every((count) => count <= 1) && pair[0]! + pair[1]! <= 1),
      "canonical emit-overload window is unique",
    );
  }
  const constructorEdit = certifyComponentConstructorTransition(
    Buffer.from(before),
    Buffer.from(after),
  );
  if (constructorEdit) edits.push(constructorEdit);
  const componentExport = certifyComponentExportTransition(Buffer.from(before), Buffer.from(after));
  if (componentExport) {
    edits.push(...componentExport.edits);
    if (componentExport.addedMapping) addedMappings.push(componentExport.addedMapping);
  }
  const counts = [before, after].map((text) => [
    text.split(previousEmit).length - 1,
    text.split(currentEmit).length - 1,
  ]);
  if (counts[0]![0] !== counts[1]![0] || counts[0]![1] !== counts[1]![1]) {
    assert.deepEqual(
      counts,
      [
        [1, 0],
        [0, 1],
      ],
      "only the whole certified emit helper may migrate",
    );
    add(before.indexOf(previousEmit), previousEmit.length, currentEmit);
  } else {
    assert.ok(
      counts.every((pair) => pair.every((n) => n <= 1) && pair[0]! + pair[1]! <= 1),
      "canonical emit helper is unique",
    );
  }
  const oldTemplate = template(previousGenerator),
    newTemplate = template(currentGenerator);
  const guardedEdits = guardedInferenceEdits(before, after);
  edits.push(...guardedEdits);
  edits.push(
    ...guardedContractOwnershipEdits(before, after, oldTemplate, newTemplate, guardedEdits),
  );
  const matches = [...before.matchAll(matcher(oldTemplate))];
  if (matches.length > 0) {
    assert.equal(
      typeof authored,
      "string",
      "static emitter transition requires its authored source",
    );
    assert.doesNotMatch(
      authored!,
      /<component\b/iu,
      "computed component targets need a separate certificate",
    );
    for (const match of matches) {
      const start = Buffer.byteLength(before.slice(0, match.index));
      const end = start + Buffer.byteLength(match[0]);
      if (edits.some((edit) => edit.start <= start && end <= edit.end)) continue;
      const replacements = ["unknown[]", "__L"]
        .map((loose_args) =>
          render(newTemplate, {
            ...match.groups!,
            loose_args,
          }),
        )
        .filter((replacement, index, all) => all.indexOf(replacement) === index)
        .filter((replacement) => after.split(replacement).length === 2);
      assert.equal(
        replacements.length,
        1,
        "exactly one current static inference declaration is present",
      );
      add(match.index, match[0].length, replacements[0]!);
    }
  }
  return { edits, addedMappings };
}
