import { computed, onScopeDispose, ref } from "vue";
import type { ComputedRef, Ref } from "vue";

import { numberFieldContext } from "./number-field-context.ts";
import type { NumberFieldContextValue } from "./number-field-context.ts";
import type {
  NumberFieldStepDirection,
  NumberFieldTriggerSlotState,
} from "./number-field-types.ts";

/** Behavior shared by NumberFieldIncrement and NumberFieldDecrement. */
export interface NumberFieldTriggerController {
  readonly context: NumberFieldContextValue;
  readonly disabled: ComputedRef<boolean>;
  readonly holding: Ref<boolean>;
  readonly slotState: ComputedRef<NumberFieldTriggerSlotState>;
  readonly onPointerdown: (event: PointerEvent) => void;
  readonly onPointerEnd: () => void;
  readonly onClick: (event: MouseEvent) => void;
}

/**
 * Step once on press, then repeat after `holdDelay` every `holdInterval`
 * milliseconds until the pointer is released, leaves, is canceled, or a bound
 * is reached. Clicks from assistive technology (no pointer press) step once.
 */
export function useNumberFieldTrigger(
  direction: NumberFieldStepDirection,
): NumberFieldTriggerController {
  const context = numberFieldContext.use();
  const holding = ref(false);
  const disabled = computed(() => !context.canStep(direction));
  let timer: ReturnType<typeof setTimeout> | undefined;
  let pressedByPointer = false;

  function stop(): void {
    if (timer !== undefined) clearTimeout(timer);
    timer = undefined;
    holding.value = false;
  }

  function schedule(delay: number): void {
    timer = setTimeout(() => {
      timer = undefined;
      if (!context.canStep(direction)) {
        stop();
        return;
      }
      holding.value = true;
      context.step(direction, "step", "trigger");
      schedule(context.holdInterval.value);
    }, delay);
  }

  function onPointerdown(event: PointerEvent): void {
    if (event.button !== 0 || disabled.value) return;
    // Keep focus (and the caret) in the spinbutton instead of the trigger.
    event.preventDefault();
    context.inputElement.value?.focus({ preventScroll: true });
    pressedByPointer = true;
    stop();
    context.step(direction, "step", "trigger");
    schedule(context.holdDelay.value);
  }

  function onClick(event: MouseEvent): void {
    if (pressedByPointer) {
      pressedByPointer = false;
      return;
    }
    if (disabled.value) return;
    event.preventDefault();
    context.step(direction, "step", "trigger");
  }

  onScopeDispose(stop);

  return {
    context,
    disabled,
    holding,
    slotState: computed(() => ({
      direction,
      disabled: disabled.value,
      holding: holding.value,
    })),
    onPointerdown,
    onPointerEnd: stop,
    onClick,
  };
}
