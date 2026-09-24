import { shallowReadonly, shallowRef } from "vue";

import { useSortable } from "../../interaction/sortable/sortable.ts";
import type {
  TreeDropPosition,
  TreeKey,
  TreeMoveEvent,
  TreeReorderController,
  TreeReorderItemRegistration,
  TreeReorderOptions,
} from "./tree-types.ts";

/**
 * Create a drag-to-reorder adapter for one TreeRoot.
 *
 * Pointer drags run through the shared sortable and drag-and-drop engine with
 * nesting enabled, so drops report `before`, `after`, or `inside` a target row
 * and speak through its live region. Keyboard users move the focused row with
 * `Alt+ArrowUp`/`Alt+ArrowDown` among siblings, `Alt+ArrowRight` into the
 * previous sibling, and `Alt+ArrowLeft` out of the parent. The tree never
 * mutates consumer data: `onMove` receives the request and the consumer
 * updates `items`. Must be called during component setup.
 *
 * @example
 * ```ts
 * const reorder = useTreeReorder<string>({ onMove: (move) => moveNode(move) });
 * ```
 */
export function useTreeReorder<K extends TreeKey>(
  options: TreeReorderOptions<K>,
): TreeReorderController<K> {
  const keys = new Map<string, K>();
  const dropTarget = shallowRef<{ readonly key: K; readonly position: TreeDropPosition } | null>(
    null,
  );
  const isDisabled = () => options.disabled?.() === true;

  const move = (event: TreeMoveEvent<K>): boolean => {
    if (isDisabled() || Object.is(event.key, event.targetKey)) return false;
    if (options.canMove?.(event) === false) return false;
    options.onMove(Object.freeze({ ...event }));
    return true;
  };

  const sortable = useSortable({
    nesting: true,
    isDisabled,
    onSortPreview(event) {
      const key = event.overKey === null ? undefined : keys.get(event.overKey);
      dropTarget.value =
        key === undefined || event.position === null ? null : { key, position: event.position };
    },
    onSortCommit(event) {
      dropTarget.value = null;
      const key = keys.get(event.key);
      const targetKey = event.overKey === null ? undefined : keys.get(event.overKey);
      if (key === undefined || targetKey === undefined || event.position === null) return;
      move({
        key,
        targetKey,
        position: event.position,
        source: event.pointerType === "keyboard" ? "keyboard" : "pointer",
        originalEvent: event.originalEvent,
      });
    },
    onSortCancel() {
      dropTarget.value = null;
    },
  });

  const registerItem: TreeReorderController<K>["registerItem"] = (input) => {
    const id = String(input.key);
    keys.set(id, input.key);
    const registration = sortable.registerItem({
      key: id,
      element: input.element,
      label: input.label,
      isDisabled: input.disabled,
    });
    const { itemProps } = registration;
    const result: TreeReorderItemRegistration = {
      isDragging: registration.isDragging,
      pointerProps: Object.freeze({
        onDragstart: itemProps.onDragstart,
        onMousedown: itemProps.onMousedown,
        onPointerdown: itemProps.onPointerdown,
        onTouchstart: itemProps.onTouchstart,
      }),
      dispose: () => {
        registration.dispose();
        if (keys.get(id) === input.key) keys.delete(id);
      },
    };
    return Object.freeze(result);
  };

  return Object.freeze({
    isDragging: sortable.isSorting,
    dropTarget: shallowReadonly(dropTarget),
    registerItem,
    move,
    cancel: () => sortable.cancel(),
  });
}
