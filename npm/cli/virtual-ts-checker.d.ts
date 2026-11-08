// Globals of a program whose templates vize checks. Content-mapper modules load
// them with a type-only import that declaration emit drops, so an emitted
// declaration brings only the helpers, never this policy, into its consumer.
import type {} from "./virtual-ts-checker-strict.js" with { "resolution-mode": "import" };

declare global {
  // vue-tsc generic library components accept unknown attrs through this
  // fallback, which vue-tsc itself declares unless `checkUnknownProps` is set.
  var __VLS_PROPS_FALLBACK: Record<string, unknown>;
}

export {};
