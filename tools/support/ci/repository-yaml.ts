import { isDeepStrictEqual } from "node:util";
import {
  isAlias,
  isMap,
  isScalar,
  isSeq,
  parseDocument,
  stringify,
  type Document,
  type Node,
  type YAMLMap,
} from "yaml";

export type ConfigValue =
  | string
  | number
  | boolean
  | null
  | ConfigValue[]
  | { [key: string]: ConfigValue };

type ScalarEdit = { start: number; end: number; text: string };

function sameValue(left: unknown, right: ConfigValue, ordered: boolean): boolean {
  // External configuration readers can use mapping order (for example, pnpm overrides).
  return (
    isDeepStrictEqual(left, right) && (!ordered || JSON.stringify(left) === JSON.stringify(right))
  );
}

function readDocument(document: Document): ConfigValue | undefined {
  if (document.errors.length) return undefined;
  try {
    return document.toJS() as ConfigValue;
  } catch {
    // A dangling alias cannot supply a presentation template.
    return undefined;
  }
}

function matches(text: string, value: ConfigValue, ordered: boolean): boolean {
  const document = parseDocument(text);
  return sameValue(readDocument(document), value, ordered);
}

function isMapping(value: ConfigValue): value is Record<string, ConfigValue> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function itemIdentity(item: ConfigValue): string | undefined {
  if (!isMapping(item)) return undefined;
  if (typeof item.id === "string") return "id:" + item.id;
  if (typeof item.name === "string") return "name:" + item.name;
  return undefined;
}

// Comments and anchors belong to a sequence item even when it moves to a new index.
function sequenceSources(previous: ConfigValue[], next: ConfigValue[], ordered: boolean) {
  const available = new Set(previous.keys());
  function take(matches: (item: ConfigValue) => boolean): number | undefined {
    for (const index of available) {
      if (matches(previous[index]!)) {
        available.delete(index);
        return index;
      }
    }
    return undefined;
  }
  const indices = next.map((item) => take((old) => sameValue(old, item, ordered)));
  for (const [index, item] of next.entries()) {
    const identity = itemIdentity(item);
    if (indices[index] === undefined && identity !== undefined) {
      indices[index] = take((old) => itemIdentity(old) === identity);
    }
  }
  if (previous.length === next.length) {
    for (const index of next.keys()) {
      if (indices[index] === undefined && available.delete(index)) indices[index] = index;
    }
  }
  return indices;
}

// A mapping that gains one scalar key keeps its presentation: the new pair is
// written in place, before the key that follows it in the new value.
function insertedKey(
  node: YAMLMap,
  value: Record<string, ConfigValue>,
  edits: ScalarEdit[],
  source: string,
): boolean {
  const keys = Object.keys(value);
  if (node.items.length === keys.length) return true;
  const present = node.items.map((pair) => (isScalar(pair.key) ? pair.key.value : undefined));
  const added = keys.filter((key) => !present.includes(key));
  if (node.items.length + 1 !== keys.length || added.length !== 1) return false;
  const key = added[0]!;
  const child = value[key]!;
  if (child !== null && typeof child === "object") return false;
  const following = node.items[keys.indexOf(key)];
  const start = isScalar(following?.key) ? following.key.range?.[0] : undefined;
  if (start === undefined) return false;
  const pair = stringify({ [key]: child }, { lineWidth: 0 }).trimEnd();
  if (/[\r\n]/u.test(pair)) return false;
  const lineStart = source.lastIndexOf("\n", start - 1) + 1;
  const indent = source.slice(lineStart, start);
  const separator = node.flow ? ", " : /^[ ]*$/u.test(indent) ? "\n" + indent : undefined;
  if (separator === undefined) return false;
  edits.push({ start, end: start, text: pair + separator });
  return true;
}

