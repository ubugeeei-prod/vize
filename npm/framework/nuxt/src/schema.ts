import type { VizeNuxtOptions } from "./options";

declare module "nuxt/schema" {
  interface NuxtConfig {
    vize?: Partial<VizeNuxtOptions>;
  }

  interface NuxtOptions {
    vize?: Partial<VizeNuxtOptions>;
  }
}

declare module "@nuxt/schema" {
  interface NuxtConfig {
    vize?: Partial<VizeNuxtOptions>;
  }

  interface NuxtOptions {
    vize?: Partial<VizeNuxtOptions>;
  }
}
