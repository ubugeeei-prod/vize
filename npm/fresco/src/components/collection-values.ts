export type CollectionValueKey<Value> = Value extends object
  ? Extract<keyof Value, string | number | symbol>
  : never;

export type CollectionBy<Value> =
  | CollectionValueKey<Value>
  | ((left: Value, right: Value) => boolean);

export type RuntimeCollectionBy =
  | string
  | number
  | symbol
  | ((left: unknown, right: unknown) => boolean);

/** Compare collection values by identity, an object key, or a custom comparator. */
export function collectionValuesEqual(
  left: unknown,
  right: unknown,
  by: RuntimeCollectionBy | undefined,
): boolean {
  if (typeof by === "function") return by(left, right);
  if (
    by !== undefined &&
    typeof left === "object" &&
    left !== null &&
    typeof right === "object" &&
    right !== null
  ) {
    return Object.is(Reflect.get(left, by), Reflect.get(right, by));
  }
  return Object.is(left, right);
}
