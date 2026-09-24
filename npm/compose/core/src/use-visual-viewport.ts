import { computed, readonly, ref, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Structural `VisualViewport` observed by {@link useVisualViewport}. */
export interface VisualViewportHost extends EventTarget {
  /** Visible width in CSS pixels. */
  readonly width: number;
  /** Visible height in CSS pixels (shrinks when an on-screen keyboard overlays the page). */
  readonly height: number;
  /** Offset of the visual viewport from the layout viewport's top edge. */
  readonly offsetTop: number;
  /** Offset of the visual viewport from the layout viewport's left edge. */
  readonly offsetLeft: number;
  /** Pinch-zoom scale factor. */
  readonly scale: number;
}

/** Structural VirtualKeyboard API (`navigator.virtualKeyboard`). */
export interface VirtualKeyboardHost extends EventTarget {
  /** Whether the keyboard overlays content instead of resizing the viewport. */
  overlaysContent: boolean;
  /** Keyboard rectangle in CSS pixels. */
  readonly boundingRect: { readonly height: number };
}

/** Window-like capability read by {@link useVisualViewport}. */
export interface VisualViewportWindowHost {
  /** Layout viewport height. */
  readonly innerHeight: number;
  /** Visual viewport, when the engine supports it. */
  readonly visualViewport?: VisualViewportHost | null;
  /** Navigator that may expose the VirtualKeyboard API as `virtualKeyboard`. */
  readonly navigator?: object;
}

/** Options for {@link useVisualViewport}. */
export interface UseVisualViewportOptions {
  /**
   * Minimum occluded height (CSS px) reported as an open keyboard, filtering
   * browser toolbars that collapse while scrolling.
   *
   * @default 120
   */
  readonly keyboardThreshold?: number;

  /**
   * Opt in to `navigator.virtualKeyboard.overlaysContent = true` (Chromium) so
   * the keyboard height comes from the VirtualKeyboard API and the layout
   * viewport no longer resizes. The previous value is restored on cleanup.
   *
   * @default false
   */
  readonly overlaysContent?: boolean;

  /**
   * Reactive window capability for alternate runtimes and tests.
   *
   * @default globalThis.window when available
   */
  readonly host?: MaybeRefOrGetter<VisualViewportWindowHost | null | undefined>;
}

/** Reactive viewport geometry returned by {@link useVisualViewport}. */
export interface VisualViewportControls {
  /** Visual viewport width (0 on the server). */
  readonly width: Readonly<Ref<number>>;
  /** Visual viewport height (0 on the server). */
  readonly height: Readonly<Ref<number>>;
  /** Visual viewport top offset within the layout viewport. */
  readonly offsetTop: Readonly<Ref<number>>;
  /** Visual viewport left offset within the layout viewport. */
  readonly offsetLeft: Readonly<Ref<number>>;
  /** Pinch-zoom scale (1 on the server). */
  readonly scale: Readonly<Ref<number>>;
  /** Height of the bottom area hidden by an on-screen keyboard, in CSS px. */
  readonly keyboardHeight: Readonly<Ref<number>>;
  /** Whether `keyboardHeight` exceeds `keyboardThreshold`. */
  readonly keyboardOpen: ComputedRef<boolean>;
  /** Whether a VisualViewport (or VirtualKeyboard) capability is attached. */
  readonly isSupported: Readonly<Ref<boolean>>;
}

/**
 * Track the visual viewport and on-screen keyboard.
 *
 * On mobile browsers the visual viewport shrinks (or scrolls) when a virtual
 * keyboard opens while the layout viewport stays put; `keyboardHeight` is the
 * occluded bottom area (`innerHeight - height - offsetTop`), or the
 * VirtualKeyboard API rectangle when `overlaysContent` is enabled. Pin
 * toolbars above the keyboard with `bottom: calc(var(--kb, 0px))` bound to it.
 *
 * Server rendering reports zeros (scale 1) and attaches nothing; listeners
 * (`resize`, `scroll`, `geometrychange`) are passive and removed with the
 * owning reactive scope.
 *
 * @example
 * ```ts
 * const { keyboardHeight, keyboardOpen } = useVisualViewport();
 * const toolbarStyle = computed(() => ({ bottom: `${keyboardHeight.value}px` }));
 * ```
 *
 * @param options Keyboard threshold, VirtualKeyboard opt-in, and capability.
 * @default options {}
 * @returns Reactive viewport geometry and keyboard state.
 */
export function useVisualViewport(options: UseVisualViewportOptions = {}): VisualViewportControls {
  const threshold =
    typeof options.keyboardThreshold === "number" && Number.isFinite(options.keyboardThreshold)
      ? Math.max(options.keyboardThreshold, 0)
      : 120;
  const width = ref(0);
  const height = ref(0);
  const offsetTop = ref(0);
  const offsetLeft = ref(0);
  const scale = ref(1);
  const keyboardHeight = ref(0);
  const isSupported = ref(false);

  const stopWatch = watch(
    () => (options.host === undefined ? browserViewportHost() : toValue(options.host)),
    (host, _previous, onCleanup) => {
      const viewport = host?.visualViewport ?? undefined;
      const keyboard =
        options.overlaysContent === true ? virtualKeyboardOf(host?.navigator) : undefined;
      isSupported.value = viewport !== undefined || keyboard !== undefined;
      if (host === null || host === undefined || !isSupported.value) {
        width.value = 0;
        height.value = 0;
        offsetTop.value = 0;
        offsetLeft.value = 0;
        scale.value = 1;
        keyboardHeight.value = 0;
        return;
      }
      const previousOverlay = keyboard?.overlaysContent;
      if (keyboard !== undefined) keyboard.overlaysContent = true;
      const update = (): void => {
        if (viewport !== undefined) {
          width.value = viewport.width;
          height.value = viewport.height;
          offsetTop.value = viewport.offsetTop;
          offsetLeft.value = viewport.offsetLeft;
          scale.value = viewport.scale;
        }
        const occluded =
          keyboard !== undefined
            ? keyboard.boundingRect.height
            : viewport === undefined
              ? 0
              : host.innerHeight - viewport.height - viewport.offsetTop;
        keyboardHeight.value = Math.max(Math.round(occluded), 0);
      };
      update();
      const passive: AddEventListenerOptions = { passive: true };
      viewport?.addEventListener("resize", update, passive);
      viewport?.addEventListener("scroll", update, passive);
      keyboard?.addEventListener("geometrychange", update, passive);
      onCleanup(() => {
        viewport?.removeEventListener("resize", update);
        viewport?.removeEventListener("scroll", update);
        keyboard?.removeEventListener("geometrychange", update);
        if (keyboard !== undefined && previousOverlay !== undefined) {
          keyboard.overlaysContent = previousOverlay;
        }
      });
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stopWatch.stop());

  return {
    width: readonly(width),
    height: readonly(height),
    offsetTop: readonly(offsetTop),
    offsetLeft: readonly(offsetLeft),
    scale: readonly(scale),
    keyboardHeight: readonly(keyboardHeight),
    keyboardOpen: computed(() => keyboardHeight.value > threshold),
    isSupported: readonly(isSupported),
  };
}

function browserViewportHost(): VisualViewportWindowHost | undefined {
  return typeof window !== "undefined" ? window : undefined;
}

function isVirtualKeyboard(value: unknown): value is VirtualKeyboardHost {
  return (
    typeof value === "object" &&
    value !== null &&
    "overlaysContent" in value &&
    "boundingRect" in value &&
    typeof Reflect.get(value, "addEventListener") === "function"
  );
}

function virtualKeyboardOf(navigator: object | undefined): VirtualKeyboardHost | undefined {
  if (navigator === undefined || !("virtualKeyboard" in navigator)) return undefined;
  const keyboard: unknown = navigator.virtualKeyboard;
  return isVirtualKeyboard(keyboard) ? keyboard : undefined;
}
