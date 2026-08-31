import { inject, provide } from "vue";
import type { InjectionKey } from "vue";

const missingContext = Symbol("missing component context");

/** A typed provider and consumer contract for one component family. */
export interface ComponentContext<Value> {
  /** Human-readable context name used by diagnostics and developer tools. */
  readonly name: string;
  /** Public injection key for application-level adapters and test harnesses. */
  readonly key: InjectionKey<Value>;
  /** Provides a value from component setup and returns that same value. */
  readonly provide: (value: Value) => Value;
  /** Reads the nearest value or throws a stable missing-provider diagnostic. */
  readonly use: () => Value;
  /** Reads the nearest value when the provider is intentionally optional. */
  readonly useOptional: () => Value | undefined;
}

/**
 * Creates an immutable typed context for a compound component family.
 *
 * A private sentinel distinguishes a missing provider from a provider whose
 * value is explicitly `undefined`.
 */
export function createContext<Value>(name: string): ComponentContext<Value> {
  const contextName = name.trim();
  if (contextName.length === 0) {
    throw new Error("VIZE_UI_CONTEXT_NAME: context name must not be empty");
  }

  const key = Symbol(contextName) as InjectionKey<Value>;
  const internalKey = key as InjectionKey<Value | typeof missingContext>;
  const read = () => inject(internalKey, missingContext);

  return Object.freeze({
    name: contextName,
    key,
    provide: (value: Value) => {
      provide(key, value);
      return value;
    },
    use: () => {
      const value = read();
      if (value === missingContext) {
        throw new Error(`VIZE_UI_CONTEXT_MISSING: ${contextName} requires a matching provider`);
      }
      return value;
    },
    useOptional: () => {
      const value = read();
      return value === missingContext ? undefined : value;
    },
  });
}
