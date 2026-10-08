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
}
