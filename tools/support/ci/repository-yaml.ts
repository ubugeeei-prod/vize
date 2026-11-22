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
  return ordered ? JSON.stringify(left) === JSON.stringify(right) : isDeepStrictEqual(left, right);
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

function scalarEdits(node: Node | null, value: ConfigValue, edits: ScalarEdit[]): boolean {
  if (isMap(node) && value !== null && typeof value === "object" && !Array.isArray(value)) {
    return (
      node.items.length === Object.keys(value).length &&
      node.items.every((pair) => {
        if (!isScalar(pair.key) || typeof pair.key.value !== "string") return false;
        const key = pair.key.value;
        return (
          Object.hasOwn(value, key) && scalarEdits(pair.value as Node | null, value[key]!, edits)
        );
      })
    );
  }
  if (isSeq(node) && Array.isArray(value)) {
    return (
      node.items.length === value.length &&
      value.every((child, index) => scalarEdits(node.items[index] as Node | null, child, edits))
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
      if (scalarEdits(document.contents, value, edits)) {
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
        if (Array.isArray(previous) && Array.isArray(next)) {
          for (let index = previous.length - 1; index >= next.length; index--) {
            document.deleteIn([...location, index]);
          }
          next.forEach((child, index) => {
            const childPath = [...location, index];
            if (index < previous.length) update(previous[index]!, child, childPath);
            else document.setIn(childPath, document.createNode(child));
          });
          return;
        }
        if (
          previous !== null &&
          next !== null &&
          typeof previous === "object" &&
          typeof next === "object" &&
          !Array.isArray(previous) &&
          !Array.isArray(next)
        ) {
          for (const key of Object.keys(previous)) {
            if (!Object.hasOwn(next, key)) document.deleteIn([...location, key]);
          }
          for (const [key, child] of Object.entries(next)) {
            const childPath = [...location, key];
            if (Object.hasOwn(previous, key)) update(previous[key]!, child, childPath);
            else document.setIn(childPath, document.createNode(child));
          }
          const map = document.getIn(location, true);
          if (isMap(map)) {
            const pairs = new Map(map.items.map((pair) => [String(pair.key), pair]));
            map.items = Object.keys(next).map((key) => pairs.get(key)!);
          }
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
