declare module "*.css" {}

declare module "virtual:vize-ui-examples" {
  import type { Component } from "vue";

  export const examples: Record<
    string,
    {
      title: string;
      sourceSha256: string;
      load: () => Promise<{ default: Component }>;
    }
  >;
  export const composables: Record<string, (typeof examples)[string] & { ssrHtml: string }>;
}
