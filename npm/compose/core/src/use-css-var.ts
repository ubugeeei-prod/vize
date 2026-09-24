import { computed, ref, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref } from "vue";

/** Style declaration subset used by {@link useCssVar}. */
export interface CssVarStyle {
  /** Read a property value. */
  getPropertyValue(name: string): string;

  /** Write a property value. */
  setProperty(name: string, value: string | null): void;

  /** Remove a property. */
  removeProperty(name: string): string;
}

/** Element subset used by {@link useCssVar}. */
export interface CssVarTarget {
  /** Inline style declaration. */
  readonly style: CssVarStyle;
}

/** Minimal `MutationObserver` instance. */
export interface CssVarObserver {
  /** Start observing `target`. */
  observe(target: CssVarTarget, options: { attributes: true; attributeFilter: string[] }): void;

  /** Stop observing. */
  disconnect(): void;
}

/** Capabilities used by {@link useCssVar}. */
export interface CssVarHost {
  /** Resolved (computed) style of a target. */
  getComputedStyle(target: CssVarTarget): Pick<CssVarStyle, "getPropertyValue">;

  /** Target used when none is given (the document root element). */
  readonly documentElement: CssVarTarget;

  /** Observer factory used when `observe` is enabled. */
  readonly createObserver?: (callback: () => void) => CssVarObserver;
}

/** Options for {@link useCssVar}. */
export interface UseCssVarOptions {
  /**
   * Value used while the variable is unset and during server rendering.
   *
   * @default ""
   */
  readonly initialValue?: string;

  /**
   * Re-read the variable when the target's `style` or `class` attribute
   * changes (MutationObserver).
   *
   * @default false
   */
  readonly observe?: boolean;

  /**
   * Style capability for alternate runtimes and tests.
   *
   * @default window.getComputedStyle, document.documentElement, and MutationObserver
   */
  readonly host?: MaybeRefOrGetter<CssVarHost | null | undefined>;
}

/** Reactive state returned by {@link useCssVar}. */
export interface CssVarControls {
  /** Writable variable value; assignments are written as inline style. */
  readonly value: Ref<string>;

  /** Whether a style capability and target are attached. */
  readonly supported: ComputedRef<boolean>;
}

function browserCssVarHost(): CssVarHost | undefined {
  if (typeof window === "undefined") return undefined;
  const view = window;
  return {
    getComputedStyle: (target) =>
      target instanceof view.Element ? view.getComputedStyle(target) : target.style,
    documentElement: view.document.documentElement,
    createObserver: (callback) => {
      const observer = new view.MutationObserver(callback);
      return {
        observe: (target, init) => {
          if (target instanceof view.Node) observer.observe(target, init);
        },
        disconnect: () => observer.disconnect(),
      };
    },
  };
}

function assertVariableName(name: string): void {
  if (!name.startsWith("--")) {
    throw new TypeError(
      `[VIZE_COMPOSE_CSS_VAR_INVALID_NAME] custom property names must start with "--"; received ${JSON.stringify(name)}`,
    );
  }
}

/**
 * Read and write a CSS custom property reactively.
 *
 * The value is read from the target's computed style whenever the
 * reactive name, target, or host changes (and, with `observe`, when its
 * `style`/`class` attribute changes). Assigning `value` writes an inline
 * `style.setProperty`; assigning `""` removes the inline declaration. Values
 * read from the element are never written back, so a stylesheet-provided
 * variable is not frozen into inline style.
 *
 * Server rendering: nothing is read or written; `value` holds
 * `initialValue`. Keep server markup and the first client render identical
 * by rendering from `initialValue` until mounted. The observer and
 * watchers are released with the owning reactive scope.
 *
 * @example
 * ```ts
 * const accent = useCssVar("--accent", rootRef, { initialValue: "#0af" });
 * accent.value.value = "#f60";
 * ```
 *
 * @param name Reactive custom property name, starting with `--`.
 * @param target Reactive element; defaults to the document root element.
 * @param options Initial value, observation, and capability.
 * @default options {}
 * @returns The writable variable value.
 * @throws {TypeError} `[VIZE_COMPOSE_CSS_VAR_INVALID_NAME]` when the name does not start with `--`.
 */
export function useCssVar(
  name: MaybeRefOrGetter<string>,
  target?: MaybeRefOrGetter<CssVarTarget | null | undefined>,
  options: UseCssVarOptions = {},
): CssVarControls {
  const initialValue = options.initialValue ?? "";
  assertVariableName(toValue(name));
  const value = ref(initialValue);
  let lastRead: string | undefined;

  const resolveHost = (): CssVarHost | undefined =>
    options.host === undefined ? browserCssVarHost() : (toValue(options.host) ?? undefined);
  const resolveTarget = (host: CssVarHost | undefined): CssVarTarget | undefined => {
    if (!host) return undefined;
    return target === undefined ? host.documentElement : (toValue(target) ?? undefined);
  };

  const read = (host: CssVarHost, element: CssVarTarget, variable: string): void => {
    const current = host.getComputedStyle(element).getPropertyValue(variable).trim();
    const next = current === "" ? initialValue : current;
    lastRead = next;
    value.value = next;
  };

  watch(
    () => {
      const host = resolveHost();
      return [host, resolveTarget(host), toValue(name)] as const;
    },
    ([host, element, variable], _previous, onCleanup) => {
      assertVariableName(variable);
      if (!host || !element) return;
      read(host, element, variable);
      if (!(options.observe ?? false) || !host.createObserver) return;
      const observer = host.createObserver(() => read(host, element, variable));
      observer.observe(element, { attributes: true, attributeFilter: ["style", "class"] });
      onCleanup(() => observer.disconnect());
    },
    { immediate: true, flush: "sync" },
  );

  watch(
    value,
    (next) => {
      if (next === lastRead) return;
      lastRead = undefined;
      const element = resolveTarget(resolveHost());
      if (!element) return;
      const variable = toValue(name);
      if (next === "") element.style.removeProperty(variable);
      else element.style.setProperty(variable, next);
    },
    { flush: "sync" },
  );

  return {
    value,
    supported: computed(() => resolveTarget(resolveHost()) !== undefined),
  };
}
