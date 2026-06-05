/// <reference types="vite-plus/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

interface Window {
  MonacoEnvironment?: {
    getWorker(workerId: string, label: string): Worker;
  };
}

declare const self: Window & typeof globalThis;
declare const __VIZE_VERSION__: string;
