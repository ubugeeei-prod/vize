import { readonly, ref, toValue, unref, watch } from "vue";
import type { MaybeRef, MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Objects accepted by `URL.createObjectURL`. */
export type ObjectUrlSource = Blob | MediaSource;

/** Minimal object-URL capability (the `URL` constructor's static methods). */
export interface ObjectUrlHost {
  /** Create a URL referencing `object`. */
  createObjectURL(object: ObjectUrlSource): string;

  /** Release a URL previously returned by `createObjectURL`. */
  revokeObjectURL(url: string): void;
}

/** Options for {@link useObjectUrl}. */
export interface UseObjectUrlOptions {
  /**
   * Object-URL capability for alternate runtimes and tests. A ref (not a
   * getter) because the default host, `URL`, is itself a constructor.
   *
   * @default window.URL when a browser window exists
   */
  readonly host?: MaybeRef<ObjectUrlHost | null | undefined>;
}

function browserObjectUrlHost(): ObjectUrlHost | undefined {
  if (typeof window === "undefined") return undefined;
  const { URL } = window;
  return typeof URL.createObjectURL === "function" ? URL : undefined;
}

/**
 * Create a reactive object URL for a `Blob`, `File`, or `MediaSource`.
 *
 * A new URL is created whenever the reactive object changes and the
 * previous one is revoked immediately, so memory held by the old object is
 * released. The last URL is revoked when the owning reactive scope stops.
 * Outside a scope, the caller owns the last URL (it lives until unload).
 *
 * Server rendering: no URL is created and the ref stays `undefined`.
 *
 * @example
 * ```ts
 * const file = shallowRef<File>();
 * const preview = useObjectUrl(file);
 * // <img :src="preview">
 * ```
 *
 * @param object Reactive object to reference; `null`/`undefined` clears the URL.
 * @param options Object-URL capability.
 * @default options {}
 * @returns Readonly ref holding the current object URL.
 */
export function useObjectUrl(
  object: MaybeRefOrGetter<ObjectUrlSource | null | undefined>,
  options: UseObjectUrlOptions = {},
): Readonly<Ref<string | undefined>> {
  const url = ref<string | undefined>(undefined);
  const resolveHost = (): ObjectUrlHost | undefined =>
    options.host === undefined ? browserObjectUrlHost() : (unref(options.host) ?? undefined);

  let revokeCurrent: (() => void) | undefined;
  const release = (): void => {
    revokeCurrent?.();
    revokeCurrent = undefined;
    url.value = undefined;
  };

  watch(
    [() => toValue(object), resolveHost],
    ([next, host]) => {
      release();
      if (!next || !host) return;
      const created = host.createObjectURL(next);
      url.value = created;
      revokeCurrent = () => host.revokeObjectURL(created);
    },
    { immediate: true, flush: "sync" },
  );

  tryOnScopeDispose(release);

  return readonly(url);
}
