import { computed, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { createInputMask } from "./input-mask-engine.ts";
import type { InputMask, InputMaskResult, InputMaskTokens } from "./input-mask-engine.ts";

/** Which representation the model value uses. */
export type InputMaskValueFormat = "masked" | "raw";

/** Options accepted by {@link useInputMask}. */
export interface UseInputMaskOptions {
  /** Mask pattern, for example `"(999) 999-9999"`. */
  readonly mask: MaybeRefOrGetter<string>;

  /**
   * Extra or overriding tokens merged over the defaults.
   *
   * @default undefined
   */
  readonly tokens?: MaybeRefOrGetter<InputMaskTokens | undefined>;

  /**
   * Placeholder character for unfilled slots when `lazy` is `false`.
   *
   * @default "_"
   */
  readonly placeholderChar?: MaybeRefOrGetter<string | undefined>;

  /**
   * Hide unfilled slots.
   *
   * @default true
   */
  readonly lazy?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Append literals right after the last filled slot while typing.
   *
   * @default false
   */
  readonly eager?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Controlled model value in `valueFormat`. `undefined` selects uncontrolled use.
   *
   * @default undefined
   */
  readonly value?: MaybeRefOrGetter<string | undefined>;

  /**
   * Initial uncontrolled model value in `valueFormat`.
   *
   * @default ""
   */
  readonly defaultValue?: MaybeRefOrGetter<string | undefined>;

  /**
   * Representation of the model value.
   *
   * @default "raw"
   */
  readonly valueFormat?: MaybeRefOrGetter<InputMaskValueFormat | undefined>;

  /**
   * Called when the model value changes, with the full conform result.
   *
   * @default undefined
   */
  readonly onChange?: (value: string, result: InputMaskResult) => void;

  /**
   * Called when the last slot is filled.
   *
   * @default undefined
   */
  readonly onComplete?: (result: InputMaskResult) => void;
}

/** Reactive masked-input controller returned by {@link useInputMask}. */
export interface InputMaskController {
  /** Compiled mask. */
  readonly mask: ComputedRef<InputMask>;

  /** Current conform result. */
  readonly result: ComputedRef<InputMaskResult>;

  /** Displayed text. */
  readonly masked: ComputedRef<string>;

  /** Characters accepted into slots. */
  readonly raw: ComputedRef<string>;

  /** Whether every slot is filled. */
  readonly complete: ComputedRef<boolean>;

  /** Model value in the configured `valueFormat`. */
  readonly value: ComputedRef<string>;

  /** Suggested `inputmode`: `"numeric"` when every slot is a digit. */
  readonly inputMode: ComputedRef<"numeric" | "text">;

  /** Conform typed or pasted text from a native `input` event and restore the caret. */
  readonly handleInput: (event: Event) => void;

  /** Conform arbitrary text and store it; returns whether the model changed. */
  readonly setValue: (text: string) => boolean;

  /** Restore the default value; returns whether the model changed. */
  readonly reset: () => boolean;
}

function inputTypeOf(event: Event): string {
  return "inputType" in event && typeof event.inputType === "string" ? event.inputType : "";
}

/**
 * Drive any native text input with a pattern mask.
 *
 * SSR-safe and instance-free: it only touches the DOM inside `handleInput`,
 * so it can run in components, other composables, or tests.
 */
export function useInputMask(options: UseInputMaskOptions): InputMaskController {
  const mask = computed(() =>
    createInputMask(toValue(options.mask), {
      tokens: toValue(options.tokens),
      placeholderChar: toValue(options.placeholderChar),
      lazy: toValue(options.lazy),
      eager: toValue(options.eager),
    }),
  );
  const format = computed<InputMaskValueFormat>(() => toValue(options.valueFormat) ?? "raw");
  const toResult = (value: string): InputMaskResult =>
    format.value === "raw" ? mask.value.fromRaw(value) : mask.value.conform(value);
  const toModel = (result: InputMaskResult): string =>
    format.value === "raw" ? result.raw : result.masked;
  const state = useControllableState<string>({
    value: () => (options.value === undefined ? undefined : toValue(options.value)),
    defaultValue: () => toValue(options.defaultValue) ?? "",
    onChange: (value) => options.onChange?.(value, toResult(value)),
  });
  const result = computed(() => toResult(state.value.value));

  function commit(next: InputMaskResult): boolean {
    const wasComplete = result.value.complete;
    const changed = state.set(toModel(next));
    if (next.complete && !wasComplete) options.onComplete?.(next);
    return changed;
  }

  function handleInput(event: Event): void {
    const element = event.target;
    if (!(element instanceof HTMLInputElement)) return;
    const text = element.value;
    const caret = element.selectionStart ?? text.length;
    const previous = result.value;
    let next = mask.value.conform(text);
    let rawBeforeCaret = mask.value.conform(text.slice(0, caret)).raw.length;
    const inputType = inputTypeOf(event);
    // Deleting only a literal would be undone by re-masking, so remove the
    // adjacent slot character instead (Backspace deletes before, Delete after).
    if (
      inputType.startsWith("delete") &&
      next.raw === previous.raw &&
      text.length < previous.masked.length
    ) {
      const index = inputType === "deleteContentForward" ? rawBeforeCaret : rawBeforeCaret - 1;
      if (index >= 0 && index < previous.raw.length) {
        next = mask.value.fromRaw(previous.raw.slice(0, index) + previous.raw.slice(index + 1));
        rawBeforeCaret = index;
      }
    }
    commit(next);
    // Keep the native value on the conformed text even when a controlled
    // parent rejects the change; the next render reconciles the value prop.
    const shown = result.value.masked;
    element.value = shown;
    if (element.ownerDocument.activeElement === element) {
      const position = Math.min(mask.value.caretForRawCount(rawBeforeCaret), shown.length);
      element.setSelectionRange(position, position);
    }
  }

  return {
    mask,
    result,
    masked: computed(() => result.value.masked),
    raw: computed(() => result.value.raw),
    complete: computed(() => result.value.complete),
    value: state.value,
    inputMode: computed(() => (mask.value.numeric ? "numeric" : "text")),
    handleInput,
    setValue: (text: string) => commit(mask.value.conform(text)),
    reset: state.reset,
  };
}
