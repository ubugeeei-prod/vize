import { readonly, shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Window-like capability observed by {@link usePreferredLanguages}. */
export interface PreferredLanguagesHost extends EventTarget {
  /** Navigator exposing the user's languages. */
  readonly navigator: { readonly languages: readonly string[]; readonly language?: string };
}

/** Options for {@link usePreferredLanguages}. */
export interface UsePreferredLanguagesOptions {
  /**
   * Languages exposed during server rendering. Pass the request's
   * `Accept-Language` list to keep hydration stable.
   *
   * @default ["en"]
   */
  readonly ssrLanguages?: readonly string[];

  /**
   * Reactive window capability for alternate runtimes and tests.
   *
   * @default globalThis.window when available
   */
  readonly host?: MaybeRefOrGetter<PreferredLanguagesHost | null | undefined>;
}

/**
 * Track the user's preferred languages (`navigator.languages`).
 *
 * Updates on `languagechange`. The default host is gated on `window`, so
 * Node's own `navigator` never leaks the server locale into the render;
 * server renders expose `ssrLanguages`. The listener is removed with the
 * owning reactive scope.
 *
 * @param options Server fallback and window capability.
 * @default options {}
 * @returns Readonly ref of BCP 47 language tags, most preferred first.
 */
export function usePreferredLanguages(
  options: UsePreferredLanguagesOptions = {},
): Readonly<Ref<readonly string[]>> {
  const fallback = options.ssrLanguages ?? ["en"];
  const languages = shallowRef<readonly string[]>(fallback);

  const stop = watch(
    () => (options.host === undefined ? browserLanguagesHost() : toValue(options.host)),
    (host, _previous, onCleanup) => {
      if (!host) {
        languages.value = fallback;
        return;
      }
      const update = (): void => {
        const { languages: list, language } = host.navigator;
        languages.value =
          list.length > 0 ? [...list] : language !== undefined ? [language] : fallback;
      };
      update();
      host.addEventListener("languagechange", update, { passive: true });
      onCleanup(() => host.removeEventListener("languagechange", update));
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stop.stop());

  return readonly(languages);
}

function browserLanguagesHost(): PreferredLanguagesHost | undefined {
  return typeof window !== "undefined" ? window : undefined;
}
