/** Exact current emitter scaffolding; actual packets are never normalized. */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { beforeGenericEventSpecialization } from "../../../../tests/tooling/support/generic-event-specialization-source.ts";
import {
  beforeStaticListenerSource,
  staticListenerSource,
} from "../../../../tests/tooling/support/static-listener-prop-source.ts";

export type CallbackSourceReader = (path: string) => string;
type Edit = { start: number; end: number; replacement: Buffer };
const emitSource = "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-inference.d.ts.txt";
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
export function currentCallbackSourceEdits(
  before: string,
  after: string,
  authored?: string,
  readSource: CallbackSourceReader = read,
): Edit[] {
  const currentEmit = readSource(emitSource);
  const previousEmit = beforeGenericEventSpecialization(readSource, emitSource, currentEmit);
  const currentGenerator = readSource(staticListenerSource);
  const previousGenerator = beforeStaticListenerSource(
    readSource,
    staticListenerSource,
    currentGenerator,
  );
  const edits: Edit[] = [];
  const add = (start: number, length: number, replacement: string) => {
    const offset = Buffer.byteLength(before.slice(0, start));
    edits.push({
      start: offset,
      end: offset + Buffer.byteLength(before.slice(start, start + length)),
      replacement: Buffer.from(replacement),
    });
  };
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
      const replacement = render(newTemplate, { ...match.groups!, loose_args: "unknown[]" });
      assert.equal(
        after.split(replacement).length,
        2,
        "current static inference declaration is unique",
      );
      add(match.index, match[0].length, replacement);
    }
  }
  return edits;
}
