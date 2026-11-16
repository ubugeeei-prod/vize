/** A `[plugin/<code>] <subject>: <message>` refusal shared with the native verifier. */
export declare class VizePluginPlanError extends Error {
  readonly code: string;
  readonly subject: string;
}

export interface RegisteredCapability {
  readonly schemaVersions: readonly number[];
  readonly apiVersions: readonly number[];
  readonly consumers: readonly string[];
  readonly callbacks: readonly string[];
  readonly admittedBy: readonly string[];
}

export interface PluginRegistry {
  readonly planSchema: number;
  readonly apiVersions: readonly number[];
  readonly products: readonly string[];
  readonly backends: readonly string[];
  readonly admittedBackends: readonly string[];
  readonly capabilities: Readonly<Record<string, RegisteredCapability>>;
  readonly lintDocuments: readonly string[];
  readonly recordKinds: readonly string[];
  readonly factGroups: readonly string[];
  readonly reservedDirectives: readonly string[];
}

export declare const PLUGIN_REGISTRY: PluginRegistry;
export declare const PLUGIN_PLAN_SCHEMA: number;
export declare const PLUGIN_ENTRY: unique symbol;

export type PlanData =
  | null
  | boolean
  | number
  | string
  | readonly PlanData[]
  | { readonly [key: string]: PlanData };

export interface PlannedCapability {
  readonly data: { readonly [key: string]: PlanData };
  readonly id: string;
  readonly key: string;
  readonly required: boolean;
  readonly schemaVersion: number;
}

export interface PlannedPlugin {
  readonly after: readonly string[];
  readonly apiVersion: 1 | 2;
  readonly backend: string;
  readonly before: readonly string[];
  readonly cacheInputs: readonly { readonly name: string; readonly value: string }[] | null;
  readonly capabilities: readonly PlannedCapability[];
  readonly fingerprint: string;
  readonly index: number;
  readonly name: string;
  readonly options: { readonly [key: string]: PlanData };
  readonly permissions: readonly { readonly [key: string]: PlanData }[];
  readonly skipped: readonly { readonly id: string; readonly schemaVersion: number }[];
  readonly stability: string;
  readonly supports?: {
    readonly outputs: readonly string[];
    readonly runtimes: readonly string[];
    readonly vue: readonly string[];
  };
  readonly version: string;
}

export interface PluginPlan {
  readonly claims: readonly { readonly key: string; readonly owner: string }[];
  readonly planSchema: number;
  readonly plugins: readonly PlannedPlugin[];
}

export interface ResolvedPluginPlan {
  readonly plan: PluginPlan;
  /** Canonical serialization; `digest` is its sha256. */
  readonly json: string;
  readonly digest: string;
  /** Callbacks keyed by {@link callbackKey}; never serialized. */
  readonly callbacks: ReadonlyMap<string, (...args: never[]) => unknown>;
  /** The authored entry of every planned plugin, by name. */
  readonly entries: ReadonlyMap<string, unknown>;
}

export declare function resolvePluginPlan(
  entries: readonly unknown[] | undefined,
  host?: { runtime?: "node" | "bun" },
): ResolvedPluginPlan;

export declare function callbackKey(
  plugin: string,
  capability: string,
  key: string,
  name: string,
): string;

export declare function isEmptyPluginPlan(plan: PluginPlan): boolean;
