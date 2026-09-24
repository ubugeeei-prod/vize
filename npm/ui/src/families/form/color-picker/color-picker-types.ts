import type { ColorChannel, ColorFormat, ColorSpace, ColorValue } from "./color-picker-color.ts";

/** Interaction state shared by the ColorPicker root and its parts. */
export type ColorPickerState = "disabled" | "interactive" | "readonly";

/** Layout axis of a ColorPickerChannelSlider. */
export type ColorPickerOrientation = "horizontal" | "vertical";

/** Reading direction used to map horizontal pointer and arrow-key input. */
export type ColorPickerDirection = "ltr" | "rtl";

/** State of the ColorPickerEyeDropper button. */
export type ColorPickerEyeDropperState = "idle" | "picking" | "unsupported";

/** How ColorPickerEyeDropper renders when the EyeDropper API is unavailable. */
export type ColorPickerEyeDropperFallback = "disable" | "hide";

/** Validity of the ColorPickerField draft text. */
export type ColorPickerFieldState = "invalid" | "valid";

/** Selection state of a ColorPickerSwatch. */
export type ColorPickerSwatchState = "checked" | "disabled" | "unchecked";

/** Where a color change originated. */
export type ColorPickerChangeSource =
  | "api"
  | "area"
  | "channel"
  | "eye-dropper"
  | "field"
  | "form-reset"
  | "swatch";

/** Detail passed with every ColorPickerRoot `change` emit. */
export interface ColorPickerChangeDetail {
  /** Normalized color after the change. */
  readonly color: ColorValue;

  /** Color before the change. */
  readonly previousColor: ColorValue;

  /** Part that requested the change. */
  readonly source: ColorPickerChangeSource;

  /** Originating DOM event, when the change was user-driven. */
  readonly nativeEvent: Event | null;
}

/** State exposed to the ColorPickerRoot default slot. */
export interface ColorPickerSlotState {
  /** Current value serialized in the root `format`. */
  readonly value: string;

  /** Current normalized color. */
  readonly color: ColorValue;

  /** Serialization used for `value`, the hidden form input, and `update:modelValue`. */
  readonly format: ColorFormat;

  /** Whether every part suppresses interaction and focus. */
  readonly disabled: boolean;

  /** Whether parts stay focusable but reject edits. */
  readonly readOnly: boolean;

  /** Stable state token for styling and tests. */
  readonly state: ColorPickerState;
}

/** State exposed to the ColorPickerArea default slot (rendered inside the thumb). */
export interface ColorPickerAreaSlotState {
  /** Current normalized color. */
  readonly color: ColorValue;

  /** Channel mapped to the horizontal axis. */
  readonly xChannel: ColorChannel;

  /** Channel mapped to the vertical axis. */
  readonly yChannel: ColorChannel;

  /** Horizontal channel value. */
  readonly xValue: number;

  /** Vertical channel value. */
  readonly yValue: number;

  /** Horizontal thumb position as a percentage of the area width. */
  readonly xPercent: number;

  /** Vertical thumb position as a percentage of the area height, from the top. */
  readonly yPercent: number;

  /** Whether a pointer drag is in progress. */
  readonly dragging: boolean;

  /** Stable state token inherited from the root. */
  readonly state: ColorPickerState;
}

/** State exposed to the ColorPickerChannelSlider default slot (rendered inside the thumb). */
export interface ColorPickerChannelSliderSlotState {
  /** Edited channel. */
  readonly channel: ColorChannel;

  /** Color space the channel is read in. */
  readonly space: ColorSpace;

  /** Channel value snapped to its step. */
  readonly value: number;

  /** Minimum channel value. */
  readonly min: number;

  /** Maximum channel value. */
  readonly max: number;

  /** Thumb position as a percentage of the track, in reading direction. */
  readonly percent: number;

  /** Layout axis. */
  readonly orientation: ColorPickerOrientation;

  /** Current normalized color. */
  readonly color: ColorValue;

  /** Whether a pointer drag is in progress. */
  readonly dragging: boolean;

  /** Stable state token inherited from the root. */
  readonly state: ColorPickerState;
}

/** State exposed to ColorPickerSwatch slots. */
export interface ColorPickerSwatchSlotState {
  /** Swatch color string exactly as supplied. */
  readonly value: string;

  /** Parsed swatch color. */
  readonly color: ColorValue;

  /** Whether the swatch matches the root color. */
  readonly checked: boolean;

  /** Whether the swatch or root suppresses selection. */
  readonly disabled: boolean;

  /** Stable state token for styling and tests. */
  readonly state: ColorPickerSwatchState;
}

/** State exposed to the ColorPickerEyeDropper default slot. */
export interface ColorPickerEyeDropperSlotState {
  /** Whether the browser provides the EyeDropper API (always `false` during SSR). */
  readonly supported: boolean;

  /** Whether the system picker is open. */
  readonly picking: boolean;

  /** Stable state token for styling and tests. */
  readonly state: ColorPickerEyeDropperState;
}

/** Public instance exposed by ColorPickerRoot. */
export interface ColorPickerRootExpose extends ColorPickerSlotState {
  /** Rendered root element. */
  readonly element: HTMLDivElement | null;

  /** Root-owned deterministic base id. */
  readonly id: string;

  /** Request a color (object or CSS string) and report whether it changed. Invalid strings are ignored. */
  readonly setColor: (color: ColorValue | string) => boolean;

  /** Restore the default value and report whether it changed. */
  readonly reset: () => boolean;
}

/** Public instance exposed by ColorPickerArea. */
export interface ColorPickerAreaExpose {
  /** Rendered area element. */
  readonly element: HTMLDivElement | null;

  /** Rendered focusable thumb element. */
  readonly thumb: HTMLDivElement | null;

  /** Move focus to the thumb. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by ColorPickerChannelSlider. */
export interface ColorPickerChannelSliderExpose {
  /** Rendered track element. */
  readonly element: HTMLDivElement | null;

  /** Rendered focusable thumb element. */
  readonly thumb: HTMLDivElement | null;

  /** Current snapped channel value. */
  readonly value: number;

  /** Move focus to the thumb. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by ColorPickerField. */
export interface ColorPickerFieldExpose {
  /** Rendered native text input. */
  readonly element: HTMLInputElement | null;

  /** Current draft text. */
  readonly draft: string;

  /** Validity of the draft text. */
  readonly state: ColorPickerFieldState;

  /** Parse and commit the draft; invalid drafts revert. Returns whether it parsed. */
  readonly commit: () => boolean;

  /** Discard the draft and show the current color. */
  readonly revert: () => void;

  /** Move focus to the input. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by ColorPickerSwatchGroup. */
export interface ColorPickerSwatchGroupExpose {
  /** Rendered radiogroup element. */
  readonly element: HTMLDivElement | null;

  /** Move focus to the checked, active, or first enabled swatch. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by ColorPickerSwatch. */
export interface ColorPickerSwatchExpose extends ColorPickerSwatchSlotState {
  /** Rendered swatch element. */
  readonly element: HTMLDivElement | null;

  /** Move focus to the swatch. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by ColorPickerEyeDropper. */
export interface ColorPickerEyeDropperExpose extends ColorPickerEyeDropperSlotState {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Open the system picker. Resolves with the picked color, or `null` when cancelled or unsupported. */
  readonly open: () => Promise<ColorValue | null>;
}
