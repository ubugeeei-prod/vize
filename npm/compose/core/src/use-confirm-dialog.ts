import { shallowRef } from "vue";
import type { ShallowRef } from "vue";

import { createEventHook } from "./create-event-hook.ts";
import type { EventHookListener, EventHookSubscription } from "./create-event-hook.ts";

/** States of the {@link useConfirmDialog} machine. */
export type ConfirmDialogState = "idle" | "revealed";

/** Outcome a {@link ConfirmDialog.reveal} promise resolves with. */
export type ConfirmDialogResult<ConfirmData, CancelData> =
  | { readonly isCanceled: false; readonly data: ConfirmData }
  | { readonly isCanceled: true; readonly data: CancelData };

/** Arguments of a payload: none for `void`, one otherwise. */
export type ConfirmDialogPayload<Data> = [Data] extends [void] ? [] : [data: Data];

/** Dialog state machine returned by {@link useConfirmDialog}. */
export interface ConfirmDialog<RevealData, ConfirmData, CancelData> {
  /** `"revealed"` between `reveal()` and `confirm()`/`cancel()`. */
  readonly state: Readonly<ShallowRef<ConfirmDialogState>>;

  /** Shorthand for `state === "revealed"`. */
  readonly isRevealed: Readonly<ShallowRef<boolean>>;

  /**
   * Open the dialog. While already revealed, returns the pending promise
   * without re-firing `onReveal`.
   *
   * @returns Resolves when the dialog is confirmed or cancelled.
   */
  readonly reveal: (
    ...payload: ConfirmDialogPayload<RevealData>
  ) => Promise<ConfirmDialogResult<ConfirmData, CancelData>>;

  /**
   * Confirm the revealed dialog.
   *
   * @returns Whether the dialog was revealed.
   */
  readonly confirm: (...payload: ConfirmDialogPayload<ConfirmData>) => boolean;

  /**
   * Cancel the revealed dialog.
   *
   * @returns Whether the dialog was revealed.
   */
  readonly cancel: (...payload: ConfirmDialogPayload<CancelData>) => boolean;

  /** Listen for reveals. */
  readonly onReveal: (
    listener: EventHookListener<ConfirmDialogPayload<RevealData>>,
  ) => EventHookSubscription;

  /** Listen for confirmations. */
  readonly onConfirm: (
    listener: EventHookListener<ConfirmDialogPayload<ConfirmData>>,
  ) => EventHookSubscription;

  /** Listen for cancellations. */
  readonly onCancel: (
    listener: EventHookListener<ConfirmDialogPayload<CancelData>>,
  ) => EventHookSubscription;
}

/**
 * Promise-based confirm dialog state machine (`idle` → `revealed` →
 * `idle`), independent of any dialog markup.
 *
 * `reveal()` resolves with a discriminated result, so `if (!result.isCanceled)`
 * narrows `data` to the confirm payload. Payload types are declared as type
 * parameters; `void` payloads take no argument. Pair it with a native
 * `<dialog>` or any headless dialog. Synchronous state: SSR-safe; listeners
 * registered inside a scope are removed with it.
 *
 * @example
 * ```ts
 * const dialog = useConfirmDialog<{ name: string }, "keep" | "delete">();
 * const result = await dialog.reveal({ name: file.name });
 * if (!result.isCanceled && result.data === "delete") remove(file);
 * ```
 *
 * @returns The dialog machine.
 */
export function useConfirmDialog<
  RevealData = void,
  ConfirmData = void,
  CancelData = void,
>(): ConfirmDialog<RevealData, ConfirmData, CancelData>;
export function useConfirmDialog(): ConfirmDialog<unknown, unknown, unknown> {
  const state = shallowRef<ConfirmDialogState>("idle");
  const isRevealed = shallowRef(false);
  const revealHook = createEventHook<[data?: unknown]>();
  const confirmHook = createEventHook<[data?: unknown]>();
  const cancelHook = createEventHook<[data?: unknown]>();
  let pending: Promise<ConfirmDialogResult<unknown, unknown>> | undefined;
  let settle: ((result: ConfirmDialogResult<unknown, unknown>) => void) | undefined;

  const close = (result: ConfirmDialogResult<unknown, unknown>): boolean => {
    const resolve = settle;
    if (resolve === undefined) return false;
    settle = undefined;
    pending = undefined;
    state.value = "idle";
    isRevealed.value = false;
    resolve(result);
    return true;
  };

  return {
    state,
    isRevealed,
    reveal: (...payload) => {
      if (pending !== undefined) return pending;
      pending = new Promise((resolve) => {
        settle = resolve;
      });
      state.value = "revealed";
      isRevealed.value = true;
      void revealHook.trigger(...payload);
      return pending;
    },
    confirm: (...payload) => {
      if (settle === undefined) return false;
      void confirmHook.trigger(...payload);
      return close({ isCanceled: false, data: payload[0] });
    },
    cancel: (...payload) => {
      if (settle === undefined) return false;
      void cancelHook.trigger(...payload);
      return close({ isCanceled: true, data: payload[0] });
    },
    onReveal: revealHook.on,
    onConfirm: confirmHook.on,
    onCancel: cancelHook.on,
  };
}
