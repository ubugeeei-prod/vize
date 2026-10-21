import Page, { frontmatter, html, toc } from "./Page.md";
import LongPage, {
  frontmatter as longFrontmatter,
  html as longHtml,
  toc as longToc,
} from "./Page.markdown";
import type { DefineComponent } from "vue";

const markdownComponent: DefineComponent<{}, {}, any> = Page;
const longMarkdownComponent: DefineComponent<{}, {}, any> = LongPage;
const renderedHtml: string = html;
const longRenderedHtml: string = longHtml;
const headingDepth: number | undefined = toc[0]?.depth;
const nestedSlug: string | undefined = longToc[0]?.children[0]?.slug;
const title: unknown = frontmatter.title;
const longTitle: unknown = longFrontmatter.title;

// @ts-expect-error Frontmatter keys are intentionally unknown until a project narrows its schema.
const unsafeTitle: string = frontmatter.title;

void markdownComponent;
void longMarkdownComponent;
void renderedHtml;
void longRenderedHtml;
void headingDepth;
void nestedSlug;
void title;
void longTitle;
void unsafeTitle;
