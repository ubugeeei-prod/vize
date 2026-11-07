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
import { beforeStrictExtraKeys } from "./strict-extra-keys-transition.ts";

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
    "d34d9c30d9c270ac5a9f515b40378556c7a2d9b5a231cd8413c0440bcbd4d557",
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
  const currentOverloadAsset = readSource(emitOverloadsSource);
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
  edits.push(...guardedInferenceEdits(before, after));
  return { edits, addedMappings };
}
