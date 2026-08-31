import { computed, getCurrentInstance, inject, provide, toValue, useId as useVueId } from "vue";
import type { ComputedRef, InjectionKey, MaybeRefOrGetter } from "vue";

declare const deterministicIdBrand: unique symbol;

/** A validated DOM identifier created or accepted by the deterministic ID system. */
export type DeterministicId = string & {
  readonly [deterministicIdBrand]: "DeterministicId";
};

/** Stable request, island, or subtree seed accepted by an ID scope. */
export type DeterministicIdSeed = string | number;

/** Options for a root deterministic ID scope. */
export interface DeterministicIdScopeOptions {
  /**
   * Namespace prefix used by every ID in the scope.
   *
   * @default "vize"
   */
  readonly prefix?: string;

  /** Stable request, island, or application seed. */
  readonly seed: DeterministicIdSeed;
}

/** Options for a nested deterministic ID scope. */
export interface DeterministicIdChildScopeOptions {
  /**
   * Namespace prefix for the child scope.
   *
   * @default The parent scope prefix
   */
  readonly prefix?: string;

  /** Stable seed that identifies the child subtree. */
  readonly seed: DeterministicIdSeed;
}

/** One request-local namespace that allocates stable, collision-resistant DOM IDs. */
export interface DeterministicIdScope {
  /** Validated namespace prefix. */
  readonly prefix: string;

  /** Complete namespace shared by IDs allocated from this scope. */
  readonly namespace: string;

  /**
   * Allocate the next ID during component setup.
   *
   * The sequence is intentionally independent from child-scope allocation so
   * adding a nested provider cannot renumber sibling control IDs.
   */
  readonly nextId: (hint?: string) => DeterministicId;

  /** Create a request-local child namespace without sharing its ID counter. */
  readonly createChild: (options: DeterministicIdChildScopeOptions) => DeterministicIdScope;
}

/** Options for {@link useDeterministicId}. */
export interface DeterministicIdOptions {
  /**
   * Consumer-owned ID. `null` and `undefined` select the generated fallback.
   *
   * @default undefined
   */
  readonly id?: MaybeRefOrGetter<string | null | undefined>;

  /**
   * Human-readable role appended to a generated ID.
   *
   * @default "id"
   */
  readonly hint?: string;

  /**
   * Prefix used when no {@link IdProvider} is present.
   *
   * @default "vize"
   */
  readonly prefix?: string;
}

const deterministicIdScopeKey: InjectionKey<DeterministicIdScope> = Symbol(
  "VizeUiDeterministicIdScope",
);
const safePrefix = /^[A-Za-z][A-Za-z0-9_-]*$/;
const safeSegment = /^[A-Za-z0-9][A-Za-z0-9_-]*$/;

/**
 * Validate and brand a consumer-owned HTML ID.
 *
 * HTML permits a broad character set, but forbids an empty value and ASCII
 * whitespace. This contract additionally rejects ASCII controls. Consumers
 * remain free to use Unicode and punctuation; CSS selectors may need
 * `CSS.escape()` for those IDs.
 */
export function toDeterministicId(value: string): DeterministicId {
  if (!isValidHtmlId(value)) {
    throw new Error(
      "VIZE_UI_ID_VALUE: an id must be non-empty and contain no ASCII whitespace or controls",
    );
  }
  return value as DeterministicId;
}

function isValidHtmlId(value: string): boolean {
  if (value.length === 0) return false;
  for (let index = 0; index < value.length; index++) {
    const code = value.charCodeAt(index);
    if (code <= 0x20 || code === 0x7f) return false;
  }
  return true;
}

/** Append a validated semantic part to an existing DOM ID. */
export function deriveDeterministicId(id: string, part: string): DeterministicId {
  const base = toDeterministicId(id);
  return toDeterministicId(`${base}-${normalizeSegment(part, "part")}`);
}

