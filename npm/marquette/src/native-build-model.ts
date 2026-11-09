/** Stable discriminator for serialized native build profiles. */
export const NATIVE_BUILD_PROFILE_FORMAT = "vize.native-build-profile" as const;

/** Current serialized native build profile version. */
export const NATIVE_BUILD_PROFILE_FORMAT_VERSION = 1 as const;

/** Native application platforms understood by the managed build contract. */
export type NativeBuildPlatform = "android" | "ios";

/** Intended lifecycle stage of a native build. */
export type NativeBuildConfiguration = "development" | "preview" | "production";

/** Installation or distribution destination produced by one target. */
export type NativeBuildDestination = "simulator" | "device" | "store";

/** How signing material is selected without embedding credentials in the profile. */
export type NativeBuildSigning = "none" | "automatic" | "manual";

/** One platform and destination emitted from a native build profile. */
export interface NativeBuildTarget {
  /** Native platform to build. */
  readonly platform: NativeBuildPlatform;
  /** Installation or distribution destination. */
  readonly destination: NativeBuildDestination;
  /**
   * Signing selection implemented by the local or hosted executor.
   *
   * Credentials are supplied out of band and never serialized here.
   *
   * @default "none"
   */
  readonly signing?: NativeBuildSigning;
}

/** Over-the-air update selection attached to a native build. */
export interface NativeBuildUpdateSelection {
  /** Portable rollout channel identifier. */
  readonly channel: string;
  /** Runtime compatibility version checked before applying an update. */
  readonly runtimeVersion: string;
}

/**
 * Language-neutral profile consumed unchanged by local and hosted native builders.
 *
 * Execution location, credentials, and secret values deliberately stay outside
 * this record so the exact same profile can be admitted by either executor.
 */
export interface NativeBuildProfile {
  /** Serialized record discriminator. */
  readonly format: typeof NATIVE_BUILD_PROFILE_FORMAT;
  /** Serialized contract version. */
  readonly formatVersion: typeof NATIVE_BUILD_PROFILE_FORMAT_VERSION;
  /** Stable profile identifier, such as `development` or `production`. */
  readonly profile: string;
  /** Intended lifecycle stage of the produced artifacts. */
  readonly configuration: NativeBuildConfiguration;
  /** Native environment identifier declared by the application marquette. */
  readonly environment: string;
  /** Non-empty platform and destination matrix. */
  readonly targets: readonly NativeBuildTarget[];
  /**
   * Native capabilities required by every target in this profile.
   *
   * @default []
   */
  readonly capabilities?: readonly string[];
  /** Optional over-the-air update compatibility selection. */
  readonly updates?: NativeBuildUpdateSelection;
}

/** Stable validation code for a native build profile. */
export type NativeBuildProfileDiagnosticCode =
  | "invalid-format"
  | "invalid-format-version"
  | "invalid-profile-id"
  | "invalid-environment-id"
  | "missing-target"
  | "duplicate-target"
  | "store-requires-production"
  | "store-requires-signing"
  | "invalid-update-channel"
  | "invalid-runtime-version"
  | "invalid-capability-id"
  | "duplicate-capability";

/** Deterministic semantic validation diagnostic for a native build profile. */
export interface NativeBuildProfileDiagnostic {
  /** Stable machine-readable diagnostic code. */
  readonly code: NativeBuildProfileDiagnosticCode;
  /** JSON-style path of the invalid value. */
  readonly path: string;
  /** Human-readable explanation shared by every executor. */
  readonly message: string;
}
