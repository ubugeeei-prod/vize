export {
  typeCheck,
  typeCheckBatch,
  typeCheckWithVirtualTsVisitors,
  getTypeCheckCapabilities,
  beginTypeResolutionBatch,
  TypeResolutionBatch,
  typeProjectOwners,
  typeProjectInputs,
} from "@vizejs/native";
export type {
  BatchTypeCheckResultNapi,
  RelatedLocationNapi,
  TypeCheckOptionsNapi,
  TypeCheckResultNapi,
  TypeDiagnosticNapi,
  VirtualTsVisitorPluginNapi,
} from "@vizejs/native";
