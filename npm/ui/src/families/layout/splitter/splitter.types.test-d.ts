/** Compile-only assertions for the public Splitter contract. */

import type { Ref } from "vue";

import {
  SplitterGroup,
  SplitterHandle,
  SplitterPanel,
  useSplitterPersistence,
  type SplitterGroupExpose,
  type SplitterGroupSlotState,
  type SplitterGroupState,
  type SplitterHandleSlotState,
  type SplitterHandleState,
  type SplitterLayout,
  type SplitterOrientation,
  type SplitterPanelExpose,
  type SplitterPanelSlotState,
  type SplitterPanelState,
  type SplitterPersistedLayout,
  type SplitterResizeReason,
  type SplitterStorage,
} from "./splitter.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _LayoutIsReadonlyNumbers = Expect<Equal<SplitterLayout, readonly number[]>>;
type _OrientationIsLiteral = Expect<Equal<SplitterOrientation, "horizontal" | "vertical">>;
type _ReasonIsLiteral = Expect<
  Equal<SplitterResizeReason, "keyboard" | "pointer" | "programmatic">
>;
type _GroupStateIsLiteral = Expect<Equal<SplitterGroupState, "disabled" | "idle" | "resizing">>;
type _PanelStateIsLiteral = Expect<Equal<SplitterPanelState, "collapsed" | "expanded">>;
type _HandleStateIsLiteral = Expect<Equal<SplitterHandleState, "disabled" | "dragging" | "idle">>;
type _PersistedIsWritableRef = Expect<
  Equal<SplitterPersistedLayout, Ref<SplitterLayout | undefined>>
>;
type _PersistenceReturns = Expect<
  Equal<ReturnType<typeof useSplitterPersistence>, SplitterPersistedLayout>
>;
type _GroupSlotLayout = Expect<Equal<SplitterGroupSlotState["layout"], SplitterLayout>>;
type _PanelSlotSize = Expect<Equal<SplitterPanelSlotState["size"], number>>;
type _HandleSlotValue = Expect<Equal<SplitterHandleSlotState["value"], number>>;
type _GroupElement = Expect<Equal<SplitterGroupExpose["element"], HTMLDivElement | null>>;
type _PanelCollapse = Expect<Equal<SplitterPanelExpose["collapse"], () => boolean>>;

const groupProps: InstanceType<typeof SplitterGroup>["$props"] = {
  defaultLayout: [30, 70],
  dir: "rtl",
  disabled: false,
  id: "editor",
  keyboardStep: 5,
  layout: [30, 70],
  orientation: "vertical",
  "onUpdate:layout": (layout: SplitterLayout) => layout,
  onResize: (layout: SplitterLayout, reason: SplitterResizeReason) => [layout, reason],
};
const panelProps: InstanceType<typeof SplitterPanel>["$props"] = {
  collapsedSize: 4,
  collapsible: true,
  defaultSize: 30,
  id: "sidebar",
  maxSize: 60,
  minSize: 20,
  onCollapse: () => undefined,
  order: 0,
};
const handleProps: InstanceType<typeof SplitterHandle>["$props"] = {
  ariaLabel: "Resize",
  disabled: false,
};
const storage: SplitterStorage = { getItem: () => null, setItem: () => undefined };

// @ts-expect-error orientation is a closed union.
const badOrientation: SplitterOrientation = "diagonal";

const badGroup: InstanceType<typeof SplitterGroup>["$props"] = {
  // @ts-expect-error layouts are numeric percentages.
  layout: ["30%", "70%"],
};

const badPanel: InstanceType<typeof SplitterPanel>["$props"] = {
  // @ts-expect-error sizes are numbers.
  minSize: "20",
};

void badGroup;
void badOrientation;
void badPanel;
void groupProps;
void handleProps;
void panelProps;
void storage;
