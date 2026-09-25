/** Compile-only assertions for the public ColorPicker contract. */

import {
  ColorPicker,
  ColorPickerArea,
  ColorPickerChannelSlider,
  ColorPickerEyeDropper,
  ColorPickerField,
  ColorPickerRoot,
  ColorPickerSwatch,
  ColorPickerSwatchGroup,
  formatColor,
  getColorChannelValue,
  parseColor,
  setColorChannelValue,
  type ColorChannel,
  type ColorFormat,
  type ColorPickerAreaSlotState,
  type ColorPickerChangeDetail,
  type ColorPickerChangeSource,
  type ColorPickerChannelSliderSlotState,
  type ColorPickerEyeDropperExpose,
  type ColorPickerEyeDropperState,
  type ColorPickerFieldExpose,
  type ColorPickerFieldState,
  type ColorPickerRootExpose,
  type ColorPickerSlotState,
  type ColorPickerState,
  type ColorPickerSwatchExpose,
  type ColorPickerSwatchState,
  type ColorSpace,
  type ColorSpaceChannel,
  type ColorValue,
  type ColorPickerMessages,
} from "./color-picker.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: ColorPickerRootExpose;
declare const field: ColorPickerFieldExpose;
declare const swatch: ColorPickerSwatchExpose;
declare const dropper: ColorPickerEyeDropperExpose;
declare const color: ColorValue;
declare const detail: ColorPickerChangeDetail;
declare const areaSlot: ColorPickerAreaSlotState;
declare const sliderSlot: ColorPickerChannelSliderSlotState;

type _FormatIsLiteral = Expect<Equal<ColorFormat, "hex" | "hex8" | "hsb" | "hsl" | "rgb">>;
type _SpaceIsLiteral = Expect<Equal<ColorSpace, "hsb" | "hsl" | "rgb">>;
type _ChannelUnion = Expect<
  Equal<
    ColorChannel,
    "alpha" | "blue" | "brightness" | "green" | "hue" | "lightness" | "red" | "saturation"
  >
>;
type _HsbChannels = Expect<
  Equal<ColorSpaceChannel<"hsb">, "alpha" | "brightness" | "hue" | "saturation">
>;
type _HslChannels = Expect<
  Equal<ColorSpaceChannel<"hsl">, "alpha" | "hue" | "lightness" | "saturation">
>;
type _RgbChannels = Expect<Equal<ColorSpaceChannel<"rgb">, "alpha" | "blue" | "green" | "red">>;
type _StateIsLiteral = Expect<Equal<ColorPickerState, "disabled" | "interactive" | "readonly">>;
type _FieldStateIsLiteral = Expect<Equal<ColorPickerFieldState, "invalid" | "valid">>;
type _SwatchStateIsLiteral = Expect<
  Equal<ColorPickerSwatchState, "checked" | "disabled" | "unchecked">
>;
type _DropperStateIsLiteral = Expect<
  Equal<ColorPickerEyeDropperState, "idle" | "picking" | "unsupported">
>;
type _ColorValueIsReadonly = Expect<
  Equal<
    ColorValue,
    {
      readonly hue: number;
      readonly saturation: number;
      readonly brightness: number;
      readonly alpha: number;
    }
  >
>;
type _RootValueIsString = Expect<Equal<typeof root.value, string>>;
type _RootColorIsValue = Expect<Equal<typeof root.color, ColorValue>>;
type _RootElementIsDiv = Expect<Equal<typeof root.element, HTMLDivElement | null>>;
type _RootSlotIsSubset = Expect<
  Equal<Pick<ColorPickerRootExpose, keyof ColorPickerSlotState>, ColorPickerSlotState>
>;
type _FieldElementIsInput = Expect<Equal<typeof field.element, HTMLInputElement | null>>;
type _SwatchCheckedIsBoolean = Expect<Equal<typeof swatch.checked, boolean>>;
type _DropperOpenResolvesColor = Expect<
  Equal<ReturnType<typeof dropper.open>, Promise<ColorValue | null>>
