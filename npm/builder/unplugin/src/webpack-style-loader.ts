// Webpack runs the host's loaders right to left: sass-loader, then css-loader.
// Scoping must sit between them, so the pitch phase re-issues a scoped style
// file request with this loader (`?scope`) inserted after the preprocessor.
// vue-loader's pitcher uses the same technique.
import { scopeCss } from "./style-scope.ts";
import { webpackStyleScope } from "./webpack-styles.ts";

interface LoaderEntry {
  path: string;
  request: string;
}

interface StyleLoaderContext {
  loaders: LoaderEntry[];
  loaderIndex: number;
  query: string | Record<string, unknown>;
  resource: string;
  resourcePath: string;
  context: string;
  utils?: { contextify(context: string, request: string): string };
}

const SCOPE_QUERY = "?scope";
const CSS_LOADER = /(?:^|[/\\])css-loader(?:[/\\]|$)/;

export function pitch(this: StyleLoaderContext): string | undefined {
  if (this.query === SCOPE_QUERY) {
    return undefined;
  }
  const self = this.loaders[this.loaderIndex];
  const rest = this.loaders.filter((_, index) => index !== this.loaderIndex);
  const cssLoaderIndex = rest.findIndex((loader) => CSS_LOADER.test(loader.path));
  if (!self || cssLoaderIndex < 0) {
    return undefined;
  }

  const requests = rest.map((loader) => loader.request);
  requests.splice(cssLoaderIndex + 1, 0, `${self.path}${SCOPE_QUERY}`);
  const request = `!!${requests.join("!")}!${this.resource}`;
  const specifier = JSON.stringify(this.utils?.contextify(this.context, request) ?? request);
  const isModule = /\.module\.\w+$/.test(this.resourcePath);
  return isModule
    ? `export { default } from ${specifier};\nexport * from ${specifier};`
    : `export * from ${specifier};`;
}

// Also runs unpitched, last in a chain without css-loader (for example
// webpack's native CSS), where the source is already plain CSS.
export default function vizeStyleLoader(this: StyleLoaderContext, source: string): string {
  const scoped = webpackStyleScope(this.resourcePath);
  return scoped ? scopeCss(source, scoped) : source;
}
