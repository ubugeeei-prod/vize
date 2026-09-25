/** Compile-only assertions for the public Hotspot contract. */

import {
  Hotspot,
  HotspotArea,
  HotspotContent,
  HotspotImage,
  HotspotMarker,
  HotspotRoot,
  hotspotShapeContains,
  nextHotspotInDirection,
  type HotspotActiveChangeSource,
  type HotspotAreaExpose,
  type HotspotDefinition,
  type HotspotDirection,
  type HotspotImageExpose,
  type HotspotMarkerExpose,
  type HotspotMarkerSlotState,
  type HotspotMarkerState,
  type HotspotRootExpose,
  type HotspotShape,
  type HotspotSlotState,
} from "./hotspot.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Product {
  readonly sku: string;
}

declare const root: HotspotRootExpose;
declare const marker: HotspotMarkerExpose;
declare const area: HotspotAreaExpose;
declare const image: HotspotImageExpose;
declare const slot: HotspotSlotState<Product>;

type _MarkerState = Expect<Equal<HotspotMarkerState, "closed" | "disabled" | "open">>;
type _Source = Expect<Equal<HotspotActiveChangeSource, "api" | "area" | "dismiss" | "marker">>;
type _Direction = Expect<Equal<HotspotDirection, "down" | "left" | "right" | "up">>;
type _ShapeKinds = Expect<Equal<HotspotShape["type"], "circle" | "polygon" | "rect">>;
type _SlotDataFlows = Expect<Equal<(typeof slot.hotspots)[number]["data"], Product | undefined>>;
type _MarkerSlot = Expect<
  Equal<
    HotspotMarkerSlotState,
    {
      readonly id: string;
      readonly open: boolean;
      readonly disabled: boolean;
      readonly state: HotspotMarkerState;
    }
  >
>;
type _Active = Expect<Equal<typeof root.active, string | null>>;
type _HitTest = Expect<Equal<ReturnType<typeof root.hitTest>, readonly string[]>>;
type _MarkerElement = Expect<Equal<typeof marker.element, HTMLDivElement | null>>;
type _AreaElement = Expect<Equal<typeof area.element, SVGSVGElement | null>>;
type _ImageElement = Expect<Equal<typeof image.element, HTMLImageElement | null>>;
type _Alias = Expect<Equal<typeof Hotspot, typeof HotspotRoot>>;
type _Contains = Expect<Equal<ReturnType<typeof hotspotShapeContains>, boolean>>;
type _Next = Expect<Equal<ReturnType<typeof nextHotspotInDirection>, string | null>>;

const _definition: HotspotDefinition<Product> = {
  id: "a",
  x: 1,
  y: 2,
  label: "A",
  data: { sku: "1" },
};
void _definition;
void HotspotArea;
void HotspotContent;
void HotspotImage;
void HotspotMarker;

// @ts-expect-error payloads keep their declared type.
const _badData: HotspotDefinition<Product> = { id: "a", x: 1, y: 2, label: "A", data: { sku: 1 } };
// @ts-expect-error shapes are a closed union.
const _badShape: HotspotShape = { type: "ellipse", cx: 1, cy: 1, rx: 1, ry: 1 };
// @ts-expect-error exposed state is read-only.
root.active = "a";
