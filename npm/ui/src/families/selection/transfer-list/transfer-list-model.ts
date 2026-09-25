/**
 * Pure helpers for TransferList: value equality, filtering, and the move
 * algorithm. DOM-free and deterministic for SSR and tests.
 */

/** Property keys of object values usable as a comparison key. */
export type TransferListValueKey<T> = T extends object ? Extract<keyof T, string> : never;

/** Compare values by a property key or with a custom equality function. */
export type TransferListBy<T> = TransferListValueKey<T> | ((left: T, right: T) => boolean);

/** Filter deciding whether an item stays visible for a panel query. */
export type TransferListFilter<T> = (item: T, query: string, text: string) => boolean;

/** Which panel an item lives in. */
export type TransferListSide = "source" | "target";

/** Movement requested by an action button or the keyboard. */
export type TransferListAction =
  | "move-all-to-source"
  | "move-all-to-target"
  | "move-selected-to-source"
  | "move-selected-to-target";

/** Direction of a completed move. */
export type TransferListDirection = "to-source" | "to-target";

/** Where moved items land in the target list. */
export type TransferListOrderMode = "append" | "source-order";

function readKey(value: unknown, key: string): unknown {
  return typeof value === "object" && value !== null ? Reflect.get(value, key) : value;
}

/** Resolve `by` into one equality function. */
export function createTransferEquality<T>(
  by: TransferListBy<T> | undefined,
): (left: T, right: T) => boolean {
  if (by === undefined) return Object.is;
  if (typeof by === "function") return by;
  const key: string = by;
  return (left, right) =>
    Object.is(left, right) || Object.is(readKey(left, key), readKey(right, key));
}

/** Normalize text for accent- and case-insensitive matching. */
export function normalizeTransferText(text: string): string {
  return text
    .normalize("NFKD")
    .replace(/\p{M}+/gu, "")
    .toLocaleLowerCase()
    .trim();
}

/** Default filter: the item text contains the query. */
export function containsTransferFilter<T>(item: T, query: string, text: string): boolean {
  void item;
  const needle = normalizeTransferText(query);
  return needle.length === 0 || normalizeTransferText(text).includes(needle);
}

/** Default text for a value. */
export function defaultTransferText<T>(value: T): string {
  if (typeof value === "string") return value;
  if (typeof value === "object" && value !== null) {
    for (const key of ["label", "name", "title", "text"] as const) {
      const candidate = readKey(value, key);
      if (typeof candidate === "string") return candidate;
    }
  }
  return String(value);
}

/** Serialize a value for native form submission. */
export function serializeTransferValue<T>(value: T, by: TransferListBy<T> | undefined): string {
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean" || typeof value === "bigint") {
    return String(value);
  }
  if (typeof by === "string") return serializeTransferValue(readKey(value, by), undefined);
  return JSON.stringify(value) ?? "";
}

/** Input for {@link transferItems}. */
export interface TransferInput<T> {
  /** Every item in canonical order. */
  readonly items: readonly T[];
  /** Current target values. */
  readonly target: readonly T[];
  /** Candidate values to move, already restricted to movable items. */
  readonly moving: readonly T[];
  /** Movement direction. */
  readonly direction: TransferListDirection;
  /** Target ordering policy. */
  readonly orderMode: TransferListOrderMode;
  /** Maximum target size. */
  readonly max: number;
  /** Value equality. */
  readonly equals: (left: T, right: T) => boolean;
}

/** Result of {@link transferItems}. */
export interface TransferResult<T> {
  /** Next target list. */
  readonly target: readonly T[];
  /** Values that actually moved (limited by `max`). */
  readonly moved: readonly T[];
}

function includes<T>(values: readonly T[], value: T, equals: (a: T, b: T) => boolean): boolean {
  return values.some((candidate) => equals(candidate, value));
}

/** Compute the next target list for a move. */
export function transferItems<T>(input: TransferInput<T>): TransferResult<T> {
  const { equals } = input;
  if (input.direction === "to-source") {
    const moved = input.moving.filter((value) => includes(input.target, value, equals));
    return Object.freeze({
      moved: Object.freeze(moved),
      target: Object.freeze(input.target.filter((value) => !includes(moved, value, equals))),
    });
  }
  const room = Math.max(0, input.max - input.target.length);
  const moved = input.moving
    .filter((value) => !includes(input.target, value, equals))
    .slice(0, room);
  const appended = [...input.target, ...moved];
  const target =
    input.orderMode === "source-order"
      ? [
          ...input.items.filter((item) => includes(appended, item, equals)),
          ...appended.filter((value) => !includes(input.items, value, equals)),
        ]
      : appended;
  return Object.freeze({ moved: Object.freeze(moved), target: Object.freeze(target) });
}