>;
type _DetailSource = Expect<Equal<typeof detail.source, ColorPickerChangeSource>>;
type _AreaChannels = Expect<Equal<typeof areaSlot.xChannel, ColorChannel>>;
type _SliderSpace = Expect<Equal<typeof sliderSlot.space, ColorSpace>>;
type _ParseIsNullable = Expect<Equal<ReturnType<typeof parseColor>, ColorValue | null>>;

const formatted: string = formatColor(color, "hsl");
const hue: number = getColorChannelValue(color, "hue");
const red: number = getColorChannelValue(color, "red", "rgb");
const lighter: ColorValue = setColorChannelValue(color, "lightness", 60, "hsl");

// @ts-expect-error red is not an HSB channel; pass the "rgb" space to read it.
getColorChannelValue(color, "red");

// @ts-expect-error lightness is not an RGB channel.
setColorChannelValue(color, "lightness", 50, "rgb");

// @ts-expect-error format is a closed serialization contract.
const invalidFormat: ColorFormat = "cmyk";

// @ts-expect-error colors are immutable.
color.hue = 10;

const rootProps: InstanceType<typeof ColorPickerRoot>["$props"] = {
  defaultValue: "#ffffff",
  dir: "rtl",
  disabled: false,
  form: "settings",
  format: "rgb",
  id: "accent",
  modelValue: "hsl(210 50% 50%)",
  name: "accent",
  readOnly: false,
  required: true,
  onChange: (value: string, changeDetail: ColorPickerChangeDetail) => {
    void value;
    void changeDetail;
  },
  onCommit: (value: string, committed: ColorValue) => {
    void value;
    void committed;
  },
  "onUpdate:modelValue": (value: string) => value,
};
const areaProps: InstanceType<typeof ColorPickerArea>["$props"] = {
  ariaLabel: "Tone",
  space: "hsl",
  xChannel: "hue",
  yChannel: "lightness",
};
const sliderProps: InstanceType<typeof ColorPickerChannelSlider>["$props"] = {
  channel: "alpha",
  orientation: "vertical",
  space: "rgb",
  step: 0.05,
};
const fieldProps: InstanceType<typeof ColorPickerField>["$props"] = {
  ariaLabel: "Hex",
  format: "hex8",
  placeholder: "#000000",
  preserveAlpha: true,
  onInvalid: (draft: string, event: Event | null) => {
    void draft;
    void event;
  },
};
const swatchProps: InstanceType<typeof ColorPickerSwatch>["$props"] = {
  disabled: false,
  label: "Brand",
  order: 1,
  value: "#3366cc",
};
const groupProps: InstanceType<typeof ColorPickerSwatchGroup>["$props"] = {
  ariaLabel: "Presets",
  loop: false,
};
const dropperProps: InstanceType<typeof ColorPickerEyeDropper>["$props"] = {
  preserveAlpha: false,
  unsupported: "hide",
  onPick: (picked: ColorValue, hex: string) => {
    void picked;
    void hex;
  },
};

// @ts-expect-error channel is required on channel sliders.
const missingChannel: InstanceType<typeof ColorPickerChannelSlider>["$props"] = {};

const badChannel: InstanceType<typeof ColorPickerChannelSlider>["$props"] = {
  // @ts-expect-error channel names are a closed union.
  channel: "cyan",
};

const badFallback: InstanceType<typeof ColorPickerEyeDropper>["$props"] = {
  // @ts-expect-error unsupported fallback is either disable or hide.
  unsupported: "remove",
};

root.setColor("#fff");
root.setColor(color);
root.reset();
field.commit();
field.revert();
swatch.focus();
void dropper.open();

void ColorPicker;
void areaProps;
void badChannel;
void badFallback;
void dropperProps;
void fieldProps;
void formatted;
void groupProps;
void hue;
void invalidFormat;
void lighter;
void missingChannel;
void red;
void rootProps;
void sliderProps;
void swatchProps;

type _MessagesAreOptionalFunctions = Expect<
  Equal<
    ColorPickerMessages,
    {
      readonly channelLabel?: (channel: ColorChannel) => string;
      readonly channelValueText?: (channel: ColorChannel, value: number, label: string) => string;
      readonly areaLabel?: (xLabel: string, yLabel: string) => string;
    }
  >
>;
// @ts-expect-error messages must return strings.
const _badMessages: ColorPickerMessages = { channelLabel: () => 1 };
