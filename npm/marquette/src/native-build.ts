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
