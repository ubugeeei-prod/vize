import { isRef, readonly, ref, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** `<style>` subset used by {@link useStyleTag}. */
export interface StyleTagElement {
  /** Style sheet text. */
  textContent: string | null;

  /** Write an attribute. */
  setAttribute(name: string, value: string): void;
}

/** Document capability used by {@link useStyleTag}. */
export interface StyleTagHost {
  /** Find an existing `<style>` with the given `id`. */
  findStyle(id: string): StyleTagElement | undefined;

  /** Create a detached `<style>`. */
  createStyle(): StyleTagElement;

  /** Attach a style element to `<head>`. */
  append(element: StyleTagElement): void;

  /** Detach a style element. */
  remove(element: StyleTagElement): void;
}

/** Options for {@link useStyleTag}. */
export interface UseStyleTagOptions {
  /**
   * Element `id`. Instances sharing an id share one element (last write
   * wins, first unload removes it), so give every independent style sheet
   * its own stable id. The default is a constant rather than a generated
   * counter because generated ids differ between server and client.
   *
   * @default "vize-style"
   */
  readonly id?: string;

  /**
   * `media` attribute.
   *
   * @default undefined
   */
  readonly media?: string;

  /**
   * CSP nonce.
   *
   * @default undefined
   */
  readonly nonce?: string;

  /**
   * Attach the style element when created.
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * Document capability for alternate runtimes and tests.
   *
   * @default window.document's `<head>`
   */
  readonly host?: MaybeRefOrGetter<StyleTagHost | null | undefined>;
}

/** Reactive state and actions returned by {@link useStyleTag}. */
export interface StyleTagControls {
  /** Element id in use. */
  readonly id: string;

  /**
   * Style sheet text. Writable; when a getter was passed, the next getter
   * change overwrites manual assignments.
   */
  readonly css: Ref<string>;

  /** Whether the style element is attached. */
  readonly loaded: Readonly<Ref<boolean>>;

  /**
   * Attach (or adopt an existing element with the same id) and apply `css`.
   *
   * @returns Whether a style element is attached afterwards.
   */
  readonly load: () => boolean;

  /** Remove the style element. */
  readonly unload: () => void;
}

function browserStyleHost(): StyleTagHost | undefined {
  if (typeof window === "undefined") return undefined;
  const { document, HTMLStyleElement } = window;
  return {
    findStyle: (id) => {
      const found = document.getElementById(id);
      return found instanceof HTMLStyleElement ? found : undefined;
    },
    createStyle: () => document.createElement("style"),
    append: (element) => {
      if (element instanceof HTMLStyleElement) document.head.append(element);
    },
    remove: (element) => {
      if (element instanceof HTMLStyleElement) element.remove();
    },
  };
}

/**
 * Inject a reactive `<style>` element.
 *
 * The element's text follows `css`. An existing `<style>` with the same
 * id — for example one emitted in server-rendered HTML — is adopted instead
 * of duplicated, which keeps hydration free of flashes. The element is
 * removed when the owning reactive scope stops or `unload` is called.
 *
 * Server rendering: nothing is injected and `loaded` is false. Emit the
 * server-side `<style id>` through the application's head manager using the
 * same id to make the client adopt it.
 *
 * @example
 * ```ts
 * const { css } = useStyleTag(() => `:root { --accent: ${accent.value} }`, { id: "theme" });
 * ```
 *
 * @param css Reactive style sheet text.
 * @param options Id, attributes, timing, and capability.
 * @default options {}
 * @returns Style state and actions.
 */
export function useStyleTag(
  css: MaybeRefOrGetter<string>,
  options: UseStyleTagOptions = {},
): StyleTagControls {
  const id = options.id ?? "vize-style";
  const state: Ref<string> = isRef(css) ? css : ref(toValue(css));
  const loaded = ref(false);
  let element: StyleTagElement | undefined;

  const resolveHost = (): StyleTagHost | undefined =>
    options.host === undefined ? browserStyleHost() : (toValue(options.host) ?? undefined);

  if (typeof css === "function") {
    watch(css, (next) => {
      state.value = next;
    });
  }

  const load = (): boolean => {
    if (element) return true;
    const host = resolveHost();
    if (!host) return false;
    const existing = host.findStyle(id);
    const target = existing ?? host.createStyle();
    if (!existing) {
      target.setAttribute("id", id);
      if (options.media !== undefined) target.setAttribute("media", options.media);
      if (options.nonce !== undefined) target.setAttribute("nonce", options.nonce);
    }
    if (target.textContent !== state.value) target.textContent = state.value;
    if (!existing) host.append(target);
    element = target;
    loaded.value = true;
    return true;
  };

  const unload = (): void => {
    if (element) resolveHost()?.remove(element);
    element = undefined;
    loaded.value = false;
  };

  watch(
    state,
    (next) => {
      if (element && element.textContent !== next) element.textContent = next;
    },
    { flush: "sync" },
  );

  if (options.immediate ?? true) load();
  tryOnScopeDispose(unload);

  return { id, css: state, loaded: readonly(loaded), load, unload };
}