function scalarEdits(
  node: Node | null,
  previous: ConfigValue,
  value: ConfigValue,
  edits: ScalarEdit[],
  ordered: boolean,
  source: string,
): boolean {
  if (isMap(node) && isMapping(previous) && isMapping(value)) {
    if (!insertedKey(node, value, edits, source)) return false;
    return node.items.every((pair) => {
      if (!isScalar(pair.key) || typeof pair.key.value !== "string") return false;
      const key = pair.key.value;
      return (
        Object.hasOwn(value, key) &&
        scalarEdits(pair.value as Node | null, previous[key]!, value[key]!, edits, ordered, source)
      );
    });
  }
  if (isSeq(node) && Array.isArray(previous) && Array.isArray(value)) {
    const plainValues = node.items.every(
      (item) => isScalar(item) && !item.comment && !item.commentBefore && !item.anchor,
    );
    return (
      node.items.length === value.length &&
      (plainValues ||
        sequenceSources(previous, value, ordered).every((source, index) => source === index)) &&
      value.every((child, index) =>
        scalarEdits(
          node.items[index] as Node | null,
          previous[index]!,
          child,
          edits,
          ordered,
          source,
        ),
      )
    );
  }
  if (isAlias(node)) return true; // The caller verifies the expanded result.
  if (!isScalar(node) || (typeof value === "object" && value !== null)) return false;
  if (node.value === value) return true;
  if (!node.range || node.type === "BLOCK_LITERAL" || node.type === "BLOCK_FOLDED") return false;
  let text = JSON.stringify(value);
  if (typeof value === "string" && !/[\r\n]/u.test(value)) {
    if (node.type === "QUOTE_SINGLE") text = "'" + value.replaceAll("'", "''") + "'";
    if (node.type === "PLAIN") text = stringify(value, { lineWidth: 0 }).trimEnd();
  }
  edits.push({ start: node.range[0], end: node.range[1], text });
  return true;
}

// Retain existing YAML presentation for consumers that also read source comments,
// anchors, and shell blocks. Pkl determines every key, value, and sequence.
export function renderRepositoryYaml(value: ConfigValue, current?: string, ordered = true): string {
  if (current !== undefined) {
    const document = parseDocument(current);
    const existing = readDocument(document);
    if (existing !== undefined) {
      if (sameValue(existing, value, ordered)) return current;
      const edits: ScalarEdit[] = [];
      if (scalarEdits(document.contents, existing, value, edits, ordered, current)) {
        let text = current;
        for (const { start, end, text: replacement } of edits.reverse()) {
          text = text.slice(0, start) + replacement + text.slice(end);
        }
        if (matches(text, value, ordered)) return text;
      }

      // Preserve the surrounding document for structural edits, including multiline run steps.
      function update(
        previous: ConfigValue,
        next: ConfigValue,
        location: (string | number)[],
      ): void {
        if (sameValue(previous, next, ordered)) return;
        const node = document.getIn(location, true);
        if (isSeq(node) && Array.isArray(previous) && Array.isArray(next)) {
          const sources = sequenceSources(previous, next, ordered);
          const items = node.items;
          node.items = sources.map((source, index) =>
            source === undefined ? document.createNode(next[index]) : items[source],
          );
          sources.forEach((source, index) => {
            if (source !== undefined) update(previous[source]!, next[index]!, [...location, index]);
          });
          return;
        }
        if (isMap(node) && isMapping(previous) && isMapping(next)) {
          for (const key of Object.keys(previous)) {
            if (!Object.hasOwn(next, key)) document.deleteIn([...location, key]);
          }
          for (const [key, child] of Object.entries(next)) {
            const childPath = [...location, key];
            if (Object.hasOwn(previous, key)) update(previous[key]!, child, childPath);
            else document.setIn(childPath, document.createNode(child));
          }
          const pairs = new Map(node.items.map((pair) => [String(pair.key), pair]));
          node.items = Object.keys(next).map((key) => pairs.get(key)!);
          return;
        }
        if (isScalar(node) && (next === null || typeof next !== "object")) {
          node.value = next;
          if (
            typeof next === "string" &&
            !/[\r\n]/u.test(next) &&
            (node.type === "BLOCK_LITERAL" || node.type === "BLOCK_FOLDED")
          )
            node.type = "PLAIN";
          return;
        }
        document.setIn(location, document.createNode(next));
      }
      try {
        update(existing, value, []);
        const text = document.toString({ lineWidth: 0 });
        if (matches(text, value, ordered)) return text;
      } catch {
        // An edit through an alias may require materializing its independent value.
      }
    }
  }
  return stringify(value, { lineWidth: 0, aliasDuplicateObjects: false });
}
