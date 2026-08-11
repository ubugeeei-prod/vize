import { toValue } from "vue";

import { extractCollectionTextValue, normalizeCollectionTextValue } from "./collection-text.ts";
import type { CollectionItem, CollectionItemInput, CollectionKey } from "./collection-types.ts";

/** Registration-time record kept for one collection item. */
export interface CollectionRecord<Key extends CollectionKey, Value> {
  readonly input: Readonly<CollectionItemInput<Key, Value>>;
  readonly sequence: number;
}

/** Immutable item snapshot paired with its registration sequence. */
export interface ResolvedCollectionRecord<Key extends CollectionKey, Value> {
  readonly item: CollectionItem<Key, Value>;
  readonly sequence: number;
}

const documentPositionDisconnected = 0x01;
const documentPositionPreceding = 0x02;
const documentPositionFollowing = 0x04;

/** Resolve every reactive source of one record into an immutable snapshot. */
export function resolveCollectionRecord<Key extends CollectionKey, Value>(
  record: CollectionRecord<Key, Value>,
): ResolvedCollectionRecord<Key, Value> {
  const { input } = record;
  const element = input.element === undefined ? null : (toValue(input.element) ?? null);
  const explicitText = input.textValue === undefined ? undefined : toValue(input.textValue);
  const order = input.order === undefined ? undefined : toValue(input.order);
  if (order !== undefined && !Number.isSafeInteger(order)) {
    throw new Error(
      `VIZE_UI_COLLECTION_ORDER_VALUE: ${String(input.key)} requires a safe integer order`,
    );
  }

  return {
    sequence: record.sequence,
    item: Object.freeze({
      key: input.key,
      value: input.value,
      element,
      textValue:
        explicitText === null || explicitText === undefined
          ? element === null
            ? ""
            : extractCollectionTextValue(element)
          : normalizeCollectionTextValue(explicitText),
      disabled: input.disabled === undefined ? false : Boolean(toValue(input.disabled)),
      order,
    }),
  };
}

/**
 * Sort resolved records in place using explicit order, DOM order, or
 * registration order, rejecting ambiguous mixes of explicit orders.
 */
export function sortCollectionRecords<Key extends CollectionKey, Value>(
  records: ResolvedCollectionRecord<Key, Value>[],
): void {
  const explicitOrderCount = records.filter(({ item }) => item.order !== undefined).length;
  if (explicitOrderCount > 0 && explicitOrderCount !== records.length) {
    throw new Error(
      "VIZE_UI_COLLECTION_ORDER_PARTIAL: every item must provide order when explicit ordering is used",
    );
  }

  if (explicitOrderCount === records.length && records.length > 0) {
    const seenOrders = new Set<number>();
    for (const { item } of records) {
      const order = item.order;
      if (order === undefined) continue;
      if (seenOrders.has(order)) {
        throw new Error(
          `VIZE_UI_COLLECTION_ORDER_DUPLICATE: order ${String(order)} is registered more than once`,
        );
      }
      seenOrders.add(order);
    }
    records.sort((left, right) => (left.item.order ?? 0) - (right.item.order ?? 0));
    return;
  }

  const ownerDocument = records[0]?.item.element?.ownerDocument;
  const hasCompleteDomOrder =
    ownerDocument !== undefined &&
    records.every(
      ({ item }) =>
        item.element !== null &&
        item.element.isConnected &&
        item.element.ownerDocument === ownerDocument,
    );
  if (!hasCompleteDomOrder) {
    records.sort((left, right) => left.sequence - right.sequence);
    return;
  }

  records.sort((left, right) => {
    const leftElement = left.item.element;
    const rightElement = right.item.element;
    if (leftElement !== null && rightElement !== null && leftElement !== rightElement) {
      const position = leftElement.compareDocumentPosition(rightElement);
      if ((position & documentPositionDisconnected) === 0) {
        if ((position & documentPositionFollowing) !== 0) return -1;
        if ((position & documentPositionPreceding) !== 0) return 1;
      }
    }
    return left.sequence - right.sequence;
  });
}
