import type { FactGroups } from "./facts.js";

/** Schema version accepted by rule-plugin batch payloads. */
export declare const BATCH_SCHEMA: 1;
/** Host-owned syntax node exposed to deterministic rule plugins. */
export interface PluginNode {
  readonly id: number;
  readonly parent?: number;
  readonly kind: string;
  readonly name?: string;
  readonly value?: string;
  readonly attrs?: readonly (readonly [string, string | null])[];
  readonly alias?: {
    readonly value: string;
    readonly key: string | null;
    readonly index: string | null;
  };
}
/** Template binding visible at a node and the role that introduced it. */
export interface ScopeEntry {
  readonly name: string;
  readonly position: "value" | "key" | "index" | "slot";
}
/** Read-only host facts and reporting operations available to one rule. */
export interface RuleContext {
  readonly nodes: readonly PluginNode[];
  facts(name: "templateScopes"): ReadonlyMap<number, readonly ScopeEntry[]>;
  facts<K extends keyof FactGroups>(name: K): FactGroups[K];
  facts<T = unknown>(name: string): ReadonlyMap<string | number, T>;
  ancestors(node: PluginNode, kind: string): IterableIterator<PluginNode>;
  /** Replace the whole reported node span; ranges are assigned by the host. */
  report(node: PluginNode, message: string, fix?: string): void;
}
/** Authored rule-plugin definition before normalization and fingerprinting. */
export interface PluginDefinition {
  name: string;
  version: string;
  visit?: readonly string[];
  demands?: readonly string[];
  /** Explicit list of all captured configuration and ambient inputs. */
  cacheInputs?: readonly { name: string; value: string }[];
  rules: Readonly<Record<string, (context: RuleContext) => void>>;
}
/** Frozen at runtime. Array declarations interoperate with generated native descriptors. */
export interface Plugin extends Omit<PluginDefinition, "visit" | "demands" | "cacheInputs"> {
  readonly visit?: string[];
  readonly demands: string[];
  readonly cacheInputs?: { name: string; value: string }[];
  readonly fingerprint: string;
  run(batchJson: string): string;
}
/** Serializable syntax and fact payload delivered to a rule plugin. */
export interface VisitBatch {
  schema: 1;
  plugin?: string;
  file?: string;
  parents: number[];
  nodes: PluginNode[];
  facts: Record<string, Array<[string | number, unknown]>>;
}
/** Validate, freeze, and fingerprint an authored rule-plugin definition. */
export declare function definePlugin(definition: PluginDefinition): Plugin;
/** Run a normalized plugin against a serialized visit batch. */
export declare function runBatch(
  plugin: Plugin,
  batch: VisitBatch,
): Array<{ rule: string; node: number; message: string; fix?: string }>;

/** Run every rule against an already decoded host source. */
export declare function runRules(
  plugin: Plugin,
  source: {
    nodes: readonly PluginNode[];
    byId: ReadonlyMap<number, PluginNode>;
    facts(name: string): ReadonlyMap<string | number, unknown>;
    parent(id: number): number;
  },
): Array<{ rule: string; node: number; message: string; fix?: string }>;

export { defineFactProvider, defineOutputPlugin, defineTransformPlugin } from "./stages.js";
export { applyFixes } from "./fixes.js";

export type * from "./facts.js";
export type * from "./stages.js";
