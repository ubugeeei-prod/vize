/** JSON-compatible value accepted across plugin process boundaries. */
export type JsonValue =
  | null
  | boolean
  | number
  | string
  | readonly JsonValue[]
  | { readonly [key: string]: JsonValue };
/** Authored hook identity and every input captured by its fingerprint. */
export interface HookIdentity {
  name: string;
  version: string;
  cacheInputs?: readonly { name: string; value: string }[];
}
/** Runtime arrays are frozen; declarations match generated native descriptors. */
export interface NativeHook extends Omit<HookIdentity, "cacheInputs"> {
  readonly cacheInputs?: { name: string; value: string }[];
  readonly fingerprint: string;
  run(batchJson: string): string;
}
/** Precanonical element and attribute payload delivered to transform hooks. */
export interface StaticAttributeTransformBatch {
  readonly schema: 1;
  readonly stage: "s2-precanonical-static-attributes";
  readonly plugin: string;
  readonly file: string;
  readonly nodes: readonly {
    readonly id: number;
    readonly kind: "ui.element";
    readonly tag: string;
    readonly namespace: string;
    readonly attrs: readonly { readonly name: string; readonly value: string | null }[];
  }[];
}
/** Static attribute names that a precanonical transform may replace. */
export type StaticAttributeName =
  | "class"
  | "id"
  | "title"
  | "role"
  | "alt"
  | `data-${string}`
  | `aria-${string}`;
/** Deterministic replacement for one static element attribute. */
export interface StaticAttributeEdit {
  kind: "replace-static-attribute";
  node: number;
  name: StaticAttributeName;
  value: string | null;
}
/** Validate and fingerprint a static-attribute transform hook. */
export declare function defineTransformPlugin(
  definition: HookIdentity & {
    transform(batch: StaticAttributeTransformBatch): readonly StaticAttributeEdit[];
  },
): NativeHook;
/** Compiled module output visible to formatter and output hooks. */
export interface CompiledArtifact {
  readonly code: string;
  readonly preamble: string;
  readonly ast: unknown;
  readonly map?: unknown;
  readonly helpers: readonly string[];
  readonly templates?: readonly string[] | null;
}
/** Shared host payload delivered after compilation. */
export interface OutputBatch {
  readonly schema: 1;
  readonly plugin: string;
  readonly family: "formatter" | "output";
  readonly offsetEncoding: "utf8";
  readonly compiled: CompiledArtifact;
}
/** UTF-8 replacement emitted by a formatter hook. */
export interface FormatEdit {
  start: number;
  end: number;
  text: string;
}
/** Comment-only addition emitted before or after compiled code. */
export interface OutputAddition {
  placement: "prepend" | "append";
  comment: string;
}
/** Post-compilation payload specialized for formatter hooks. */
export interface FormatterBatch extends OutputBatch {
  readonly family: "formatter";
}
/** Post-compilation payload specialized for output hooks. */
export interface OutputHookBatch extends OutputBatch {
  readonly family: "output";
}
/** Validate and fingerprint a formatter or output hook. */
export declare function defineOutputPlugin(
  definition: HookIdentity & {
    family: "formatter";
    output(batch: FormatterBatch): readonly FormatEdit[];
  },
): NativeHook & { readonly family: "formatter" };
export declare function defineOutputPlugin(
  definition: HookIdentity & {
    family: "output";
    output(batch: OutputHookBatch): readonly OutputAddition[];
  },
): NativeHook & { readonly family: "output" };
/** Syntax and existing facts delivered to a fact-provider hook. */
export interface FactProviderBatch {
  readonly schema: 1;
  readonly plugin: string;
  readonly file: string;
  readonly parents: readonly number[];
  readonly nodes: readonly import("./index.js").PluginNode[];
  readonly facts: Readonly<Record<string, readonly (readonly [string | number, JsonValue])[]>>;
}
/** Validate and fingerprint a deterministic fact-provider hook. */
export declare function defineFactProvider(
  definition: HookIdentity & {
    visit?: readonly string[];
    demands?: readonly string[];
    provides: readonly string[];
    provide(
      batch: FactProviderBatch,
    ): Readonly<Record<string, readonly (readonly [string | number, JsonValue])[]>>;
  },
): NativeHook & {
  readonly provides: string[];
  readonly visit?: string[];
  readonly demands: string[];
};
