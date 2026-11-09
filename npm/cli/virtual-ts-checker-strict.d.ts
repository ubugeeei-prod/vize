// Globals of a program whose templates vize checks with `checkUnknownProps`.
// Content-mapper modules load them with a type-only import that declaration
// emit drops, so an emitted declaration never brings them into its consumer.
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
}

export {};
