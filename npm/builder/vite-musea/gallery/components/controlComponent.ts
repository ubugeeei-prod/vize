import TextControl from "./controls/TextControl.vue";
import NumberControl from "./controls/NumberControl.vue";
import BooleanControl from "./controls/BooleanControl.vue";
import RangeControl from "./controls/RangeControl.vue";
import SelectControl from "./controls/SelectControl.vue";
import ColorControl from "./controls/ColorControl.vue";
import ObjectControl from "./controls/ObjectControl.vue";
import UnsupportedPropControl from "./controls/UnsupportedPropControl.vue";

export function getControlComponent(kind: string, unsupported = false) {
  if (unsupported) return UnsupportedPropControl;
  switch (kind) {
    case "text":
      return TextControl;
    case "number":
      return NumberControl;
    case "boolean":
      return BooleanControl;
    case "range":
      return RangeControl;
    case "select":
    case "radio":
      return SelectControl;
    case "color":
      return ColorControl;
    case "object":
    case "array":
      return ObjectControl;
    default:
      return TextControl;
  }
}
