import { getCurrentInstance, useId } from "vue";

/** Options for {@link useIdGenerator}. */
export interface UseIdGeneratorOptions {
  /**
   * Namespace placed in front of every id.
   *
   * @default "vize"
   */
  readonly prefix?: string;

  /**
   * Stable seed identifying this generator. Defaults to Vue's `useId()`
   * inside a component (identical on server and hydrating client), or to
   * `"root"` outside one.
   *
   * @default useId() or "root"
   */
  readonly seed?: string;
}

/** Id factory returned by {@link useIdGenerator}. */
export type IdGenerator = (hint?: string) => string;

function sanitize(part: string): string {
  return part.replaceAll(/[^\w-]/g, "-");
}

/**
 * Create an SSR-stable id factory for one component.
 *
 * Each call to the returned function yields
 * `<prefix>-<seed>-<hint>-<n>` with a per-generator counter, so as long as
 * ids are generated in the same order on the server and the client (for
 * example during `setup`), the markup hydrates without mismatches. The seed
 * comes from Vue's `useId()` inside components, which is itself stable across
 * SSR and hydration; outside components pass an explicit `seed`. Characters
 * outside `[A-Za-z0-9_-]` are replaced so ids are valid in selectors.
 *
 * @example
 * ```ts
 * const nextId = useIdGenerator({ prefix: "field" });
 * const inputId = nextId("input"); // "field-v-0-input-0"
 * ```
 *
 * @param options Prefix and seed.
 * @default options {}
 * @returns The id factory.
 */
export function useIdGenerator(options: UseIdGeneratorOptions = {}): IdGenerator {
  const seed = options.seed ?? (getCurrentInstance() === null ? "root" : useId());
  const base = `${sanitize(options.prefix ?? "vize")}-${sanitize(seed)}`;
  let counter = 0;
  return (hint = "id") => {
    const id = `${base}-${sanitize(hint)}-${counter}`;
    counter += 1;
    return id;
  };
}
