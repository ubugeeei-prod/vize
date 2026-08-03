import { readonly, ref, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref, WatchHandle } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Options for {@link useEventListener}. */
export interface UseEventListenerOptions {
  /**
   * Invoke the listener during the capture phase.
   *
   * @default false
   */
  readonly capture?: boolean;

  /**
   * Stop listening after the first event.
   *
   * @default false
   */
  readonly once?: boolean;

  /**
   * Declare that the listener does not cancel the event's default action.
   *
   * @default false
   */
  readonly passive?: boolean;

  /**
   * Stop listening when this signal is aborted. An already-aborted signal
   * prevents listening from ever starting.
   *
   * @default undefined
   */
  readonly signal?: AbortSignal;

  /**
   * Start listening during composable creation.
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * Reactive target update timing.
   *
   * @default "pre"
   */
  readonly flush?: "pre" | "post" | "sync";
}

/** Reactive controls returned by {@link useEventListener}. */
export interface EventListenerControls {
  /** Whether a concrete target currently owns the listener. */
  readonly isListening: Readonly<Ref<boolean>>;

  /**
   * Begin listening.
   *
   * @returns Whether a new reactive watcher was started. `false` while the
   * watcher is already active (including a null-target watcher with no
   * listener attached), after the abort signal has fired, and after the
   * owning reactive scope has been disposed.
   */
  readonly start: () => boolean;

  /** Stop listening. Repeated calls are safe. */
  readonly stop: () => void;
}

/**
 * Attach an event listener to a reactive target and clean it up with the
 * current reactive scope. Missing targets are valid during server rendering:
 * a `null`/`undefined` target keeps the listener detached until a concrete
 * target appears, so no browser globals are required.
 *
 * The listener is re-attached whenever the reactive target changes and is
 * removed when the owning reactive scope stops, when
 * {@link EventListenerControls.stop} is called, or when the abort signal
 * fires. Outside an active scope, teardown ownership stays with the caller,
 * who must call `stop` explicitly. Errors thrown by a custom target's
 * add/remove methods propagate to the active watcher run.
 *
 * @param target Reactive event target.
 * @param event Event name.
 * @param listener Typed event listener.
 * @param options Listener lifecycle and scheduling options.
 * @default options {}
 * @returns Controls to observe and change the listening state.
 */
export function useEventListener<Key extends keyof WindowEventMap>(
  target: MaybeRefOrGetter<Window | null | undefined>,
  event: Key,
  listener: (event: WindowEventMap[Key]) => void,
  options?: UseEventListenerOptions,
): EventListenerControls;
export function useEventListener<Key extends keyof DocumentEventMap>(
  target: MaybeRefOrGetter<Document | null | undefined>,
  event: Key,
  listener: (event: DocumentEventMap[Key]) => void,
  options?: UseEventListenerOptions,
): EventListenerControls;
export function useEventListener<Key extends keyof HTMLElementEventMap>(
  target: MaybeRefOrGetter<HTMLElement | null | undefined>,
  event: Key,
  listener: (event: HTMLElementEventMap[Key]) => void,
  options?: UseEventListenerOptions,
): EventListenerControls;
export function useEventListener(
  target: MaybeRefOrGetter<EventTarget | null | undefined>,
  event: string,
  listener: EventListener,
  options?: UseEventListenerOptions,
): EventListenerControls;
export function useEventListener(
  target: MaybeRefOrGetter<EventTarget | null | undefined>,
  event: string,
  listener: EventListener,
  options: UseEventListenerOptions = {},
): EventListenerControls {
  const {
    capture = false,
    once = false,
    passive = false,
    signal,
    immediate = true,
    flush = "pre",
  } = options;
  const isListening = ref(false);
  const eventOptions: AddEventListenerOptions = {
    capture,
    passive,
    ...(signal ? { signal } : {}),
  };
  let disposed = false;
  let stopWatch: WatchHandle | undefined;

  const stop = (): void => {
    stopWatch?.stop();
    stopWatch = undefined;
    isListening.value = false;
  };
  const start = (): boolean => {
    if (disposed || stopWatch || signal?.aborted) return false;

    stopWatch = watch(
      () => toValue(target),
      (next, _previous, onCleanup) => {
        isListening.value = false;
        if (!next || signal?.aborted) return;

        const invoke: EventListener = (nativeEvent) => {
          if (once) stop();
          listener(nativeEvent);
        };
        const onAbort = (): void => stop();
        next.addEventListener(event, invoke, eventOptions);
        signal?.addEventListener("abort", onAbort, { once: true });
        isListening.value = true;

        onCleanup(() => {
          next.removeEventListener(event, invoke, capture);
          signal?.removeEventListener("abort", onAbort);
          isListening.value = false;
        });
      },
      { flush, immediate: true },
    );
    return true;
  };

  tryOnScopeDispose(() => {
    disposed = true;
    stop();
  });
  if (immediate) start();

  return { isListening: readonly(isListening), start, stop };
}
