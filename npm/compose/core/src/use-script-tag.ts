import { readonly, ref, shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** `<script>` subset used by {@link useScriptTag}. */
export interface ScriptTagElement extends EventTarget {
  /** Read an attribute. */
  getAttribute(name: string): string | null;

  /** Write an attribute. */
  setAttribute(name: string, value: string): void;
}

/** Document capability used by {@link useScriptTag}. */
export interface ScriptTagHost {
  /** Find an existing `<script>` whose `src` attribute equals `src`. */
  findScript(src: string): ScriptTagElement | undefined;

  /** Create a detached `<script>`. */
  createScript(): ScriptTagElement;

  /** Attach a script to `<head>`, starting its download. */
  append(element: ScriptTagElement): void;

  /** Detach a script. */
  remove(element: ScriptTagElement): void;
}

/** Loading state of the script. */
export type ScriptTagStatus = "idle" | "loading" | "loaded" | "error";

/** Discriminated outcome of {@link ScriptTagControls.load}. */
export type ScriptTagResult =
  | {
      /** The script executed. */
      readonly status: "loaded";
      /** Script element. */
      readonly element: ScriptTagElement;
    }
  | {
      /** The script failed to load. */
      readonly status: "error";
      /** Script element. */
      readonly element: ScriptTagElement;
    }
  | {
      /** No document is available (server rendering) or the load was superseded. */
      readonly status: "unsupported" | "cancelled";
    };

/** Options for {@link useScriptTag}. */
export interface UseScriptTagOptions {
  /**
   * Called once the script has loaded.
   *
   * @default undefined
   */
  readonly onLoaded?: (element: ScriptTagElement) => void;

  /**
   * Load when created and whenever `src` changes.
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * Set the `async` attribute.
   *
   * @default true
   */
  readonly async?: boolean;

  /**
   * Set the `defer` attribute.
   *
   * @default false
   */
  readonly defer?: boolean;

  /**
   * Script `type` (for example `"module"`).
   *
   * @default "text/javascript"
   */
  readonly type?: string;

  /**
   * CORS mode (`crossorigin` attribute).
   *
   * @default undefined
   */
  readonly crossOrigin?: "anonymous" | "use-credentials";

  /**
   * Referrer policy.
   *
   * @default undefined
   */
  readonly referrerPolicy?: ReferrerPolicy;

  /**
   * Set the `nomodule` attribute.
   *
   * @default false
   */
  readonly noModule?: boolean;

  /**
   * CSP nonce.
   *
   * @default undefined
   */
  readonly nonce?: string;

  /**
   * Subresource integrity hash.
   *
   * @default undefined
   */
  readonly integrity?: string;

  /**
   * Additional attributes.
   *
   * @default {}
   */
  readonly attributes?: Readonly<Record<string, string>>;

  /**
   * Remove a script created by this composable when the owning scope stops.
   *
   * @default true
   */
  readonly removeOnDispose?: boolean;

  /**
   * Document capability for alternate runtimes and tests.
   *
   * @default window.document's `<head>`
   */
  readonly host?: MaybeRefOrGetter<ScriptTagHost | null | undefined>;
}

/** Reactive state and actions returned by {@link useScriptTag}. */
export interface ScriptTagControls {
  /** Loading state. */
  readonly status: Readonly<Ref<ScriptTagStatus>>;

  /** Script element in use, if any. */
  readonly element: Readonly<ShallowRef<ScriptTagElement | undefined>>;

  /**
   * Load the script (or reuse an existing tag with the same `src`).
   * Concurrent calls share one load; `unload` and scope disposal resolve a
   * pending load as `"cancelled"`.
   *
   * @returns The discriminated outcome; never rejects.
   */
  readonly load: () => Promise<ScriptTagResult>;

  /** Remove a script created by this composable and reset the state. */
  readonly unload: () => void;
}

const STATUS_ATTRIBUTE = "data-vize-script-status";

function browserScriptHost(): ScriptTagHost | undefined {
  if (typeof window === "undefined") return undefined;
  const { document, HTMLScriptElement, CSS } = window;
  return {
    findScript: (src) => {
      const found = document.querySelector(`script[src="${CSS.escape(src)}"]`);
      return found instanceof HTMLScriptElement ? found : undefined;
    },
    createScript: () => document.createElement("script"),
    append: (element) => {
      if (element instanceof HTMLScriptElement) document.head.append(element);
    },
    remove: (element) => {
      if (element instanceof HTMLScriptElement) element.remove();
    },
  };
}

/**
 * Inject a `<script>` tag and track its loading state.
 *
 * An existing tag with the same `src` is reused instead of duplicated: if
 * it was created by another `useScriptTag` call its state is shared through
 * a `data-vize-script-status` attribute; a tag without that attribute (for
 * example emitted in server-rendered HTML) is assumed to have executed.
 * Changing the reactive `src` unloads the previous script and, with
 * `immediate`, loads the new one.
 *
 * Server rendering: no document exists, so nothing is injected, `status`
 * stays `"idle"`, and `load` resolves `"unsupported"`. Emit server-side
 * script tags through the application's head manager. Scripts created by
 * this composable are removed when the owning scope stops (configurable);
 * already-executed code cannot be unloaded.
 *
 * @example
 * ```ts
 * const { status } = useScriptTag("https://example.com/widget.js", {
 *   onLoaded: () => window.Widget.init(),
 * });
 * ```
 *
 * @param src Reactive script URL.
 * @param options Attributes, lifecycle, and capability.
 * @default options {}
 * @returns Loading state and actions.
 */
export function useScriptTag(
  src: MaybeRefOrGetter<string>,
  options: UseScriptTagOptions = {},
): ScriptTagControls {
  const status = ref<ScriptTagStatus>("idle");
  const element = shallowRef<ScriptTagElement | undefined>(undefined);
  let created = false;
  let pending: Promise<ScriptTagResult> | undefined;
  let generation = 0;
  let cancelWait: (() => void) | undefined;

  const resolveHost = (): ScriptTagHost | undefined =>
    options.host === undefined ? browserScriptHost() : (toValue(options.host) ?? undefined);

  const configure = (script: ScriptTagElement, url: string): void => {
    script.setAttribute("type", options.type ?? "text/javascript");
    if (options.async ?? true) script.setAttribute("async", "");
    if (options.defer ?? false) script.setAttribute("defer", "");
    if (options.noModule ?? false) script.setAttribute("nomodule", "");
    if (options.crossOrigin !== undefined) script.setAttribute("crossorigin", options.crossOrigin);
    if (options.referrerPolicy !== undefined) {
      script.setAttribute("referrerpolicy", options.referrerPolicy);
    }
    if (options.nonce !== undefined) script.setAttribute("nonce", options.nonce);
    if (options.integrity !== undefined) script.setAttribute("integrity", options.integrity);
    for (const [name, value] of Object.entries(options.attributes ?? {})) {
      script.setAttribute(name, value);
    }
    script.setAttribute(STATUS_ATTRIBUTE, "loading");
    script.setAttribute("src", url);
  };

  const waitFor = (script: ScriptTagElement, current: number): Promise<ScriptTagResult> =>
    new Promise((resolve) => {
      const finish = (outcome: "loaded" | "error"): void => {
        cancelWait = undefined;
        script.removeEventListener("load", onLoad);
        script.removeEventListener("error", onError);
        script.setAttribute(STATUS_ATTRIBUTE, outcome);
        if (current !== generation) {
          resolve({ status: "cancelled" });
          return;
        }
        status.value = outcome;
        if (outcome === "loaded") options.onLoaded?.(script);
        resolve({ status: outcome, element: script });
      };
      const onLoad = (): void => finish("loaded");
      const onError = (): void => finish("error");
      script.addEventListener("load", onLoad);
      script.addEventListener("error", onError);
      cancelWait = () => {
        script.removeEventListener("load", onLoad);
        script.removeEventListener("error", onError);
        resolve({ status: "cancelled" });
      };
    });

  const load = (): Promise<ScriptTagResult> => {
    if (pending) return pending;
    const host = resolveHost();
    if (!host) return Promise.resolve({ status: "unsupported" });
    const current = generation;
    const url = toValue(src);
    const existing = host.findScript(url);
    if (existing) {
      element.value = existing;
      const state = existing.getAttribute(STATUS_ATTRIBUTE);
      if (state === null || state === "loaded") {
        status.value = "loaded";
        options.onLoaded?.(existing);
        return Promise.resolve({ status: "loaded", element: existing });
      }
      if (state === "error") {
        status.value = "error";
        return Promise.resolve({ status: "error", element: existing });
      }
    }
    const script = existing ?? host.createScript();
    status.value = "loading";
    element.value = script;
    const settled = waitFor(script, current).finally(() => {
      if (pending === settled) pending = undefined;
    });
    pending = settled;
    if (!existing) {
      created = true;
      configure(script, url);
      host.append(script);
    }
    return settled;
  };

  const unload = (): void => {
    generation += 1;
    pending = undefined;
    cancelWait?.();
    cancelWait = undefined;
    const script = element.value;
    if (script && created) resolveHost()?.remove(script);
    created = false;
    element.value = undefined;
    status.value = "idle";
  };

  if (options.immediate ?? true) {
    watch(
      () => toValue(src),
      (_next, previous) => {
        if (previous !== undefined) unload();
        void load();
      },
      { immediate: true, flush: "sync" },
    );
  }

  tryOnScopeDispose(() => {
    if (options.removeOnDispose ?? true) {
      unload();
    } else {
      generation += 1;
      cancelWait?.();
    }
  });

  return { status: readonly(status), element, load, unload };
}
