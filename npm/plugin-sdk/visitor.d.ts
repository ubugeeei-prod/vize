import type { PluginNode } from "./index.js";
import type { JsonValue } from "./stages.js";

/** Visitor protocol version understood by this SDK. */
export declare const VISITOR_API_VERSION: 1;
/** Closed capability vocabulary accepted during visitor negotiation. */
export declare const VISITOR_CAPABILITIES: readonly [
  "sfc",
  "template",
  "script",
  "style",
  "virtual-ts",
  "diagnostics",
  "lint",
  "format",
  "devtools-trace",
];
/** One capability from the versioned visitor vocabulary. */
export type VisitorCapability = (typeof VISITOR_CAPABILITIES)[number];
/** Stable validation or negotiation failure reported by the visitor API. */
export type VisitorErrorCode =
  | "invalid-definition"
  | "invalid-name"
  | "invalid-version"
  | "unsupported-api-version"
  | "invalid-capabilities"
  | "unknown-capability"
  | "duplicate-capability"
  | "invalid-visitors"
  | "missing-visitor"
  | "undeclared-visitor"
  | "async-visitor"
  | "invalid-cache-inputs"
  | "invalid-plugin"
  | "invalid-host-contract"
  | "unsupported-capability"
  | "undeclared-capability"
  | "invalid-event"
  | "invalid-visitor-result";

/** Definition or host-contract error with a stable code and field path. */
export declare class VisitorPluginValidationError extends TypeError {
  readonly code: VisitorErrorCode;
  readonly field: string;
}

/** Half-open UTF-8 byte range used by visitor payloads. */
export interface VisitorRange {
  readonly start: number;
  readonly end: number;
  readonly encoding: "utf8";
}
/** Diagnostic delivered to a visitor with source and severity metadata. */
export interface VisitorDiagnostic {
  readonly code: string;
  readonly message: string;
  readonly severity: "error" | "warning" | "information" | "hint";
  readonly range: VisitorRange;
  readonly source?: string;
}
/** Replacement edit associated with a lint visitor payload. */
export interface VisitorFix {
  readonly range: VisitorRange;
  readonly text: string;
}
/** One parsed block in an SFC visitor payload. */
export interface VisitorSfcBlock {
  readonly kind: "template" | "script" | "script-setup" | "style" | "custom";
  readonly language: string | null;
  readonly range: VisitorRange;
  readonly attributes: Readonly<Record<string, string | null>>;
}
/** Complete authored SFC source and its parsed block inventory. */
export interface VisitorSfcPayload {
  readonly source: string;
  readonly blocks: readonly VisitorSfcBlock[];
}
/** Normalized template nodes delivered to template visitors. */
export interface VisitorTemplatePayload {
  readonly nodes: readonly PluginNode[];
}
/** Authored script source, language, and optional negotiated syntax data. */
export interface VisitorScriptPayload {
  readonly source: string;
  readonly language: "js" | "jsx" | "ts" | "tsx";
  /** Host-owned serialized syntax data. Its schema belongs to the negotiated host. */
  readonly syntax?: JsonValue;
}
/** Authored style source and its scoped or module mode. */
export interface VisitorStylePayload {
  readonly source: string;
  readonly language: string;
  readonly scoped: boolean;
  readonly module: boolean | string;
}
/** Source-to-generated range pair for virtual TypeScript. */
export interface VisitorVirtualTsMapping {
  readonly source: VisitorRange;
  readonly generated: VisitorRange;
}
/** Virtual TypeScript code, file identities, and source mappings. */
export interface VisitorVirtualTsPayload {
  readonly sourceFile: string;
  readonly virtualFile: string;
  readonly code: string;
  readonly mappings: readonly VisitorVirtualTsMapping[];
}
/** Diagnostics currently owned by the host pipeline. */
export interface VisitorDiagnosticsPayload {
  readonly diagnostics: readonly VisitorDiagnostic[];
}
/** Lint diagnostics together with their available fixes. */
export interface VisitorLintPayload extends VisitorDiagnosticsPayload {
  readonly fixes: readonly VisitorFix[];
}
/** Authored and formatted source delivered to format visitors. */
export interface VisitorFormatPayload {
  readonly source: string;
  readonly formatted: string;
}
/** One parented timing event from the devtools trace pipeline. */
export interface VisitorDevtoolsTraceEvent {
  readonly id: string;
  readonly parentId: string | null;
  readonly name: string;
  readonly stage: string;
  readonly startNs: number;
  readonly durationNs: number;
  readonly data?: JsonValue;
}
/** Ordered trace events delivered to a devtools visitor. */
export interface VisitorDevtoolsTracePayload {
  readonly events: readonly VisitorDevtoolsTraceEvent[];
}
/** Capability-to-payload mapping for every visitor event family. */
export interface VisitorPayloads {
  readonly sfc: VisitorSfcPayload;
  readonly template: VisitorTemplatePayload;
  readonly script: VisitorScriptPayload;
  readonly style: VisitorStylePayload;
  readonly "virtual-ts": VisitorVirtualTsPayload;
  readonly diagnostics: VisitorDiagnosticsPayload;
  readonly lint: VisitorLintPayload;
  readonly format: VisitorFormatPayload;
  readonly "devtools-trace": VisitorDevtoolsTracePayload;
}
/** Versioned host event specialized by one visitor capability. */
export interface VisitorEvent<C extends VisitorCapability = VisitorCapability> {
  readonly apiVersion: 1;
  readonly capability: C;
  readonly file: string;
  readonly payload: VisitorPayloads[C];
}
/** Synchronous handler for one negotiated visitor event. */
export type VisitorHandler<C extends VisitorCapability> = (event: VisitorEvent<C>) => void;
/** Exact handler map required by a capability tuple. */
export type VisitorHandlers<C extends readonly VisitorCapability[]> = {
  readonly [K in C[number]]: VisitorHandler<K>;
};
/** Captured ambient or configuration input included in a visitor fingerprint. */
export interface VisitorCacheInput {
  readonly name: string;
  readonly value: string;
}
/** Authored visitor descriptor before validation and fingerprinting. */
export interface VisitorPluginDefinition<C extends readonly VisitorCapability[]> {
  readonly apiVersion: 1;
  readonly name: string;
  readonly version: string;
  readonly capabilities: C;
  /** Explicit list of every captured configuration or ambient input. */
  readonly cacheInputs: readonly VisitorCacheInput[];
  readonly visitors: VisitorHandlers<C>;
}
/** Immutable visitor descriptor accepted by host negotiation. */
export interface VisitorPlugin<
  C extends readonly VisitorCapability[] = readonly VisitorCapability[],
