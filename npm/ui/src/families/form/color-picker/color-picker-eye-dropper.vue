<script setup lang="ts">
import { computed, onMounted, onScopeDispose, shallowRef, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { parseColor } from "./color-picker-color.ts";
import type { ColorValue } from "./color-picker-color.ts";
import { colorPickerContext } from "./color-picker-context.ts";
import {
  createEyeDropper,
  isEyeDropperAbort,
  isEyeDropperSupported,
  readEyeDropperResult,
} from "./color-picker-eye-dropper.ts";
import type {
  ColorPickerEyeDropperExpose,
  ColorPickerEyeDropperFallback,
  ColorPickerEyeDropperSlotState,
  ColorPickerEyeDropperState,
} from "./color-picker-types.ts";

const {
  unsupported = "disable",
  preserveAlpha = true,
  ariaLabel = undefined,
} = defineProps<{
  /**
   * Rendering when the EyeDropper API is unavailable (and always during SSR):
   * `"disable"` keeps a disabled button, `"hide"` adds the `hidden` attribute.
   *
   * @default "disable"
   */
  readonly unsupported?: ColorPickerEyeDropperFallback;

  /**
   * Keep the current alpha; the system picker always returns an opaque color.
   *
   * @default true
   */
  readonly preserveAlpha?: boolean;

  /**
   * Accessible name when the default slot has no text.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired with the picked color after it has been applied. */
  pick: [color: ColorValue, sRGBHex: string];

  /** Fired when the user dismisses the system picker. */
  cancel: [];

  /** Fired when the picker rejects for a reason other than dismissal. */
  error: [error: unknown];
}>();

defineSlots<{
  /** Button contents. Receives support and picking state. */
  default(props: ColorPickerEyeDropperSlotState): unknown;
}>();

const context = colorPickerContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const supported = shallowRef(false);
const picking = shallowRef(false);
let controller: AbortController | null = null;
const state = computed<ColorPickerEyeDropperState>(() => {
  if (!supported.value) return "unsupported";
  return picking.value ? "picking" : "idle";
});
const buttonDisabled = computed(
  () => !supported.value || context.disabled.value || context.readOnly.value,
);
const slotState = computed<ColorPickerEyeDropperSlotState>(() => ({
  picking: picking.value,
  state: state.value,
  supported: supported.value,
}));

onMounted(() => {
  supported.value = isEyeDropperSupported();
});

onScopeDispose(() => {
  controller?.abort();
  controller = null;
});

async function open(): Promise<ColorValue | null> {
  if (picking.value || !context.editable.value) return null;
  const dropper = createEyeDropper();
  if (dropper === null) return null;
  controller = typeof AbortController === "function" ? new AbortController() : null;
  picking.value = true;
  try {
    const result = await dropper.open(
      controller === null ? undefined : { signal: controller.signal },
    );
    const hex = readEyeDropperResult(result);
    const parsed = hex === null ? null : parseColor(hex);
    if (hex === null || parsed === null) {
      emit("error", new TypeError("VIZE_UI_COLOR_PICKER_EYE_DROPPER: malformed EyeDropper result"));
      return null;
    }
    const next = preserveAlpha ? { ...parsed, alpha: context.color.value.alpha } : parsed;
    context.setColor(next, "eye-dropper", null);
    context.commit("eye-dropper", null);
    emit("pick", next, hex);
    return next;
  } catch (error) {
    if (isEyeDropperAbort(error)) emit("cancel");
    else emit("error", error);
    return null;
  } finally {
    picking.value = false;
    controller = null;
  }
}

function onClick(): void {
  void open();
}

type ColorPickerEyeDropperSetupExpose = Omit<
  ColorPickerEyeDropperExpose,
  keyof ColorPickerEyeDropperSlotState | "element"
> & {
  readonly element: typeof element;
  readonly picking: typeof picking;
  readonly state: ComputedRef<ColorPickerEyeDropperState>;
  readonly supported: typeof supported;
};

const exposed = {
  element,
  open,
  picking,
  state,
  supported,
} satisfies ColorPickerEyeDropperSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    :id="context.getPartId('eye-dropper')"
    ref="element"
    type="button"
    :disabled="buttonDisabled"
    :hidden="!supported && unsupported === 'hide' ? true : undefined"
    :aria-label="ariaLabel"
    :aria-busy="picking ? 'true' : undefined"
    data-vize-ui="color-picker-eye-dropper"
    part="eye-dropper"
    :data-state="state"
    :data-supported="supported ? 'true' : 'false'"
    @click="onClick"
  >
    <slot v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
