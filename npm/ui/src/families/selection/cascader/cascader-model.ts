/**
 * Pure helpers for Cascader: value equality, tree access, path search, and
 * serialization. DOM-free and deterministic for SSR and tests.
 */

/**
 * Public model: one option path (`readonly T[]`, empty when nothing is
 * selected) in single mode, or a list of leaf paths when `Multiple` is `true`.
 */
export type CascaderModelValue<T, Multiple extends boolean> = Multiple extends true
  ? readonly (readonly T[])[]
  : readonly T[];

/** Property keys of object nodes usable as a comparison key. */
export type CascaderValueKey<T> = T extends object ? Extract<keyof T, string> : never;

/** Compare nodes by a property key or with a custom equality function. */
export type CascaderBy<T> = CascaderValueKey<T> | ((left: T, right: T) => boolean);

/** How branch options expand their child column. */
export type CascaderExpandTrigger = "click" | "hover";

/** Resolved node equality. */
export type CascaderEquality<T> = (left: T, right: T) => boolean;

/** Child accessor: `undefined` means "no static children". */
export type CascaderChildren<T> = (node: T) => readonly T[] | undefined;

function readKey(value: unknown, key: string): unknown {
  return typeof value === "object" && value !== null ? Reflect.get(value, key) : value;
}

/** Resolve `by` into one equality function. */
export function createCascaderEquality<T>(by: CascaderBy<T> | undefined): CascaderEquality<T> {
  if (by === undefined) return Object.is;
  if (typeof by === "function") return by;
  const key: string = by;
  return (left, right) =>
    Object.is(left, right) || Object.is(readKey(left, key), readKey(right, key));
}

/** Default child accessor: an array-valued `children` property. */
export function defaultCascaderChildren<T>(node: T): readonly T[] | undefined {
  const children = readKey(node, "children");
  return Array.isArray(children) ? (children as readonly T[]) : undefined;
}

/** Default text for a node. */
export function defaultCascaderText<T>(node: T): string {
  if (typeof node === "string") return node;
  if (typeof node === "object" && node !== null) {
    for (const key of ["label", "name", "title", "text", "value"] as const) {
      const candidate = readKey(node, key);
      if (typeof candidate === "string") return candidate;
    }
  }
  return String(node);
}

/** Serialize one node for form submission. */
export function serializeCascaderNode<T>(node: T, by: CascaderBy<T> | undefined): string {
  if (typeof node === "string") return node;
  if (typeof node === "number" || typeof node === "boolean" || typeof node === "bigint") {
    return String(node);
  }
  if (typeof by === "string") return serializeCascaderNode(readKey(node, by), undefined);
  for (const key of ["value", "id"] as const) {
    const candidate = readKey(node, key);
    if (typeof candidate === "string" || typeof candidate === "number") return String(candidate);
  }
  return JSON.stringify(node) ?? "";
}

/** Whether two paths hold equal nodes in the same order. */
export function areCascaderPathsEqual<T>(
  left: readonly T[],
  right: readonly T[],
  equals: CascaderEquality<T>,
): boolean {
  return (
    left.length === right.length && left.every((node, index) => equals(node, right[index] as T))
  );
}

/** Index of `path` inside `paths`, or `-1`. */
export function indexOfCascaderPath<T>(
  paths: readonly (readonly T[])[],
  path: readonly T[],
  equals: CascaderEquality<T>,
): number {
  return paths.findIndex((candidate) => areCascaderPathsEqual(candidate, path, equals));
}

function isPathList<T>(
  value: readonly (readonly T[])[] | readonly T[],
): value is readonly (readonly T[])[] {
  return value.every((entry) => Array.isArray(entry));
}

/**
 * Normalize a public model into the internal list of selected paths.
 *
 * In single mode the model is one path; in multiple mode a list of paths.
 * Empty paths are dropped.
 */
export function toCascaderSelection<T>(
  value: readonly (readonly T[])[] | readonly T[] | undefined,
  multiple: boolean,
): readonly (readonly T[])[] {
  if (value === undefined || value.length === 0) return Object.freeze([]);
  if (multiple && isPathList(value)) {
    return Object.freeze(
      value.filter((path) => path.length > 0).map((path) => Object.freeze([...path])),
    );
  }
  // A single-mode model is exactly one path of nodes, even when nodes are arrays.
  return Object.freeze([Object.freeze([...(value as readonly T[])])]);
}

/** Convert the selected paths into the public model for `multiple`. */
export function fromCascaderSelection<T, Multiple extends boolean>(
  paths: readonly (readonly T[])[],
  multiple: boolean,
): CascaderModelValue<T, Multiple> {
  const model: readonly (readonly T[])[] | readonly T[] = multiple
    ? Object.freeze([...paths])
    : (paths[0] ?? Object.freeze([]));
  return model as CascaderModelValue<T, Multiple>;
}

/** Depth-first search for the first path whose last node matches `predicate`. */
export function findCascaderPath<T>(
  options: readonly T[],
  predicate: (node: T, path: readonly T[]) => boolean,
  getChildren: CascaderChildren<T> = defaultCascaderChildren,
): readonly T[] | null {
  const visit = (nodes: readonly T[], parents: readonly T[]): readonly T[] | null => {
    for (const node of nodes) {
      const path = [...parents, node];
      if (predicate(node, path)) return Object.freeze(path);
      const children = getChildren(node);
      if (children !== undefined && children.length > 0) {
        const found = visit(children, path);
        if (found !== null) return found;
      }
    }
    return null;
  };
  return visit(options, []);
}

/**
 * Flatten the (loaded part of the) tree into selectable paths: every leaf
 * path, plus every branch path when `includeBranches` is set.
 */
export function flattenCascaderPaths<T>(
  options: readonly T[],
  getChildren: CascaderChildren<T> = defaultCascaderChildren,
  includeBranches = false,
): readonly (readonly T[])[] {
  const paths: (readonly T[])[] = [];
  const visit = (nodes: readonly T[], parents: readonly T[]): void => {
    for (const node of nodes) {
      const path = Object.freeze([...parents, node]);
      const children = getChildren(node);
      const branch = children !== undefined && children.length > 0;
      if (!branch || includeBranches) paths.push(path);
      if (branch) visit(children, path);
    }
  };
  visit(options, []);
  return Object.freeze(paths);
}

/** Normalize text for accent- and case-insensitive matching. */
export function normalizeCascaderText(text: string): string {
  return text
    .normalize("NFKD")
    .replace(/\p{M}+/gu, "")
    .toLocaleLowerCase()
    .trim();
}

/** Keep paths whose joined node text contains `query`. */
export function searchCascaderPaths<T>(
  paths: readonly (readonly T[])[],
  query: string,
  text: (node: T) => string,
): readonly (readonly T[])[] {
  const needle = normalizeCascaderText(query);
  if (needle.length === 0) return Object.freeze([]);
  return Object.freeze(
    paths.filter((path) => normalizeCascaderText(path.map(text).join(" ")).includes(needle)),
  );
}

/** Join a path into display or form text. */
export function joinCascaderPath<T>(
  path: readonly T[],
  text: (node: T) => string,
  separator: string,
): string {
  return path.map(text).join(separator);
}
