/**
 * Streaming census of a tsgo `--generateTrace` types file (`types_<checker>.json`,
 * one record per line). Every type gets two origin keys:
 * - declared: its own declaration's (alias or symbol, as types.json reports
 *   it), or for an anonymous union, intersection, conditional or literal the
 *   declared key of its creator;
 * - created for: the first Vize-owned key (helper or projection) up its
 *   creator chain, so a lib `Pick` built inside a helper counts for the helper.
 * A type's creator is the first type created after it that references it.
 */

import { createReadStream } from "node:fs";
import { createInterface } from "node:readline";

export type TraceLocation = { path: string; start: { line: number; character: number } };
export type TypeRecord = {
  id: number;
  intrinsicName?: string;
  symbolName?: string;
  flags?: string[];
  display?: string;
  firstDeclaration?: TraceLocation;
} & Partial<Record<(typeof CHILD_LISTS)[number], number[]>> &
  Partial<Record<(typeof CHILD_FIELDS)[number], number>>;

const CHILD_LISTS = [
  "unionTypes",
  "intersectionTypes",
  "aliasTypeArguments",
  "typeArguments",
] as const;
const CHILD_FIELDS = [
  "keyofType",
  "conditionalCheckType",
  "conditionalExtendsType",
  "conditionalTrueType",
  "conditionalFalseType",
  "substitutionBaseType",
  "constraintType",
  "indexedAccessObjectType",
  "indexedAccessIndexType",
  "reverseMappedSourceType",
  "reverseMappedMappedType",
  "reverseMappedConstraintType",
  "evolvingArrayElementType",
  "evolvingArrayFinalType",
] as const;

export function childIds(record: TypeRecord): number[] {
  const children: number[] = [];
  for (const list of CHILD_LISTS) children.push(...(record[list] ?? []));
  for (const field of CHILD_FIELDS) {
    const child = record[field];
    if (child != null) children.push(child);
  }
  return children;
}

/** Records of one types file in id order; tolerates the `[`/`,`/`]` framing tsgo writes per line. */
export async function* readTypes(path: string): AsyncGenerator<TypeRecord> {
  for await (const raw of createInterface({ input: createReadStream(path), crlfDelay: Infinity })) {
    let line = raw.trim();
    if (line.startsWith("[")) line = line.slice(1);
    if (line.endsWith(",") || line.endsWith("]")) line = line.slice(0, -1);
    if (line) yield JSON.parse(line) as TypeRecord;
  }
}

export type TypeDescriptor = { key: string; owner: string; display: string };
export type TypeCensus = {
  /** Types created per declared origin key. */
  created: Map<string, number>;
  /** Types created per Vize-owned key up the creator chain (else the declared key). */
  createdFor: Map<string, number>;
  /** Descriptors of the requested ids. */
  described: Map<number, TypeDescriptor>;
  /** Anonymous types no later type references, by flags and display: built and dropped mid-check. */
  unowned: Map<string, number>;
  /** Requested [source, target] pairs that are members of one union, with that union's key. */
  siblings: Map<string, string>;
  total: number;
};

export const pairKey = (source: number, target: number): string => `${source}>${target}`;

/**
 * One pass over a types file. `origin` names a record's own declaration (null
 * when it has none); ids in `wanted` are described; `pairs` are checked for
 * union siblinghood, the shape union subtype reduction compares.
 */
export async function censusTypes(
  path: string,
  origin: (record: TypeRecord) => string | null,
  owned: (key: string) => boolean,
  wanted: ReadonlySet<number>,
  pairs: ReadonlyArray<readonly [number, number]>,
): Promise<TypeCensus> {
  const keys: string[] = [];
  const keyIndex = new Map<string, number>();
  const intern = (key: string) => {
    let index = keyIndex.get(key);
    if (index == null) {
      index = keys.length;
      keys.push(key);
      keyIndex.set(key, index);
    }
    return index;
  };
  // Per type id: its own key index, and the creating parent for anonymous types.
  const own: number[] = [];
  const parent: number[] = [];
  const displays = new Map<number, string>();
  const pairsByMember = new Map<number, number[]>();
  pairs.forEach(([source, target], index) => {
    for (const member of [source, target]) {
      pairsByMember.set(member, [...(pairsByMember.get(member) ?? []), index]);
    }
  });
  const siblingUnion = new Map<number, number>();
  const intrinsic = new Set<number>();
  const anonymous = new Map<number, string>();
  let maxId = 0;
  for await (const record of readTypes(path)) {
    const { id } = record;
    maxId = Math.max(maxId, id);
    // Intrinsics exist before any check; no instantiation creates them.
    if (record.intrinsicName != null) intrinsic.add(id);
    const key = record.intrinsicName != null ? "(intrinsic)" : origin(record);
    own[id] = intern(key ?? `(anonymous ${(record.flags ?? ["?"]).join("|")})`);
    if (key == null) own[id] = -1 - (own[id] as number);
    for (const child of childIds(record)) {
      if (child < id && parent[child] == null && own[child] != null && !intrinsic.has(child)) {
        parent[child] = id;
      }
    }
    if (wanted.has(id)) displays.set(id, (record.display ?? record.symbolName ?? "").slice(0, 120));
    if (key == null && record.intrinsicName == null) {
      anonymous.set(
        id,
        `${(record.flags ?? []).join("|")}: ${(record.display ?? "").replace(/\d+/gu, "N").slice(0, 100)}`,
      );
    }
    if (record.unionTypes && pairs.length > 0) {
      const hits = new Map<number, number>();
      for (const member of record.unionTypes) {
        for (const index of pairsByMember.get(member) ?? [])
          hits.set(index, (hits.get(index) ?? 0) + 1);
      }
      for (const [index, count] of hits) {
        const [source, target] = pairs[index] as readonly [number, number];
        if (count >= 2 && source !== target && !siblingUnion.has(index))
          siblingUnion.set(index, id);
      }
    }
  }
  // Parents are created after their children, so resolve from the highest id down.
  const resolved: number[] = [];
  const attributed: number[] = [];
  const isOwned = keys.map(owned);
  const created = new Map<string, number>();
  const createdFor = new Map<string, number>();
  const unowned = new Map<string, number>();
  const count = (map: Map<string, number>, key: number) => {
    const name = keys[key] as string;
    map.set(name, (map.get(name) ?? 0) + 1);
  };
  for (let id = maxId; id >= 1; id--) {
    const index = own[id];
    if (index == null) continue;
    const from = parent[id];
    const key = index >= 0 ? index : from != null ? (resolved[from] as number) : -1 - index;
    resolved[id] = key;
    attributed[id] =
      index >= 0 && isOwned[index] ? index : from != null ? (attributed[from] as number) : key;
    count(created, key);
    count(createdFor, attributed[id] as number);
    const shape = from == null ? anonymous.get(id) : undefined;
    if (shape != null) unowned.set(shape, (unowned.get(shape) ?? 0) + 1);
  }
  const described = new Map<number, TypeDescriptor>();
  for (const id of wanted) {
    const key = resolved[id];
    const owner = attributed[id];
    if (key != null && owner != null) {
      described.set(id, {
        key: keys[key] as string,
        owner: keys[owner] as string,
        display: displays.get(id) ?? "",
      });
    }
  }
  const siblings = new Map<string, string>();
  for (const [index, union] of siblingUnion) {
    const [source, target] = pairs[index] as readonly [number, number];
    siblings.set(pairKey(source, target), keys[resolved[union] as number] as string);
  }
  return {
    created,
    createdFor,
    unowned,
    described,
    siblings,
    total: resolved.filter((key) => key != null).length,
  };
}
