import { computed, readonly, ref, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref } from "vue";

/** Data shared through the Web Share API. */
export interface ShareDataLike {
  /** Title of the shared content. */
  readonly title?: string;

  /** Body text of the shared content. */
  readonly text?: string;

  /** URL of the shared content. */
  readonly url?: string;

  /** Files to share (Level 2 of the Web Share API). */
  readonly files?: File[];
}

/** Minimal Web Share capability (`navigator`). */
export interface ShareHost {
  /** Open the platform share sheet. */
  share(data: ShareDataLike): Promise<void>;

  /** Whether the platform can share `data`. Absent in older engines. */
  canShare?(data?: ShareDataLike): boolean;
}

/** Discriminated outcome of {@link ShareControls.share}. */
export type ShareResult =
  | {
      /** The share sheet completed. */
      readonly status: "shared";
      /** Data that was shared. */
      readonly data: ShareDataLike;
    }
  | {
      /**
       * `"cancelled"`: the user dismissed the sheet (AbortError).
       * `"unsupported"`: no Web Share capability.
       * `"invalid"`: the platform cannot share this data (canShare / TypeError).
       * `"not-allowed"`: no transient user activation or blocked by policy.
       * `"failed"`: any other failure.
       */
      readonly status: "cancelled" | "unsupported" | "invalid" | "not-allowed" | "failed";
      /** Exact error thrown by the host, when one was thrown. */
      readonly error: unknown;
    };

/** Options for {@link useShare}. */
export interface UseShareOptions {
  /**
   * Web Share capability for alternate runtimes and tests.
   *
   * @default window.navigator when it implements `share`
   */
  readonly host?: MaybeRefOrGetter<ShareHost | null | undefined>;
}

/** Reactive state and actions returned by {@link useShare}. */
export interface ShareControls {
  /** Whether the Web Share API is available. */
  readonly supported: ComputedRef<boolean>;

  /** Whether a share sheet opened by this composable is currently open. */
  readonly sharing: Readonly<Ref<boolean>>;

  /**
   * Whether the platform can share the given (or default) data.
   *
   * @param data Data merged over the defaults.
   * @returns `false` when unsupported.
   */
  readonly canShare: (data?: ShareDataLike) => boolean;

  /**
   * Open the share sheet. Must run inside a user gesture in browsers.
   *
   * @param overrides Data merged over the defaults.
   * @returns The discriminated outcome; never rejects.
   */
  readonly share: (overrides?: ShareDataLike) => Promise<ShareResult>;
}

function browserShareHost(): ShareHost | undefined {
  if (typeof window === "undefined") return undefined;
  const { navigator } = window;
  return typeof navigator.share === "function" ? navigator : undefined;
}

function errorName(error: unknown): string | undefined {
  return typeof error === "object" && error !== null && "name" in error
    ? String(error.name)
    : undefined;
}

/**
 * Share text, links, and files through the platform share sheet.
 *
 * Reactive defaults are merged with per-call overrides. Outcomes are
 * reported as a discriminated {@link ShareResult}, so a user dismissing the
 * sheet (`"cancelled"`) is distinguishable from missing support or invalid
 * data without try/catch.
 *
 * Server rendering: no capability is resolved, `supported` is false, and
 * nothing is shared. No resources are retained, so nothing needs cleanup.
 *
 * @example
 * ```ts
 * const { share, supported } = useShare(() => ({ title: document.title, url: location.href }));
 * const onClick = () => share();
 * ```
 *
 * @param defaults Reactive default share data.
 * @param options Web Share capability.
 * @default defaults {}
 * @default options {}
 * @returns Share state and actions.
 */
export function useShare(
  defaults: MaybeRefOrGetter<ShareDataLike> = {},
  options: UseShareOptions = {},
): ShareControls {
  const sharing = ref(false);
  const resolveHost = (): ShareHost | undefined =>
    options.host === undefined ? browserShareHost() : (toValue(options.host) ?? undefined);
  const merge = (data: ShareDataLike | undefined): ShareDataLike => ({
    ...toValue(defaults),
    ...data,
  });

  const canShare = (data?: ShareDataLike): boolean => {
    const host = resolveHost();
    if (!host) return false;
    if (!host.canShare) return true;
    try {
      return host.canShare(merge(data));
    } catch {
      return false;
    }
  };

  const share = async (overrides?: ShareDataLike): Promise<ShareResult> => {
    const host = resolveHost();
    if (!host) return { status: "unsupported", error: undefined };
    const data = merge(overrides);
    if (host.canShare && !canShare(overrides)) return { status: "invalid", error: undefined };
    sharing.value = true;
    try {
      await host.share(data);
      return { status: "shared", data };
    } catch (error) {
      const name = errorName(error);
      if (name === "AbortError") return { status: "cancelled", error };
      if (name === "NotAllowedError") return { status: "not-allowed", error };
      if (name === "TypeError" || name === "DataError") return { status: "invalid", error };
      return { status: "failed", error };
    } finally {
      sharing.value = false;
    }
  };

  return {
    supported: computed(() => resolveHost() !== undefined),
    sharing: readonly(sharing),
    canShare,
    share,
  };
}
