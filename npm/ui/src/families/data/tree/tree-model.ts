import type { TreeCheckedState, TreeFlatNode, TreeKey } from "./tree-types.ts";

const duplicateKeyDiagnostic = "VIZE_UI_TREE_DUPLICATE_KEY";

/** Resolved, render-independent record for one known tree node. */
export interface TreeIndexEntry<T, K extends TreeKey> {
  readonly key: K;
  readonly node: T;
  readonly parentKey: K | null;
  readonly level: number;
  readonly posinset: number;
  readonly setsize: number;
  /** Resolved child keys, or `null` when children are absent or not loaded yet. */
  readonly childKeys: readonly K[] | null;
}

/** Whole-tree index keyed by node key, plus root keys in display order. */
export interface TreeIndex<T, K extends TreeKey> {
  readonly entries: ReadonlyMap<TreeKey, TreeIndexEntry<T, K>>;
  readonly rootKeys: readonly K[];
  /** Every known key in pre-order. */
  readonly order: readonly K[];
}

/** Accessors used to walk consumer-owned nodes. */
export interface TreeIndexAccessors<T, K extends TreeKey> {
  readonly getKey: (node: T) => K;
  /** Resolved children, or `null` when absent or not loaded. */
  readonly resolveChildren: (node: T, key: K) => readonly T[] | null;
}

/**
 * Index every resolved node of a tree in pre-order.
 *
 * Collapsed branches are indexed too so checkbox propagation, expand-all, and
 * typed key lookups work for rows that are not currently visible.
 *
 * @throws An error carrying `VIZE_UI_TREE_DUPLICATE_KEY` when two nodes share a key.
 */
export function indexTree<T, K extends TreeKey>(
  items: readonly T[],
  accessors: TreeIndexAccessors<T, K>,
): TreeIndex<T, K> {
  const entries = new Map<TreeKey, TreeIndexEntry<T, K>>();
  const order: K[] = [];

  const visit = (siblings: readonly T[], parentKey: K | null, level: number): K[] => {
    const keys: K[] = [];
    siblings.forEach((node, position) => {
      const key = accessors.getKey(node);
      if (entries.has(key)) {
        throw new Error(`${duplicateKeyDiagnostic}: tree key ${String(key)} is not unique`);
      }
      const children = accessors.resolveChildren(node, key);
      const entry: TreeIndexEntry<T, K> = {
        key,
        node,
        parentKey,
        level,
        posinset: position + 1,
        setsize: siblings.length,
        childKeys: null,
      };
      entries.set(key, entry);
      order.push(key);
      keys.push(key);
      if (children !== null) {
        entries.set(key, { ...entry, childKeys: visit(children, key, level + 1) });
      }
    });
    return keys;
  };

  const rootKeys = visit(items, null, 1);
  return { entries, rootKeys, order };
}

/** Flatten the visible rows of an indexed tree for rendering and navigation. */
export function flattenVisibleTree<T, K extends TreeKey>(
  index: TreeIndex<T, K>,
  expanded: ReadonlySet<TreeKey>,
  isExpandable: (entry: TreeIndexEntry<T, K>) => boolean,
): readonly TreeFlatNode<T, K>[] {
  const rows: TreeFlatNode<T, K>[] = [];
  const visit = (keys: readonly K[]): void => {
    for (const key of keys) {
      const entry = index.entries.get(key);
      if (entry === undefined) continue;
      const expandable = isExpandable(entry);
      const open = expandable && expanded.has(key);
      rows.push(
        Object.freeze({
          key,
          node: entry.node,
          index: rows.length,
          level: entry.level,
          posinset: entry.posinset,
          setsize: entry.setsize,
          parentKey: entry.parentKey,
          expandable,
          expanded: open,
        }),
      );
      if (open && entry.childKeys !== null) visit(entry.childKeys);
    }
  };
  visit(index.rootKeys);
  return rows;
}

/** Collect every indexed descendant key of `key`, excluding `key` itself. */
export function collectDescendantKeys<T, K extends TreeKey>(
  index: TreeIndex<T, K>,
  key: TreeKey,
): K[] {
  const result: K[] = [];
  const stack = [...(index.entries.get(key)?.childKeys ?? [])].reverse();
  while (stack.length > 0) {
    const next = stack.pop();
    if (next === undefined) break;
    result.push(next);
    const children = index.entries.get(next)?.childKeys;
    if (children) stack.push(...[...children].reverse());
  }
  return result;
}

/** Whether `candidate` is a strict descendant of `ancestor`. */
export function isTreeDescendant<T, K extends TreeKey>(
  index: TreeIndex<T, K>,
  ancestor: TreeKey,
  candidate: TreeKey,
): boolean {
  let parent = index.entries.get(candidate)?.parentKey ?? null;
  while (parent !== null) {
    if (parent === ancestor) return true;
    parent = index.entries.get(parent)?.parentKey ?? null;
  }
  return false;
}

