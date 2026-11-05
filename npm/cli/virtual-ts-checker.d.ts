// Globals of a program whose templates vize checks. Content-mapper modules load
// them with a type-only import that declaration emit drops, so an emitted
// declaration brings only the helpers, never this policy, into its consumer.
import type {} from "./virtual-ts.js" with { "resolution-mode": "import" };

declare global {
  // Nuxt and the JSX runtime declare these for their consumers (#11630).
  interface ImportMeta {
    client: boolean;
    server: boolean;
    dev: boolean;
    prod: boolean;
    ssr: boolean;
  }
  namespace JSX {
    interface IntrinsicAttributes {
      class?: unknown;
      style?: unknown;
    }
  }
  // vue-tsc generic library components accept unknown attrs through this
  // fallback, which vue-tsc itself declares unless `checkUnknownProps` is set.
  var __VLS_PROPS_FALLBACK: Record<string, unknown>;
  // vize checks these templates itself, so component input guards serve only
  // TS and TSX callers and keep extra keys exact.
  var __vizeTemplateChecker: true;
}

export {};
