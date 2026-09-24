import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { ColorFormat, ColorValue } from "./color-picker-color.ts";
import type {
  ColorPickerChangeSource,
  ColorPickerDirection,
  ColorPickerState,
} from "./color-picker-types.ts";

/** Shared state and actions for the ColorPicker compound components. */
export interface ColorPickerContextValue {
  readonly id: ComputedRef<string>;
  readonly color: ComputedRef<ColorValue>;
  readonly value: ComputedRef<string>;
  readonly format: ComputedRef<ColorFormat>;
  readonly dir: ComputedRef<ColorPickerDirection>;
  readonly disabled: ComputedRef<boolean>;
  readonly readOnly: ComputedRef<boolean>;
  readonly state: ComputedRef<ColorPickerState>;
  /** Whether edits are currently accepted (not disabled and not read-only). */
  readonly editable: ComputedRef<boolean>;
  readonly getPartId: (part: string) => string;
  /** Request a color and report whether the serialized value changed. */
  readonly setColor: (
    color: ColorValue,
    source: ColorPickerChangeSource,
    nativeEvent: Event | null,
  ) => boolean;
  /** Signal that a continuous interaction (drag, key press, pick) has finished. */
  readonly commit: (source: ColorPickerChangeSource, nativeEvent: Event | null) => void;
}

export const colorPickerContext = createContext<ColorPickerContextValue>("ColorPicker");