> {
  readonly kind: "vize.visitor-plugin";
  readonly apiVersion: 1;
  readonly name: string;
  readonly version: string;
  readonly capabilities: C;
  readonly cacheInputs: readonly VisitorCacheInput[];
  readonly visitors: VisitorHandlers<C>;
  readonly fingerprint: string;
}
/** Visitor protocol version and capabilities supplied by a host. */
export interface VisitorHostContract<
  C extends readonly VisitorCapability[] = readonly VisitorCapability[],
> {
  readonly apiVersion: 1;
  readonly capabilities: C;
}
/** Exact capabilities retained after successful host negotiation. */
export interface NegotiatedVisitorContract<C extends readonly VisitorCapability[]> {
  readonly apiVersion: 1;
  readonly capabilities: C;
}
/** Native host adapter for a virtual-TypeScript-only visitor plugin. */
export interface VirtualTsVisitorHostPlugin {
  readonly name: string;
  readonly version: string;
  readonly fingerprint: string;
  /** Host-only serialized boundary consumed by `typeCheckWithVirtualTsVisitors`. */
  readonly run: (eventJson: string) => string;
}
/** Devtools host adapter for a trace-only visitor plugin. */
export interface DevtoolsTraceVisitorHostPlugin {
  readonly name: string;
  readonly version: string;
  readonly fingerprint: string;
  /** Host-only serialized boundary consumed by `createDevtoolsTraceRecorder`. */
  readonly run: (deliveryJson: string) => string;
}

/** Define an immutable descriptor. This does not register a runtime hook. */
export declare function defineVisitorPlugin<const C extends readonly VisitorCapability[]>(
  definition: VisitorPluginDefinition<C>,
): VisitorPlugin<C>;
/** Fail closed unless the host provides the exact API version and every declared capability. */
export declare function negotiateVisitorPlugin<const C extends readonly VisitorCapability[]>(
  plugin: VisitorPlugin<C>,
  host: VisitorHostContract,
): NegotiatedVisitorContract<C>;
/** Invoke one visitor in an author fixture; production lifecycle remains host-owned. */
export declare function runVisitorFixture<
  C extends readonly VisitorCapability[],
  K extends C[number],
>(plugin: VisitorPlugin<C>, capability: K, event: VisitorEvent<K>): void;
/**
 * Negotiate a virtual-TypeScript-only descriptor and adapt it to the native
 * type-check visitor boundary. Descriptors with any unsupported capability fail closed.
 */
export declare function createVirtualTsVisitorHost(
  plugin: VisitorPlugin<readonly ["virtual-ts"]>,
): VirtualTsVisitorHostPlugin;
/**
 * Negotiate a devtools-trace-only descriptor and adapt it to the opt-in
 * `@vizejs/devtools` recorder boundary. Mixed-capability descriptors fail closed.
 */
export declare function createDevtoolsTraceVisitorHost(
  plugin: VisitorPlugin<readonly ["devtools-trace"]>,
): DevtoolsTraceVisitorHostPlugin;
