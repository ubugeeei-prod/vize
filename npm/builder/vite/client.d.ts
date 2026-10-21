declare module "*.vue" {
  import type { DefineComponent } from "vue";

  const component: DefineComponent<{}, {}, any>;
  export default component;
}

declare module "*.md" {
  import type { DefineComponent } from "vue";

  export interface OxContentTocEntry {
    depth: number;
    text: string;
    slug: string;
    children: OxContentTocEntry[];
  }

  export const html: string;
  export const frontmatter: Record<string, unknown>;
  export const toc: OxContentTocEntry[];

  const component: DefineComponent<{}, {}, any>;
  export default component;
}

declare module "*.markdown" {
  import type { DefineComponent } from "vue";

  export interface OxContentTocEntry {
    depth: number;
    text: string;
    slug: string;
    children: OxContentTocEntry[];
  }

  export const html: string;
  export const frontmatter: Record<string, unknown>;
  export const toc: OxContentTocEntry[];

  const component: DefineComponent<{}, {}, any>;
  export default component;
}
