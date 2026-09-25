/** Compile-only assertions for the public PanZoom contract. */

import {
  PanZoom,
  PanZoomContent,
  PanZoomFit,
  PanZoomReset,
  PanZoomRoot,
  PanZoomStatus,
  PanZoomViewport,
  PanZoomZoomIn,
  PanZoomZoomOut,
  clampToBounds,
  pinchTransform,
  zoomAt,
  type PanZoomBounds,
  type PanZoomButtonExpose,
  type PanZoomChangeSource,
  type PanZoomContentExpose,
  type PanZoomMessages,
  type PanZoomPoint,
  type PanZoomRootExpose,
  type PanZoomSlotState,
  type PanZoomState,
  type PanZoomStatusExpose,
  type PanZoomTransform,
  type PanZoomViewportExpose,
  type PanZoomWheelMode,
} from "./pan-zoom.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: PanZoomRootExpose;
declare const viewport: PanZoomViewportExpose;
declare const content: PanZoomContentExpose;
declare const button: PanZoomButtonExpose;
declare const status: PanZoomStatusExpose;

type _TransformIsExact = Expect<
  Equal<PanZoomTransform, { readonly x: number; readonly y: number; readonly scale: number }>
>;
type _StateIsLiteral = Expect<Equal<PanZoomState, "disabled" | "idle" | "panning" | "pinching">>;
type _WheelModeIsLiteral = Expect<Equal<PanZoomWheelMode, "pan" | "zoom" | "zoom-with-ctrl">>;
type _SourceIsLiteral = Expect<
  Equal<
    PanZoomChangeSource,
    "api" | "button" | "double-click" | "keyboard" | "pinch" | "pointer" | "wheel"
  >
>;
type _BoundsAcceptRegions = Expect<
  Equal<
    Extract<PanZoomBounds, object>,
    { readonly x: number; readonly y: number; readonly width: number; readonly height: number }
  >
>;
type _SlotTransform = Expect<Equal<PanZoomSlotState["transform"], PanZoomTransform>>;
type _ZoomToSignature = Expect<
  Equal<typeof root.zoomTo, (scale: number, point?: PanZoomPoint) => boolean>
>;
type _ZoomLevelFormatter = Expect<
  Equal<PanZoomMessages["zoomLevel"], ((percent: number) => string) | undefined>
>;
type _Elements = Expect<
  Equal<
    [typeof viewport.element, typeof content.element, typeof button.element, typeof status.element],
    [HTMLDivElement | null, HTMLDivElement | null, HTMLButtonElement | null, HTMLDivElement | null]
  >
>;
type _HelpersReturnTransforms = Expect<
  Equal<
    [
      ReturnType<typeof zoomAt>,
      ReturnType<typeof clampToBounds>,
      ReturnType<typeof pinchTransform>,
    ],
    [PanZoomTransform, PanZoomTransform, PanZoomTransform]
  >
>;
type _AliasIsRoot = Expect<Equal<typeof PanZoom, typeof PanZoomRoot>>;

void PanZoomContent;
void PanZoomFit;
void PanZoomReset;
void PanZoomStatus;
void PanZoomViewport;
void PanZoomZoomIn;
void PanZoomZoomOut;

// @ts-expect-error bounds are a closed union or a rectangle.
const _unknownBounds: PanZoomBounds = "fill";
// @ts-expect-error transforms require a scale.
const _partialTransform: PanZoomTransform = { x: 0, y: 0 };
// @ts-expect-error exposed state is read-only.
root.scale = 2;
