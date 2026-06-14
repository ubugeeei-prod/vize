/// <reference types="vite-plus/client" />
/// <reference types="@vizejs/vite-plugin-musea/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}
