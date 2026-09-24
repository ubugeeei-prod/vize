import { watch } from "vue";
import type { Ref } from "vue";

/** Which way {@link syncRef} copies values. */
export type SyncRefDirection = "ltr" | "rtl" | "both";

/** Converters required by {@link syncRef} for the chosen direction. */
export type SyncRefTransform<
  Left,
  Right,
  Direction extends SyncRefDirection,
> = Direction extends "ltr"
  ? { readonly ltr: (left: Left) => Right }
  : Direction extends "rtl"
    ? { readonly rtl: (right: Right) => Left }
    : { readonly ltr: (left: Left) => Right; readonly rtl: (right: Right) => Left };

/** Whether two types are mutually assignable (identical for syncing purposes). */
type SameType<Left, Right> = [Left] extends [Right]
  ? [Right] extends [Left]
    ? true
    : false
  : false;

/** Options for {@link syncRef}; `transform` is required when the ref types differ. */
export type SyncRefOptions<
  Left,
  Right,
  Direction extends SyncRefDirection = "both",
> = SyncRefBaseOptions<Direction> &
  (SameType<Left, Right> extends true
    ? {
        /**
         * Converters between the two refs.
         *
         * @default identity
         */
        readonly transform?: Partial<SyncRefTransform<Left, Right, Direction>>;
      }
    : {
        /** Converters between the two refs (required: the types differ). */
        readonly transform: SyncRefTransform<Left, Right, Direction>;
      });

/** Direction and timing options shared by every {@link syncRef} call. */
export interface SyncRefBaseOptions<Direction extends SyncRefDirection = "both"> {
  /**
   * `"ltr"` copies left to right, `"rtl"` right to left, `"both"` both ways.
   *
   * @default "both"
   */
  readonly direction?: Direction;

  /**
   * Watch flush timing.
   *
   * @default "sync"
   */
  readonly flush?: "pre" | "post" | "sync";

  /**
   * Watch nested changes.
   *
   * @default false
   */
  readonly deep?: boolean;

  /**
   * Copy once right away (left wins for `"both"`/`"ltr"`, right for `"rtl"`).
   *
   * @default true
   */
  readonly immediate?: boolean;
}

/**
 * Keep two refs in sync, optionally converting between their types.
 *
 * The option type is computed from both ref types: converters may be
 * omitted only when the types are identical, and exactly the converters the
 * direction needs are required otherwise. A re-entrancy guard prevents
 * ping-pong when converters are not exact inverses. Watchers follow the
 * owning reactive scope; SSR-safe.
 *
 * @example
 * ```ts
 * const celsius = ref(20);
 * const label = ref("");
 * syncRef(celsius, label, {
 *   transform: { ltr: (c) => `${c}°C`, rtl: (text) => Number.parseFloat(text) },
 * });
 * ```
 *
 * @param left First ref.
 * @param right Second ref.
 * @param options Direction, timing, and converters.
 * @returns Stops every sync watcher.
 */
export function syncRef<Left, Right = Left, const Direction extends SyncRefDirection = "both">(
  left: Ref<Left>,
  right: Ref<Right>,
  ...options: SameType<Left, Right> extends true
    ? [options?: SyncRefOptions<Left, Right, Direction>]
    : [options: SyncRefOptions<Left, Right, Direction>]
): () => void;
export function syncRef(
  left: Ref<unknown>,
  right: Ref<unknown>,
  options: SyncRefBaseOptions<SyncRefDirection> & {
    readonly transform?: {
      readonly ltr?: (left: unknown) => unknown;
      readonly rtl?: (right: unknown) => unknown;
    };
  } = {},
): () => void {
  const direction = options.direction ?? "both";
  const toRight = options.transform?.ltr ?? ((value: unknown) => value);
  const toLeft = options.transform?.rtl ?? ((value: unknown) => value);
  const watchOptions = {
    flush: options.flush ?? "sync",
    deep: options.deep ?? false,
  } as const;
  const stops: (() => void)[] = [];
  let syncing = false;

  const guarded = (copy: () => void): void => {
    if (syncing) return;
    syncing = true;
    try {
      copy();
    } finally {
      syncing = false;
    }
  };

  if (direction !== "rtl") {
    stops.push(
      watch(
        left,
        (value) => {
          guarded(() => {
            right.value = toRight(value);
          });
        },
        { ...watchOptions, immediate: options.immediate ?? true },
      ),
    );
  }
  if (direction !== "ltr") {
    stops.push(
      watch(
        right,
        (value) => {
          guarded(() => {
            left.value = toLeft(value);
          });
        },
        { ...watchOptions, immediate: direction === "rtl" && (options.immediate ?? true) },
      ),
    );
  }

  return () => {
    for (const stop of stops) stop();
  };
}