/**
 * Derive tri-state checkbox values for every indexed node.
 *
 * With `"cascade"`, a checked key implies its whole resolved subtree, and nodes
 * with resolved children aggregate them: all checked is `checked`, all
 * unchecked is `unchecked`, anything else is `mixed`. Leaves and unloaded
 * branches read their own (or an inherited) membership, so seeding only a
 * parent key checks its lazily loaded children too. `"independent"` reads
 * membership for every node.
 */
export function deriveTreeCheckedStates<T, K extends TreeKey>(
  index: TreeIndex<T, K>,
  checked: ReadonlySet<TreeKey>,
  cascade: boolean,
): ReadonlyMap<TreeKey, TreeCheckedState> {
  const states = new Map<TreeKey, TreeCheckedState>();
  if (!cascade) {
    for (const key of index.order) states.set(key, checked.has(key) ? "checked" : "unchecked");
    return states;
  }
  const resolve = (key: K, inherited: boolean): TreeCheckedState => {
    const own = inherited || checked.has(key);
    const children = index.entries.get(key)?.childKeys ?? null;
    if (children === null || children.length === 0) {
      const state: TreeCheckedState = own ? "checked" : "unchecked";
      states.set(key, state);
      return state;
    }
    let all = true;
    let none = true;
    for (const child of children) {
      const state = resolve(child, own);
      if (state !== "checked") all = false;
      if (state !== "unchecked") none = false;
    }
    const state: TreeCheckedState = all ? "checked" : none ? "unchecked" : "mixed";
    states.set(key, state);
    return state;
  };
  for (const key of index.rootKeys) resolve(key, false);
  return states;
}

/** Return checked keys in pre-order, normalized so parents mirror their descendants. */
export function normalizeTreeChecked<T, K extends TreeKey>(
  index: TreeIndex<T, K>,
  checked: ReadonlySet<TreeKey>,
  cascade: boolean,
): readonly K[] {
  if (!cascade) return index.order.filter((key) => checked.has(key));
  const states = deriveTreeCheckedStates(index, checked, true);
  return index.order.filter((key) => states.get(key) === "checked");
}

/**
 * Toggle one node's checkbox and return the next normalized checked keys.
 *
 * Mixed and unchecked nodes become checked; checked nodes become unchecked.
 * Cascading skips disabled descendants, which keep their current value.
 */
export function toggleTreeChecked<T, K extends TreeKey>(
  index: TreeIndex<T, K>,
  checked: ReadonlySet<TreeKey>,
  key: TreeKey,
  options: { readonly cascade: boolean; readonly isDisabled: (key: TreeKey) => boolean },
): readonly K[] {
  const states = deriveTreeCheckedStates(index, checked, options.cascade);
  const target = states.get(key) !== "checked";
  const next = new Set<TreeKey>(
    options.cascade
      ? index.order.filter((candidate) => states.get(candidate) === "checked")
      : checked,
  );
  const apply = (candidate: TreeKey) => {
    if (target) next.add(candidate);
    else next.delete(candidate);
  };
  apply(key);
  if (!options.cascade) return normalizeTreeChecked(index, next, false);
  for (const descendant of collectDescendantKeys(index, key)) {
    if (!options.isDisabled(descendant)) apply(descendant);
  }
  // Branches with resolved children are re-derived from their descendants, so a
  // disabled unchecked descendant correctly leaves its ancestors mixed.
  for (const candidate of index.order) {
    if ((index.entries.get(candidate)?.childKeys?.length ?? 0) > 0) next.delete(candidate);
  }
  return normalizeTreeChecked(index, next, true);
}

/** Whether two key lists contain the same keys in the same order. */
export function treeKeysEqual(left: readonly TreeKey[], right: readonly TreeKey[]): boolean {
  return left.length === right.length && left.every((key, index) => Object.is(key, right[index]));
}

const safeSegment = /^[A-Za-z0-9][A-Za-z0-9_-]*$/;

/** Create a deterministic, DOM-id-safe segment for an arbitrary tree key. */
export function getTreeKeyIdSegment(key: TreeKey): string {
  if (typeof key === "number") {
    return Number.isSafeInteger(key) && key >= 0
      ? `num-${key}`
      : `num-${hashTreeKey(`number:${key}`)}`;
  }
  if (safeSegment.test(key)) return `key-${key}`;
  const readable = key
    .replaceAll(/[^A-Za-z0-9_-]+/g, "-")
    .replaceAll(/^-+|-+$/g, "")
    .slice(0, 32);
  return `key-${readable || "empty"}-${hashTreeKey(key)}`;
}

function hashTreeKey(value: string): string {
  let hash = 0x811c9dc5;
  for (let index = 0; index < value.length; index++) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193) >>> 0;
  }
  return hash.toString(36);
}