/** Create an isolated deterministic ID scope for an application or SSR request. */
export function createDeterministicIdScope(
  options: DeterministicIdScopeOptions,
): DeterministicIdScope {
  const prefix = normalizePrefix(options.prefix ?? "vize");
  const seed = normalizeSeed(options.seed);
  return createScope(prefix, `${prefix}-${seed}`);
}

/**
 * Allocate one stable ID for the current component instance.
 *
 * Generated IDs use the nearest provider scope. Without a provider, Vue's
 * hydration-stable `useId()` sequence supplies the seed, including a safe
 * application `idPrefix`. The generated fallback is allocated once; changing
 * an explicit reactive ID back to `undefined` restores that same fallback.
 */
export function useDeterministicId(
  options: DeterministicIdOptions = {},
): ComputedRef<DeterministicId> {
  if (getCurrentInstance() === null) {
    throw new Error("VIZE_UI_ID_SETUP: useDeterministicId() must run during component setup");
  }

  // Always consume Vue's instance-local sequence. This keeps later Vue IDs in
  // the same order when a component moves into or out of an IdProvider.
  const vueId = useVueId();
  const scope = useOptionalDeterministicIdScope();
  const hint = normalizeSegment(options.hint ?? "id", "hint");
  const generated =
    scope?.nextId(hint) ??
    toDeterministicId(
      `${normalizePrefix(options.prefix ?? "vize")}-${normalizeSeed(vueId)}-${hint}`,
    );

  return computed(() => {
    const explicit = options.id === undefined ? undefined : toValue(options.id);
    return explicit === null || explicit === undefined ? generated : toDeterministicId(explicit);
  });
}

/** @internal Provide the scope owned by `IdProvider.vue`. */
export function provideDeterministicIdScope(scope: DeterministicIdScope): DeterministicIdScope {
  provide(deterministicIdScopeKey, scope);
  return scope;
}

/** @internal Read the nearest provider without requiring one. */
export function useOptionalDeterministicIdScope(): DeterministicIdScope | undefined {
  return inject(deterministicIdScopeKey, undefined);
}

function createScope(prefix: string, namespace: string): DeterministicIdScope {
  let idSequence = 0;
  let childSequence = 0;

  const scope: DeterministicIdScope = {
    prefix,
    namespace,
    nextId: (hint = "id") =>
      toDeterministicId(`${namespace}-${normalizeSegment(hint, "hint")}-${idSequence++}`),
    createChild: (options) => {
      const childPrefix = options.prefix === undefined ? prefix : normalizePrefix(options.prefix);
      const childSeed = normalizeSeed(options.seed);
      const childIndex = childSequence++;
      const inheritedNamespace = `${namespace}-scope-${childIndex}-${childSeed}`;
      const childNamespace =
        childPrefix === prefix ? inheritedNamespace : `${childPrefix}-${inheritedNamespace}`;
      return createScope(childPrefix, childNamespace);
    },
  };

  return Object.freeze(scope);
}

function normalizePrefix(value: string): string {
  if (!safePrefix.test(value)) {
    throw new Error(
      "VIZE_UI_ID_PREFIX: a prefix must start with an ASCII letter and contain only letters, digits, _ or -",
    );
  }
  return value;
}

function normalizeSeed(value: DeterministicIdSeed): string {
  if (typeof value === "number") {
    if (!Number.isSafeInteger(value) || value < 0) {
      throw new Error("VIZE_UI_ID_SEED: a numeric seed must be a non-negative safe integer");
    }
    return String(value);
  }
  return normalizeSegment(value, "seed");
}

function normalizeSegment(value: string, role: "hint" | "part" | "seed"): string {
  if (!safeSegment.test(value)) {
    throw new Error(
      `VIZE_UI_ID_${role.toUpperCase()}: ${role} must contain only ASCII letters, digits, _ or - and may not start with punctuation`,
    );
  }
  return value;
}
