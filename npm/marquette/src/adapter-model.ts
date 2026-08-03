/** Current serialized adapter-capability manifest format. */
export const ADAPTER_CAPABILITY_FORMAT_VERSION = 1 as const;

/** Inclusive version range supported for one capability contract. */
export interface AdapterCapabilitySupport {
  /** Stable capability identifier from an application marquette. */
  readonly id: string;
  /** Oldest supported capability contract version, inclusive. */
  readonly minVersion: number;
  /** Newest supported capability contract version, inclusive. */
  readonly maxVersion: number;
}

/** Versioned, language-neutral capabilities offered by one adapter. */
export interface AdapterCapabilityManifest {
  /**
   * Serialized manifest format.
   *
   * @default 1
   */
  readonly formatVersion?: typeof ADAPTER_CAPABILITY_FORMAT_VERSION;
  /** Stable adapter identifier shown in diagnostics and reports. */
  readonly adapter: string;
  /**
   * Supported capability ranges.
   *
   * @default []
   */
  readonly capabilities?: readonly AdapterCapabilitySupport[];
}

/** Stable validation code for an adapter capability manifest. */
export type AdapterCapabilityDiagnosticCode =
  | "invalid-format-version"
  | "invalid-adapter-id"
  | "invalid-capability-id"
  | "invalid-version"
  | "invalid-version-range"
  | "duplicate-capability";

/** Deterministic validation diagnostic for an adapter capability manifest. */
export interface AdapterCapabilityDiagnostic {
  /** Stable machine-readable diagnostic code. */
  readonly code: AdapterCapabilityDiagnosticCode;
  /** JSON-style path of the invalid value. */
  readonly path: string;
  /** Human-readable explanation. */
  readonly message: string;
}

/** Stable incompatibility code emitted during capability negotiation. */
export type AdapterCapabilityMismatchCode =
  | "unknown-requirement"
  | "missing-capability"
  | "version-below-minimum"
  | "version-above-maximum";

/** One failed adapter capability requirement. */
export interface AdapterCapabilityMismatch {
  /** Stable machine-readable mismatch code. */
  readonly code: AdapterCapabilityMismatchCode;
  /** Required capability identifier. */
  readonly capability: string;
  /** Required contract version when the capability is declared. */
  readonly requiredVersion?: number;
  /** Adapter minimum when support exists. */
  readonly minVersion?: number;
  /** Adapter maximum when support exists. */
  readonly maxVersion?: number;
}

/** Deterministic result of negotiating requirements with one adapter. */
export interface AdapterCapabilityNegotiation {
  /** Adapter whose support was inspected. */
  readonly adapter: string;
  /** Whether the manifest is valid and every requirement is supported. */
  readonly compatible: boolean;
  /** Manifest validation failures, in stable path order. */
  readonly diagnostics: readonly AdapterCapabilityDiagnostic[];
  /** Unsupported or unknown requirements, in stable capability order. */
  readonly mismatches: readonly AdapterCapabilityMismatch[];
}

/** Compatibility classification for one adapter capability change. */
export type CompatibilityChangeKind = "additive" | "breaking";

/** One stable adapter capability compatibility change. */
export interface AdapterCapabilityCompatibilityChange {
  /** Compatibility classification. */
  readonly kind: CompatibilityChangeKind;
  /** JSON-style path of the changed capability support. */
  readonly path: string;
  /** Human-readable summary. */
  readonly message: string;
}

/** Deterministic compatibility report between two adapter manifests. */
export interface AdapterCapabilityCompatibilityReport {
  /** All changes in stable path order. */
  readonly changes: readonly AdapterCapabilityCompatibilityChange[];
}
