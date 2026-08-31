/** Compile-only assertions for the public drag-and-drop contract. */

import { ref } from "vue";
import type { HTMLAttributes, ShallowRef } from "vue";

import {
  createDragAndDrop,
  readDragTransfer,
  writeDragTransfer,
  type DragPayload,
  type DragPointerType,
  type DragSourceProps,
  type DropEdge,
  type DropIndicatorState,
} from "./drag-and-drop.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const pointerTypes: readonly DragPointerType[] = [
  "keyboard",
  "mouse",
  "pen",
  "pointer",
  "touch",
];
// @ts-expect-error virtual activation cannot own a drag session.
export const invalidPointerType: DragPointerType = "virtual";

export const edges: readonly DropEdge[] = ["bottom", "inside", "left", "right", "top"];
// @ts-expect-error the drop edge union is closed.
export const invalidEdge: DropEdge = "center";

interface TaskData {
  readonly id: number;
}

const disabled = ref(false);
export const controller = createDragAndDrop<TaskData>({
  isDisabled: disabled,
  startDistance: () => 8,
  onDragEnd(event) {
    if (event.targetKey !== null) {
      const id: number | undefined = event.payload?.data.id;
      void id;
    }
  },
});

export const source = controller.registerSource({
  key: "card",
  payload: { kind: "task", data: { id: 1 } },
});
// @ts-expect-error payloads are typed against the controller's data parameter.
controller.registerSource({ key: "bad", payload: { kind: "task", data: { id: "1" } } });

export const target = controller.registerTarget({
  key: "zone",
  element: () => null,
  accepts: (payload: DragPayload<TaskData> | null) => payload !== null,
  onDrop(event) {
    const data: TaskData | undefined = event.payload?.data;
    void data;
  },
});

type _DraggingIsReadonly = Expect<
  Equal<typeof controller.isDragging, Readonly<ShallowRef<boolean>>>
>;
type _IndicatorIsNullable = Expect<
  Equal<typeof controller.indicator, Readonly<ShallowRef<DropIndicatorState | null>>>
>;
type _SourcePropsAreExact = Expect<Equal<typeof source.sourceProps, Readonly<DragSourceProps>>>;

export const vueAttributes: HTMLAttributes = source.sourceProps;
// @ts-expect-error consumers cannot mutate readonly reactive state.
controller.isDragging.value = true;
// @ts-expect-error the start distance must resolve to a number.
createDragAndDrop({ startDistance: "8" });
// @ts-expect-error target registration requires an element accessor.
controller.registerTarget({ key: "floating" });

export const restored = readDragTransfer<TaskData>(new DataTransfer());
type _RestoredIsTyped = Expect<Equal<typeof restored, DragPayload<TaskData> | null>>;
// @ts-expect-error transfer payloads always carry a kind discriminant.
writeDragTransfer(new DataTransfer(), { data: 1 });
