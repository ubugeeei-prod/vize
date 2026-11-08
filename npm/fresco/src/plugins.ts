import { inject, type App as VueApp, type Component, type InjectionKey, type Plugin } from "vue";

/** Typed provider owned by a Fresco extension. */
export interface FrescoProvider<Value> {
  /** Stable Vue injection key for advanced adapter integration. */
  readonly key: InjectionKey<Value>;
  /** Install a value while a plugin is configuring the application. */
  provide(app: VueApp, value: Value): void;
  /** Read the nearest value from component setup. */
  use(): Value;
}

/** Declarative component and provider setup for one Fresco plugin. */
export interface FrescoPluginDefinition {
  /** Optional global components keyed by their public template names. */
  readonly components?: Readonly<Record<string, Component>>;
  /** Additional provider or application setup performed during installation. */
  readonly setup?: (app: VueApp) => void;
}

/**
 * Define a typed extension provider with an optional fallback value.
 *
 * Fallback presence follows argument presence: explicitly passing `undefined`
 * is a valid fallback for value types that include it.
 */
export function defineFrescoProvider<Value>(
  description: string,
  fallback?: Value,
): FrescoProvider<Value> {
  const key: InjectionKey<Value> = Symbol(description);
  const hasFallback = arguments.length >= 2;
  const missing = Symbol("missing-fresco-provider");

  return {
    key,
    provide(app, value) {
      app.provide(key, value);
    },
    use() {
      const value = inject(key as InjectionKey<Value | typeof missing>, missing);
      if (value !== missing) return value;
      if (hasFallback) return fallback as Value;
      throw new Error(`Fresco provider ${JSON.stringify(description)} is not installed`);
    },
  };
}

/** Define a Vue plugin that registers Fresco components and typed providers. */
export function defineFrescoPlugin(definition: Readonly<FrescoPluginDefinition>): Plugin<[]> {
  return {
    install(app) {
      for (const [name, component] of Object.entries(definition.components ?? {})) {
        app.component(name, component);
      }
      definition.setup?.(app);
    },
  };
}
