type MuseaArtStatus = "draft" | "ready" | "deprecated";

interface MuseaArtOptions {
  title?: string;
  description?: string;
  category?: string;
  tags?: string[];
  status?: MuseaArtStatus;
  order?: number;
  actionEvents?: string[];
}

declare function defineArt(source: string, options?: MuseaArtOptions): void;
declare function defineArt<TComponent>(component: TComponent, options?: MuseaArtOptions): void;

declare module "*.art.vue" {
  const component: import("vue").DefineComponent<{}, {}, any>;
  export default component;
}
