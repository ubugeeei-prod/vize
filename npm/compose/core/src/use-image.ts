import { computed, readonly, ref, shallowRef, toValue, unref, watch } from "vue";
import type { ComputedRef, MaybeRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Image attributes accepted by {@link useImage}. */
export interface ImageSource {
  /** Image URL. */
  readonly src: string;

  /** Responsive candidates. */
  readonly srcset?: string;

  /** Responsive layout sizes. */
  readonly sizes?: string;

  /** Alternative text. */
  readonly alt?: string;

  /** CORS mode. */
  readonly crossOrigin?: "anonymous" | "use-credentials";

  /** Referrer policy. */
  readonly referrerPolicy?: ReferrerPolicy;

  /** Loading strategy. */
  readonly loading?: "eager" | "lazy";

  /** Decoding hint. */
  readonly decoding?: "sync" | "async" | "auto";

  /** Fetch priority hint. */
  readonly fetchPriority?: "high" | "low" | "auto";

  /** Intrinsic width. */
  readonly width?: number;

  /** Intrinsic height. */
  readonly height?: number;
}

/** `HTMLImageElement` subset used by {@link useImage}. */
export interface ImageLike extends EventTarget {
  /** Image URL. */
  src: string;

  /** Responsive candidates. */
  srcset: string;

  /** Responsive layout sizes. */
  sizes: string;

  /** Alternative text. */
  alt: string;

  /** CORS mode. */
  crossOrigin: string | null;

  /** Referrer policy. */
  referrerPolicy: string;

  /** Loading strategy. */
  loading: string;

  /** Decoding hint. */
  decoding: string;

  /** Fetch priority hint; absent in engines without priority hints. */
  fetchPriority?: string;

  /** Intrinsic width. */
  width: number;

  /** Intrinsic height. */
  height: number;
}

/** Image constructor used by {@link useImage}. */
export type ImageHost = new () => ImageLike;

/** Loading state of the image. */
export type ImageStatus = "idle" | "loading" | "loaded" | "error";

/** Discriminated outcome of {@link ImageControls.load}. */
export type ImageLoadResult =
  | {
      /** The image loaded. */
      readonly status: "loaded";
      /** Loaded image. */
      readonly image: ImageLike;
    }
  | {
      /** The image failed to load. */
      readonly status: "error";
      /** Native error event. */
      readonly error: Event;
    }
  | {
      /** A newer load started, the scope stopped, or no capability exists. */
      readonly status: "cancelled" | "unsupported";
    };

/** Options for {@link useImage}. */
export interface UseImageOptions {
  /**
   * Load when created and whenever the source changes.
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * Image constructor for alternate runtimes and tests. A ref (not a
   * getter) because the host is a constructor function.
   *
   * @default window.Image when a browser window exists
   */
  readonly host?: MaybeRef<ImageHost | null | undefined>;
}

/** Reactive state and actions returned by {@link useImage}. */
export interface ImageControls {
  /** Loading state of the newest load. */
  readonly status: Readonly<Ref<ImageStatus>>;

  /** Most recently loaded image. */
  readonly image: Readonly<ShallowRef<ImageLike | undefined>>;

  /** Error event of the newest load, cleared when a load starts. */
  readonly error: Readonly<ShallowRef<Event | undefined>>;

  /** Whether the newest load succeeded. */
  readonly ready: ComputedRef<boolean>;

  /**
   * Load the current source, superseding any pending load.
   *
   * @returns The discriminated outcome; never rejects.
   */
  readonly load: () => Promise<ImageLoadResult>;
}

function browserImageHost(): ImageHost | undefined {
  return typeof window === "undefined" ? undefined : window.Image;
}

function applySource(image: ImageLike, source: ImageSource): void {
  if (source.crossOrigin !== undefined) image.crossOrigin = source.crossOrigin;
  if (source.referrerPolicy !== undefined) image.referrerPolicy = source.referrerPolicy;
  if (source.loading !== undefined) image.loading = source.loading;
  if (source.decoding !== undefined) image.decoding = source.decoding;
  if (source.fetchPriority !== undefined) image.fetchPriority = source.fetchPriority;
  if (source.alt !== undefined) image.alt = source.alt;
  if (source.width !== undefined) image.width = source.width;
  if (source.height !== undefined) image.height = source.height;
  if (source.sizes !== undefined) image.sizes = source.sizes;
  if (source.srcset !== undefined) image.srcset = source.srcset;
  image.src = source.src;
}

/**
 * Preload an image and track its loading state.
 *
 * A detached image is created for every source; the newest load wins, and
 * superseded images have their listeners removed and their `src` cleared so
 * the browser can abort the download. Use it to show a placeholder until
 * the real image is decoded, or to validate URLs before rendering them.
 *
 * Server rendering: no image is created and `status` stays `"idle"`. The
 * pending load is cancelled when the owning reactive scope stops.
 *
 * @example
 * ```ts
 * const { ready } = useImage(() => ({ src: avatarUrl.value }));
 * // <img v-if="ready" :src="avatarUrl"> <Skeleton v-else />
 * ```
 *
 * @param source Reactive image attributes.
 * @param options Timing and capability.
 * @default options {}
 * @returns Image state and actions.
 */
export function useImage(
  source: MaybeRefOrGetter<ImageSource>,
  options: UseImageOptions = {},
): ImageControls {
  const status = ref<ImageStatus>("idle");
  const image = shallowRef<ImageLike | undefined>(undefined);
  const error = shallowRef<Event | undefined>(undefined);
  let cancelActive: (() => void) | undefined;

  const resolveHost = (): ImageHost | undefined =>
    options.host === undefined ? browserImageHost() : (unref(options.host) ?? undefined);

  const load = (): Promise<ImageLoadResult> => {
    cancelActive?.();
    cancelActive = undefined;
    const Host = resolveHost();
    if (!Host) return Promise.resolve({ status: "unsupported" });
    const candidate = new Host();
    status.value = "loading";
    error.value = undefined;
    return new Promise((resolve) => {
      const detach = (): void => {
        candidate.removeEventListener("load", onLoad);
        candidate.removeEventListener("error", onError);
        cancelActive = undefined;
      };
      const onLoad = (): void => {
        detach();
        image.value = candidate;
        status.value = "loaded";
        resolve({ status: "loaded", image: candidate });
      };
      const onError = (event: Event): void => {
        detach();
        error.value = event;
        status.value = "error";
        resolve({ status: "error", error: event });
      };
      candidate.addEventListener("load", onLoad);
      candidate.addEventListener("error", onError);
      cancelActive = () => {
        detach();
        candidate.src = "";
        resolve({ status: "cancelled" });
      };
      applySource(candidate, toValue(source));
    });
  };

  if (options.immediate ?? true) {
    watch(
      () => toValue(source),
      () => {
        void load();
      },
      { immediate: true, deep: true, flush: "sync" },
    );
  }

  tryOnScopeDispose(() => {
    cancelActive?.();
  });

  return {
    status: readonly(status),
    image,
    error,
    ready: computed(() => status.value === "loaded"),
    load,
  };
}
