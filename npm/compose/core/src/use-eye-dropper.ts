import { computed, readonly, ref, unref } from "vue";
import type { ComputedRef, MaybeRef, Ref } from "vue";

/** Color picked by the EyeDropper API. */
export interface EyeDropperColor {
  /** Picked color as a `#rrggbb` string. */
  readonly sRGBHex: string;
}

/** Minimal `EyeDropper` instance. */
export interface EyeDropperLike {
  /** Open the picker. Rejects with AbortError when dismissed or aborted. */
  open(options?: { signal?: AbortSignal }): Promise<EyeDropperColor>;
}

/** Minimal `EyeDropper` constructor. */
export type EyeDropperHost = new () => EyeDropperLike;

/** Options for {@link useEyeDropper}. */
export interface UseEyeDropperOptions {
  /**
   * EyeDropper constructor for alternate runtimes and tests. A ref (not a
   * getter) because the host itself is a constructor function.
   *
   * @default window.EyeDropper when it exists
   */
  readonly host?: MaybeRef<EyeDropperHost | null | undefined>;

  /**
   * Initial `sRGBHex` value.
   *
   * @default ""
   */
  readonly initialValue?: string;
}

/** Discriminated outcome of {@link EyeDropperControls.open}. */
export type EyeDropperResult =
  | {
      /** A color was picked. */
      readonly status: "picked";
      /** Picked color. */
      readonly sRGBHex: string;
    }
  | {
      /** Dismissed/aborted, missing API, or another failure. */
      readonly status: "cancelled" | "unsupported" | "failed";
      /** Exact error thrown by the host, when one was thrown. */
      readonly error: unknown;
    };

/** Reactive state and actions returned by {@link useEyeDropper}. */
export interface EyeDropperControls {
  /** Whether the EyeDropper API is available. */
  readonly supported: ComputedRef<boolean>;

  /** Most recently picked color. */
  readonly sRGBHex: Readonly<Ref<string>>;

  /** Whether the picker is open. */
  readonly picking: Readonly<Ref<boolean>>;

  /**
   * Open the picker. Browsers require a user gesture.
   *
   * @param signal Aborts the open picker.
   * @returns The discriminated outcome; never rejects.
   */
  readonly open: (signal?: AbortSignal) => Promise<EyeDropperResult>;
}

function isEyeDropperHost(candidate: unknown): candidate is EyeDropperHost {
  return typeof candidate === "function";
}

function browserEyeDropper(): EyeDropperHost | undefined {
  if (typeof window === "undefined") return undefined;
  const candidate: unknown = Reflect.get(window, "EyeDropper");
  return isEyeDropperHost(candidate) ? candidate : undefined;
}

function isAbort(error: unknown): boolean {
  return (
    typeof error === "object" && error !== null && "name" in error && error.name === "AbortError"
  );
}

/**
 * Pick a color from anywhere on screen with the EyeDropper API.
 *
 * `open` resolves to a discriminated {@link EyeDropperResult}; dismissing
 * the picker (Escape) or aborting via `signal` yields `"cancelled"`.
 * The composable holds no resources beyond an open picker, which the
 * caller controls through `signal`.
 *
 * Server rendering: `supported` is false and `sRGBHex` is `initialValue`.
 *
 * @example
 * ```ts
 * const { open, sRGBHex, supported } = useEyeDropper();
 * ```
 *
 * @param options Capability and initial color.
 * @default options {}
 * @returns Picker state and the open action.
 */
export function useEyeDropper(options: UseEyeDropperOptions = {}): EyeDropperControls {
  const sRGBHex = ref(options.initialValue ?? "");
  const picking = ref(false);
  const resolveHost = (): EyeDropperHost | undefined =>
    options.host === undefined ? browserEyeDropper() : (unref(options.host) ?? undefined);

  const open = async (signal?: AbortSignal): Promise<EyeDropperResult> => {
    const Host = resolveHost();
    if (!Host) return { status: "unsupported", error: undefined };
    picking.value = true;
    try {
      const color = await new Host().open(signal === undefined ? {} : { signal });
      sRGBHex.value = color.sRGBHex;
      return { status: "picked", sRGBHex: color.sRGBHex };
    } catch (error) {
      return { status: isAbort(error) ? "cancelled" : "failed", error };
    } finally {
      picking.value = false;
    }
  };

  return {
    supported: computed(() => resolveHost() !== undefined),
    sRGBHex: readonly(sRGBHex),
    picking: readonly(picking),
    open,
  };
}
