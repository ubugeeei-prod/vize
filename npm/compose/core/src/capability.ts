/** Runtime families understood by composable capability metadata and adapters. */
export type CapabilityTarget = "web" | "server" | "worker" | "native" | "desktop" | "terminal";

/** Built-in origins from which a capability value can be resolved. */
export type CapabilitySource = "runtime" | "adapter" | "fallback";

/**
 * Standard reasons why a capability cannot currently provide a value.
 *
 * Adapters may extend this vocabulary with a narrower string-literal union;
 * consumers should preserve unknown extension reasons when crossing process
 * or version boundaries.
 */
export type CapabilityUnavailableReason =
  | "unsupported"
  | "unavailable"
  | "permission-denied"
  | "insecure-context"
  | "not-active"
  | "not-ready"
  | "cancelled"
  | "adapter-missing"
  | "adapter-error";

/** A capability value that is ready for use. */
export interface AvailableCapability<Value, Source extends string = CapabilitySource> {
  /** Stable discriminant for serialization and exhaustive matching. */
  readonly status: "available";

  /** Boolean discriminant for ergonomic guards in templates and JavaScript. */
  readonly available: true;

  /** Resolved capability implementation or value. */
  readonly value: Value;

  /** Origin that supplied the value, preserved as a string literal. */
  readonly source: Source;
}

/** A capability that cannot currently provide a value. */
export interface UnavailableCapability<
  Reason extends string = CapabilityUnavailableReason,
  Details = undefined,
> {
  /** Stable discriminant for serialization and exhaustive matching. */
  readonly status: "unavailable";

  /** Boolean discriminant for ergonomic guards in templates and JavaScript. */
  readonly available: false;

  /** Typed, machine-readable explanation of the unavailable state. */
  readonly reason: Reason;

  /** Structured diagnostic or recovery information supplied by the adapter. */
  readonly details: Details;
}

/** Explicit result of capability discovery or adapter negotiation. */
export type CapabilityResult<
  Value,
  Reason extends string = CapabilityUnavailableReason,
  Details = undefined,
  Source extends string = CapabilitySource,
> = AvailableCapability<Value, Source> | UnavailableCapability<Reason, Details>;

/**
 * Create an available capability supplied directly by the current runtime.
 *
 * The value and source retain literal types. The helper performs no global
 * access and is safe during server rendering and module evaluation.
 *
 * @param value Resolved capability implementation or value.
 * @returns An available result whose source is `"runtime"`.
 */
export function availableCapability<const Value>(
  value: Value,
): AvailableCapability<Value, "runtime">;

/**
 * Create an available capability with an explicit, literal-preserving source.
 *
 * @param value Resolved capability implementation or value.
 * @param source Runtime, adapter, fallback, or adapter-specific source name.
 * @returns An available capability result.
 */
export function availableCapability<const Value, const Source extends string>(
  value: Value,
  source: Source,
): AvailableCapability<Value, Source>;

export function availableCapability<const Value, const Source extends string>(
  value: Value,
  source?: Source,
): AvailableCapability<Value, Source | "runtime"> {
  return {
    status: "available",
    available: true,
    value,
    source: source ?? "runtime",
  };
}

/**
 * Create an unavailable capability without additional details.
 *
 * The reason retains its string-literal type. Capability absence is returned
 * as data rather than thrown, making unsupported server, worker, native,
 * desktop, and terminal environments deterministic.
 *
 * @param reason Machine-readable unavailability reason.
 * @returns An unavailable result with `undefined` details.
 */
export function unavailableCapability<const Reason extends string>(
  reason: Reason,
): UnavailableCapability<Reason, undefined>;

/**
 * Create an unavailable capability with typed recovery or diagnostic details.
 *
 * Passing `undefined` explicitly is distinct at the call boundary and still
 * preserves the exact `undefined` details type.
 *
 * @param reason Machine-readable unavailability reason.
 * @param details Structured diagnostic or recovery information.
 * @returns An unavailable capability result.
 */
export function unavailableCapability<const Reason extends string, const Details>(
  reason: Reason,
  details: Details,
): UnavailableCapability<Reason, Details>;

export function unavailableCapability<const Reason extends string, const Details>(
  reason: Reason,
  details?: Details,
): UnavailableCapability<Reason, Details | undefined> {
  return {
    status: "unavailable",
    available: false,
    reason,
    details,
  };
}

/**
 * Narrow a capability result to its available branch.
 *
 * @param result Capability discovery or negotiation result.
 * @returns Whether `result.value` is ready for use.
 */
export function isCapabilityAvailable<Value, Reason extends string, Details, Source extends string>(
  result: CapabilityResult<Value, Reason, Details, Source>,
): result is AvailableCapability<Value, Source> {
  return result.available;
}

/**
 * Narrow a capability result to its unavailable branch.
 *
 * @param result Capability discovery or negotiation result.
 * @returns Whether the capability has a typed unavailability reason.
 */
export function isCapabilityUnavailable<
  Value,
  Reason extends string,
  Details,
  Source extends string,
>(
  result: CapabilityResult<Value, Reason, Details, Source>,
): result is UnavailableCapability<Reason, Details> {
  return !result.available;
}
