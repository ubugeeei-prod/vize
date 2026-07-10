declare module "*.css";

declare module "@nuxt/ui/components/*.vue" {
  const component: import("vue").DefineComponent<Record<string, unknown>, {}, any>;
  export default component;
}

declare module "vue-select" {
  const component: import("vue").DefineComponent<Record<string, unknown>, {}, any>;
  export default component;
}

declare module "vue-i18n" {
  export function useI18n(options?: Record<string, unknown>): {
    t: (key: string) => string;
    locale: import("vue").Ref<string>;
  };
}

declare module "vue" {
  export interface GlobalComponents {
    TresAmbientLight: import("vue").DefineComponent<Record<string, unknown>, {}, any>;
    TresBoxGeometry: import("vue").DefineComponent<Record<string, unknown>, {}, any>;
    TresMesh: import("vue").DefineComponent<Record<string, unknown>, {}, any>;
    TresMeshStandardMaterial: import("vue").DefineComponent<Record<string, unknown>, {}, any>;
    TresPerspectiveCamera: import("vue").DefineComponent<Record<string, unknown>, {}, any>;
  }
}
