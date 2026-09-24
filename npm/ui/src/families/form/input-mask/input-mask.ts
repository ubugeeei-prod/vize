/** Native text input driven by a typed pattern mask (phone numbers, dates, card numbers, codes). */
export { default as MaskedInput } from "./masked-input.vue";
export {
  INPUT_MASK_DEFAULT_TOKENS,
  createInputMask,
  defineInputMaskTokens,
} from "./input-mask-engine.ts";
export type {
  InputMask,
  InputMaskDefaultTokenKey,
  InputMaskOptions,
  InputMaskResult,
  InputMaskToken,
  InputMaskTokens,
} from "./input-mask-engine.ts";
export { useInputMask } from "./input-mask-runtime.ts";
export type {
  InputMaskController,
  InputMaskValueFormat,
  UseInputMaskOptions,
} from "./input-mask-runtime.ts";
export type {
  MaskedInputAriaInvalid,
  MaskedInputEmits,
  MaskedInputExpose,
  MaskedInputProps,
  MaskedInputState,
} from "./input-mask-types.ts";
