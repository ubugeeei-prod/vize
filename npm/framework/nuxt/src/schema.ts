import type { VizeNuxtOptions } from "./options";
import type { NuxtLintConfigAddon } from "./lint/addons";

declare module "nuxt/schema" {
  interface NuxtConfig {
    vize?: Partial<VizeNuxtOptions>;
  }

  interface NuxtOptions {
    vize?: Partial<VizeNuxtOptions>;
  }

  interface NuxtHooks {
    "vize:lint:config:addons": (addons: NuxtLintConfigAddon[]) => void;
  }
}

declare module "@nuxt/schema" {
  interface NuxtConfig {
    vize?: Partial<VizeNuxtOptions>;
  }

  interface NuxtOptions {
    vize?: Partial<VizeNuxtOptions>;
  }

  interface NuxtHooks {
    "vize:lint:config:addons": (addons: NuxtLintConfigAddon[]) => void;
  }
}
