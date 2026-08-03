export {
  ADAPTER_CAPABILITY_FORMAT_VERSION,
  type AdapterCapabilityCompatibilityChange,
  type AdapterCapabilityCompatibilityReport,
  type AdapterCapabilityDiagnostic,
  type AdapterCapabilityDiagnosticCode,
  type AdapterCapabilityManifest,
  type AdapterCapabilityMismatch,
  type AdapterCapabilityMismatchCode,
  type AdapterCapabilityNegotiation,
  type AdapterCapabilitySupport,
  type CompatibilityChangeKind,
} from "./adapter-model.js";
export {
  NATIVE_ENGINE_CAPABILITY_IDS,
  NATIVE_ENGINE_CAPABILITY_VERSION,
  nativeEngineCapabilityProfile,
  type NativeEngineCapabilityId,
} from "./adapter-native-profile.js";
export { compareAdapterCapabilities } from "./adapter-compatibility.js";
export { negotiateAdapterCapabilities } from "./adapter-negotiate.js";
export {
  parseAdapterCapabilityManifest,
  validateAdapterCapabilityManifest,
} from "./adapter-validate.js";
