import {
  defineNativeBuildProfile,
  type NativeBuildConfiguration,
  type NativeBuildPlatform,
  type NativeBuildProfile,
} from "./native-build.js";

const profile = defineNativeBuildProfile({
  format: "vize.native-build-profile",
  formatVersion: 1,
  profile: "development",
  configuration: "development",
  environment: "native",
  targets: [{ platform: "ios", destination: "simulator" }],
  capabilities: ["native.camera"],
});
const exactProfile: "development" = profile.profile;
const exactEnvironment: "native" = profile.environment;
const exactPlatform: "ios" = profile.targets[0].platform;
const genericProfile: NativeBuildProfile = profile;
const configuration: NativeBuildConfiguration = "preview";
const platform: NativeBuildPlatform = "android";

// @ts-expect-error Build configurations are a closed contract.
const invalidConfiguration: NativeBuildConfiguration = "debug";
// @ts-expect-error Native platforms are a closed contract.
const invalidPlatform: NativeBuildPlatform = "windows";

void exactProfile;
void exactEnvironment;
void exactPlatform;
void genericProfile;
void configuration;
void platform;
void invalidConfiguration;
void invalidPlatform;
