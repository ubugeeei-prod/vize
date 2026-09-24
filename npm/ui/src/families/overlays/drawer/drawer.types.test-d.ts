/** Compile-only assertions for the public Drawer contract. */

import type {
  DrawerContentExpose,
  DrawerDismissReason,
  DrawerDragEndEvent,
  DrawerDragOutcome,
  DrawerHandleExpose,
  DrawerReleaseResult,
  DrawerRootExpose,
  DrawerSide,
  DrawerSlotState,
  DrawerSnapPoint,
  DrawerState,
} from "./drawer.ts";
import {
  Drawer,
  DrawerClose,
  DrawerContent,
  DrawerHandle,
  DrawerRoot,
  DrawerTrigger,
  resolveDrawerRelease,
} from "./drawer.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: DrawerRootExpose;
declare const content: DrawerContentExpose;
declare const handle: DrawerHandleExpose;
declare const slot: DrawerSlotState;
declare const dragEnd: DrawerDragEndEvent;

type _SideIsLiteral = Expect<Equal<DrawerSide, "bottom" | "left" | "right" | "top">>;
type _StateIsLiteral = Expect<Equal<DrawerState, "closed" | "open">>;
type _SnapPoint = Expect<Equal<DrawerSnapPoint, number | `${number}px`>>;
type _Outcome = Expect<Equal<DrawerDragOutcome, "cancel" | "dismiss" | "snap">>;
type _Reason = Expect<
  Equal<DrawerDismissReason, "backdrop" | "escape-key" | "pointer-down-outside">
>;
type _ContentElement = Expect<Equal<typeof content.element, HTMLDialogElement | null>>;
type _HandleElement = Expect<Equal<typeof handle.element, HTMLButtonElement | null>>;
type _ActiveSnap = Expect<Equal<typeof slot.activeSnapPoint, DrawerSnapPoint | null>>;
type _DragSnap = Expect<Equal<typeof dragEnd.snapPoint, DrawerSnapPoint | null>>;
type _SnapTo = Expect<
  Equal<typeof root.snapTo, (snapPoint: DrawerSnapPoint, event?: Event | null) => boolean>
>;

const release: DrawerReleaseResult = resolveDrawerRelease({
  closeThreshold: 0.25,
  dismissible: true,
  distance: 10,
  size: 400,
  snapPoints: [0.5, "120px"],
  startOffset: 0,
  velocity: 0,
  velocityThreshold: 0.5,
});
if (release.outcome === "snap") {
  type _Narrowed = Expect<Equal<typeof release.snapPoint, DrawerSnapPoint | null>>;
}

const rootProps: InstanceType<typeof DrawerRoot>["$props"] = {
  activeSnapPoint: "240px",
  closeThreshold: 0.3,
  defaultOpen: true,
  dismissible: false,
  modal: false,
  side: "left",
  snapPoints: [0.25, "240px", 1],
  "onUpdate:activeSnapPoint": (value: DrawerSnapPoint | null) => value,
};
const contentProps: InstanceType<typeof DrawerContent>["$props"] = {
  ariaLabelledby: null,
  closeOnBackdropPointerDown: false,
  dragFromContent: false,
};
const handleProps: InstanceType<typeof DrawerHandle>["$props"] = { ariaLabel: "Resize" };

root.snapTo(0.5);
root.openDrawer();
content.focusContent();
handle.focus();

// @ts-expect-error side is a closed edge union.
const badSide: DrawerSide = "center";

// @ts-expect-error snap points are fractions or px strings, not percentages.
const badSnap: DrawerSnapPoint = "50%";

// @ts-expect-error snapPoints must be DrawerSnapPoint values.
const badRootProps: InstanceType<typeof DrawerRoot>["$props"] = { snapPoints: ["half"] };

// @ts-expect-error dragFromContent is boolean-only.
const badContentProps: InstanceType<typeof DrawerContent>["$props"] = { dragFromContent: "yes" };

void Drawer;
void DrawerClose;
void DrawerTrigger;
void badContentProps;
void badRootProps;
void badSide;
void badSnap;
void contentProps;
void handleProps;
void rootProps;
