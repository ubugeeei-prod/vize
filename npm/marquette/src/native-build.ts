export {
  NATIVE_BUILD_PROFILE_FORMAT,
  NATIVE_BUILD_PROFILE_FORMAT_VERSION,
  type NativeBuildConfiguration,
  type NativeBuildDestination,
  type NativeBuildPlatform,
  type NativeBuildProfile,
  type NativeBuildProfileDiagnostic,
  type NativeBuildProfileDiagnosticCode,
  type NativeBuildSigning,
  type NativeBuildTarget,
  type NativeBuildUpdateSelection,
} from "./native-build-model.js";

import type { NativeBuildProfile } from "./native-build-model.js";
import type { ApplicationMarquette, CapabilityId } from "./model.js";

/** Native client environment identifiers inferred from one application contract. */
export type NativeApplicationEnvironmentId<Marquette extends ApplicationMarquette> =
  Marquette extends { readonly environments: readonly (infer Environment)[] }
    ? Environment extends {
        readonly id: infer Id extends string;
        readonly target: "native";
        readonly consumer: "client";
      }
      ? Id
      : never
    : never;

/** A native build profile whose environment and capabilities belong to the application. */
export type NativeApplicationBuildProfile<Marquette extends ApplicationMarquette> = Omit<
  NativeBuildProfile,
  "environment" | "capabilities"
> & {
  /** Native client environment declared by the application marquette. */
  readonly environment: NativeApplicationEnvironmentId<Marquette>;
  /**
   * Application-declared capabilities required by this build profile.
   *
   * @default []
   */
  readonly capabilities?: readonly CapabilityId<Marquette>[];
};

/**
 * Preserves literal profile, environment, platform, and capability types.
 *
 * The function returns its input unchanged so authoring has no runtime cost.
 */
export function defineNativeBuildProfile<const Profile extends NativeBuildProfile>(
  profile: Profile,
): Profile {
  return profile;
}

/**
 * Preserves build-profile literals while inferring native environments and capabilities.
 *
 * The application argument supplies types only; this helper returns the profile
 * unchanged. Use `admitNativeBuildProfile` from the separate admission entry when
 * profiles cross an untrusted boundary or before scheduling an actual build.
 */
export function defineNativeApplicationBuildProfile<
  const Marquette extends ApplicationMarquette,
  const Profile extends NativeApplicationBuildProfile<Marquette>,
>(marquette: Marquette, profile: Profile): Profile {
  void marquette;
  return profile;
}
