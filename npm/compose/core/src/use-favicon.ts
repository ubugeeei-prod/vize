import { computed, isRef, ref, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref } from "vue";

/** `<link>` subset updated by {@link useFavicon}. */
export interface FaviconLink {
  /** Link relation. */
  rel: string;

  /** Icon URL. */
  href: string;
}

/** Document capability used by {@link useFavicon}. */
export interface FaviconHost {
  /** Find every `<link>` whose `rel` contains `rel`. */
  findIcons(rel: string): Iterable<FaviconLink>;

  /** Create (and attach to `<head>`) a new `<link rel={rel}>`. */
  createIcon(rel: string): FaviconLink;
}

/** Options for {@link useFavicon}. */
export interface UseFaviconOptions {
  /**
   * Prefix prepended to every icon value.
   *
   * @default ""
   */
  readonly baseUrl?: string;

  /**
   * Link relation matched (substring) and used for created links.
   *
   * @default "icon"
   */
  readonly rel?: string;

  /**
   * Document capability for alternate runtimes and tests.
   *
   * @default window.document's `<head>`
   */
  readonly host?: MaybeRefOrGetter<FaviconHost | null | undefined>;
}

/** Reactive state returned by {@link useFavicon}. */
export interface FaviconControls {
  /**
   * Icon URL (before `baseUrl`). Writable; `null`/`undefined` leaves the
   * current icons untouched. When a getter was passed, the next getter
   * change overwrites manual assignments.
   */
  readonly icon: Ref<string | null | undefined>;

  /** Whether a document capability is attached. */
  readonly supported: ComputedRef<boolean>;
}

function browserFaviconHost(): FaviconHost | undefined {
  if (typeof window === "undefined") return undefined;
  const { document, HTMLLinkElement, CSS } = window;
  return {
    findIcons: (rel) =>
      [...document.head.querySelectorAll(`link[rel*="${CSS.escape(rel)}"]`)].filter(
        (element) => element instanceof HTMLLinkElement,
      ),
    createIcon: (rel) => {
      const link = document.createElement("link");
      link.rel = rel;
      document.head.append(link);
      return link;
    },
  };
}

/**
 * Bind the page favicon to reactive state.
 *
 * Every `<link>` whose `rel` contains `rel` (`icon`, `shortcut icon`,
 * `apple-touch-icon`, …) gets `baseUrl + icon` as its `href`; when none
 * exists, one `<link rel="icon">` is created. Accepts a writable ref (used
 * as-is), a getter (followed), a plain string, or nothing (an owned ref).
 *
 * Server rendering: this is a client-only side effect. The document is
 * never touched on the server, and `icon` keeps its given value; emit the
 * initial `<link rel="icon">` through the application's head manager.
 * Watchers are released with the owning reactive scope; the last icon stays.
 *
 * @example
 * ```ts
 * const { icon } = useFavicon(() => (unread.value ? "/unread.svg" : "/icon.svg"));
 * ```
 *
 * @param icon Reactive icon source.
 * @param options Base URL, relation, and capability.
 * @default options {}
 * @returns The icon ref and support flag.
 */
export function useFavicon(
  icon?: MaybeRefOrGetter<string | null | undefined>,
  options: UseFaviconOptions = {},
): FaviconControls {
  const baseUrl = options.baseUrl ?? "";
  const rel = options.rel ?? "icon";
  const resolveHost = (): FaviconHost | undefined =>
    options.host === undefined ? browserFaviconHost() : (toValue(options.host) ?? undefined);
  const state: Ref<string | null | undefined> = isRef(icon) ? icon : ref(toValue(icon));

  if (typeof icon === "function") {
    watch(icon, (next) => {
      state.value = next;
    });
  }

  watch(
    [state, resolveHost],
    ([next, host]) => {
      if (!host || next === null || next === undefined) return;
      const href = `${baseUrl}${next}`;
      let found = false;
      for (const link of host.findIcons(rel)) {
        found = true;
        if (link.href !== href) link.href = href;
      }
      if (!found) host.createIcon(rel).href = href;
    },
    { immediate: true, flush: "sync" },
  );

  return { icon: state, supported: computed(() => resolveHost() !== undefined) };
}
