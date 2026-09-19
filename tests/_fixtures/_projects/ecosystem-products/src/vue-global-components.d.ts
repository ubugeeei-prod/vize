import "vue";

declare module "vue" {
  export interface GlobalComponents {
    TresAmbientLight: import("vue").DefineComponent<Record<string, unknown>, {}, any>;
    TresBoxGeometry: import("vue").DefineComponent<Record<string, unknown>, {}, any>;
    TresMesh: import("vue").DefineComponent<Record<string, unknown>, {}, any>;
    TresMeshStandardMaterial: import("vue").DefineComponent<Record<string, unknown>, {}, any>;
    TresPerspectiveCamera: import("vue").DefineComponent<Record<string, unknown>, {}, any>;
  }
}
