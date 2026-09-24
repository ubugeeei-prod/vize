import { computed, isRef, ref, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Document subset used by {@link useDocumentTitle}. */
export interface DocumentTitleHost {
  /** Current document title (readable and writable). */
  title: string;

  /**
   * Observe external title changes (for example by other libraries).
   * Returns a function that stops observing.
   */
  readonly observeTitle?: (callback: () => void) => () => void;
}

/** Title template: a string containing `%s`, or a formatting function. */
export type DocumentTitleTemplate = string | ((title: string) => string);

/** Title applied when the owning scope stops. */
export type DocumentTitleRestore =
  | false
  | "previous"
  | ((originalTitle: string, currentTitle: string) => string);

/** Options for {@link useDocumentTitle}. */
export interface UseDocumentTitleOptions {
  /**
   * Template applied to every title written to the document.
   *
   * @default "%s"
   */
  readonly template?: DocumentTitleTemplate;

  /**
   * Title restored when the owning reactive scope stops: `false` leaves the
   * current title, `"previous"` restores the title seen at creation, and a
   * function computes it.
   *
   * @default false
   */
  readonly restoreOnDispose?: DocumentTitleRestore;

  /**
   * Follow title changes made outside this composable.
   *
   * @default false
   */
  readonly observe?: boolean;

  /**
   * Document capability for alternate runtimes and tests.
   *
   * @default window.document (with a MutationObserver on `<title>`)
   */
  readonly host?: MaybeRefOrGetter<DocumentTitleHost | null | undefined>;
}

/** Reactive state returned by {@link useDocumentTitle}. */
export interface DocumentTitleControls {
  /**
   * Title before templating. Writable; `null`/`undefined` leaves the
   * document title untouched. When a getter was passed, the next getter
   * change overwrites manual assignments.
   */
  readonly title: Ref<string | null | undefined>;

  /** Whether a document capability is attached. */
  readonly supported: ComputedRef<boolean>;
}

function browserTitleHost(): DocumentTitleHost | undefined {
  if (typeof window === "undefined") return undefined;
  const { document, MutationObserver } = window;
  return {
    get title() {
      return document.title;
    },
    set title(value: string) {
      document.title = value;
    },
    observeTitle: (callback) => {
      const observer = new MutationObserver(callback);
      const element = document.head.querySelector("title");
      if (element) {
        observer.observe(element, { childList: true, characterData: true, subtree: true });
      }
      return () => observer.disconnect();
    },
  };
}

function formatTitle(template: DocumentTitleTemplate, title: string): string {
  return typeof template === "function" ? template(title) : template.replaceAll("%s", title);
}

/**
 * Bind the document title to reactive state.
 *
 * Accepts a writable ref (used as-is), a getter (followed), a plain
 * string, or nothing (an owned ref seeded from the current title). Every
 * non-nullish title is passed through `template` before it is written.
 * With `observe`, external title changes flow back into `title`.
 *
 * Server rendering: the document is never touched and `title` keeps the
 * value it was given (or `undefined`). Head management for server-rendered
 * markup (`<title>` in the HTML) is the job of the application's head
 * manager; this composable only updates the live document on the client.
 * Watchers and the observer are released with the owning reactive scope,
 * which also applies `restoreOnDispose`.
 *
 * @example
 * ```ts
 * const { title } = useDocumentTitle(() => route.meta.title, { template: "%s | Vize" });
 * ```
 *
 * @param title Reactive title source.
 * @param options Template, restore policy, observation, and capability.
 * @default options {}
 * @returns The title ref and support flag.
 */
export function useDocumentTitle(
  title?: MaybeRefOrGetter<string | null | undefined>,
  options: UseDocumentTitleOptions = {},
): DocumentTitleControls {
  const template = options.template ?? "%s";
  const resolveHost = (): DocumentTitleHost | undefined =>
    options.host === undefined ? browserTitleHost() : (toValue(options.host) ?? undefined);
  const initialHost = resolveHost();
  const originalTitle = initialHost?.title ?? "";
  const state: Ref<string | null | undefined> = isRef(title)
    ? title
    : ref(title === undefined ? initialHost?.title : toValue(title));
  let lastWritten: string | undefined;

  if (typeof title === "function") {
    watch(title, (next) => {
      state.value = next;
    });
  }

  watch(
    [state, resolveHost],
    ([next, host]) => {
      if (!host || next === null || next === undefined) return;
      const formatted = formatTitle(template, next);
      lastWritten = formatted;
      if (host.title !== formatted) host.title = formatted;
    },
    { immediate: true, flush: "sync" },
  );

  if (options.observe ?? false) {
    watch(
      resolveHost,
      (host, _previous, onCleanup) => {
        if (!host?.observeTitle) return;
        onCleanup(
          host.observeTitle(() => {
            if (host.title !== lastWritten) state.value = host.title;
          }),
        );
      },
      { immediate: true, flush: "sync" },
    );
  }

  const restore = options.restoreOnDispose ?? false;
  if (restore !== false) {
    tryOnScopeDispose(() => {
      const host = resolveHost();
      if (!host) return;
      host.title = restore === "previous" ? originalTitle : restore(originalTitle, host.title);
    });
  }

  return { title: state, supported: computed(() => resolveHost() !== undefined) };
}
