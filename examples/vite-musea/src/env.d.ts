/// <reference types="vite-plus/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

type MuseaArtStatus = "draft" | "ready" | "deprecated";

interface MuseaArtOptions {
  title?: string;
  description?: string;
  category?: string;
  tags?: string[];
  status?: MuseaArtStatus;
  order?: number;
}

declare function defineArt(source: string, options?: MuseaArtOptions): void;
